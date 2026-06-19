//! MeshNode - the main entry point for using the mesh

use crate::address::{ServiceAddress, ServiceId};
use crate::identity::NodeIdentity;
use crate::message::{Request, Response};
use crate::registry::ServiceRegistry;
use crate::transport::Transport;
use crate::{MeshError, Result};

#[cfg(feature = "lan")]
use crate::lan::LanDiscovery;

// A-4b cfg-gate: `libp2p::Multiaddr` is unconditionally consumed inside
// the `#[cfg(feature = "p2p")]` `dial` method (and any future p2p-gated
// address parsing). Gate the import on the same feature so the
// default-build arniko (which keeps `local` only) doesn't trip E0432
// `unresolved import \`libp2p\`` against a cfg-stripped libp2p dep.
#[cfg(feature = "p2p")]
use libp2p::Multiaddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

/// Derive a libp2p `PeerId` directly from an Ed25519 DID (`did:key:z…` or
/// `did:exo:…`). Both schemes encode `bs58(0xed01 || pubkey32)` after the
/// scheme prefix (`did:key:` adds a `z` multibase indicator). A libp2p PeerId
/// is deterministic from the public key, so this lets `call_agent` route to a
/// peer over an existing connection when DHT discovery is unavailable. (s198)
#[cfg(feature = "p2p")]
fn peer_id_from_did(did: &str) -> Result<libp2p::PeerId> {
    let encoded = did
        .strip_prefix("did:key:z")
        .or_else(|| did.strip_prefix("did:exo:"))
        .ok_or_else(|| {
            MeshError::Transport(format!(
                "Unsupported DID scheme for PeerId derivation: {}",
                did
            ))
        })?;
    let bytes = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| MeshError::Transport(format!("DID base58 decode failed: {}", e)))?;
    if bytes.len() < 34 || bytes[0] != 0xed || bytes[1] != 0x01 {
        return Err(MeshError::Transport(format!(
            "DID multicodec is not Ed25519 (0xed01): {}",
            did
        )));
    }
    let pubkey: [u8; 32] = bytes[2..34]
        .try_into()
        .map_err(|_| MeshError::Transport("DID public key is not 32 bytes".to_string()))?;
    let ed_pub = libp2p::identity::ed25519::PublicKey::try_from_bytes(&pubkey)
        .map_err(|e| MeshError::Transport(format!("Invalid Ed25519 public key in DID: {}", e)))?;
    Ok(libp2p::identity::PublicKey::from(ed_pub).to_peer_id())
}

/// Configuration for a mesh node
#[derive(Debug, Clone)]
pub struct MeshConfig {
    /// Path to store node identity
    pub identity_path: PathBuf,
    /// Enable local Unix socket transport
    pub enable_local: bool,
    /// Enable mDNS LAN discovery
    pub enable_lan: bool,
    /// Enable P2P internet transport
    pub enable_p2p: bool,
    /// Bootstrap peers for P2P
    pub bootstrap_peers: Vec<String>,
    /// Stale service cleanup interval
    pub cleanup_interval: Duration,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            identity_path: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/var/lib"))
                .join("exosphere/mesh/identity.key"),
            enable_local: true,
            enable_lan: true,
            enable_p2p: true,
            bootstrap_peers: vec![],
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// A node in the Exosphere mesh network
pub struct MeshNode {
    /// Node identity
    identity: Arc<NodeIdentity>,
    /// Service registry
    registry: Arc<ServiceRegistry>,
    /// Transport layer
    transport: Arc<Transport>,
    /// Configuration
    config: MeshConfig,
    /// P2P transport (if enabled)
    #[cfg(feature = "p2p")]
    p2p: Option<Arc<crate::p2p::P2pTransport>>,
    /// LAN discovery (if enabled)
    #[cfg(feature = "lan")]
    lan: Option<Arc<LanDiscovery>>,
}

impl MeshNode {
    /// Create and start a new mesh node
    pub async fn start(config: MeshConfig) -> Result<Self> {
        // Load or generate identity
        let identity = NodeIdentity::load_or_generate(&config.identity_path)
            .map_err(|e| MeshError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        tracing::info!("Mesh node starting with DID: {}", identity.did());

        let identity = Arc::new(identity);
        let registry = Arc::new(ServiceRegistry::new(identity.did().to_string()));

        // Start P2P if enabled
        #[cfg(feature = "p2p")]
        let p2p = if config.enable_p2p {
            let bootstrap: Vec<libp2p::Multiaddr> = config
                .bootstrap_peers
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();

            match crate::p2p::P2pTransport::start(
                NodeIdentity::load_or_generate(&config.identity_path).unwrap(),
                bootstrap,
            )
            .await
            {
                Ok(p2p) => Some(Arc::new(p2p)),
                Err(e) => {
                    tracing::warn!("P2P transport failed to start: {}", e);
                    return Err(e);
                }
            }
        } else {
            None
        };

        // Build transport
        let mut transport = Transport::new();
        #[cfg(feature = "p2p")]
        if let Some(ref p2p) = p2p {
            transport = transport.with_p2p(Arc::clone(p2p));
        }
        let transport = Arc::new(transport);

        // Start LAN discovery if enabled
        #[cfg(feature = "lan")]
        let lan = if config.enable_lan {
            Some(Arc::new(LanDiscovery::new(
                Arc::clone(&registry),
                identity.did().to_string(),
                Arc::clone(&identity),
            )))
        } else {
            None
        };

        let node = Self {
            identity,
            registry,
            transport,
            config: config.clone(),
            #[cfg(feature = "p2p")]
            p2p,
            #[cfg(feature = "lan")]
            lan,
        };

        // Start background tasks
        node.start_background_tasks();

        Ok(node)
    }

    fn start_background_tasks(&self) {
        let registry = Arc::clone(&self.registry);
        let cleanup_interval = self.config.cleanup_interval;

        // Stale service cleanup
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                registry.cleanup_stale(Duration::from_secs(300));
            }
        });

        // mDNS discovery (if enabled)
        #[cfg(feature = "lan")]
        if let Some(ref lan) = self.lan {
            let lan_clone = Arc::clone(lan);
            tokio::spawn(async move {
                if let Err(e) = lan_clone.start().await {
                    tracing::warn!("LAN discovery failed: {}", e);
                }
            });
        }
    }

    /// Get the node's DID
    pub fn did(&self) -> &str {
        self.identity.did()
    }

    /// Get the node's libp2p PeerId (only when the `p2p` feature is
    /// enabled — `NodeIdentity::peer_id()` itself is gated on `p2p` in
    /// `identity.rs` since a PeerId requires a libp2p keypair to construct).
    #[cfg(feature = "p2p")]
    pub fn peer_id(&self) -> String {
        self.identity.peer_id()
    }

    /// Get a short fingerprint for display
    pub fn fingerprint(&self) -> String {
        self.identity.fingerprint()
    }

    /// Register a local service
    pub fn register_service(
        &self,
        service_id: ServiceId,
        methods: Vec<String>,
        version: &str,
    ) -> ServiceAddress {
        let addr = self
            .registry
            .register_local(service_id, methods, version.to_string());
        tracing::info!("Registered service: {}", addr);
        addr
    }

    /// Unregister a local service
    pub fn unregister_service(&self, service_id: &ServiceId) -> bool {
        self.registry.unregister_local(service_id)
    }

    /// Find a service by ID
    pub fn find_service(&self, service_id: &ServiceId) -> Option<ServiceAddress> {
        self.registry.find(service_id)
    }

    /// Find all instances of a service
    pub fn find_all_services(&self, service_id: &ServiceId) -> Vec<ServiceAddress> {
        self.registry.find_all(service_id)
    }

    /// Call a service method
    pub async fn call(
        &self,
        service_id: &ServiceId,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let address = self
            .find_service(service_id)
            .ok_or_else(|| MeshError::ServiceNotFound(service_id.to_string()))?;

        let request = Request::new(method, params).with_caller(self.did());

        let response = self.transport.send(&address, request).await?;

        response
            .into_result()
            .map_err(|e| MeshError::Transport(e.to_string()))
    }

    /// Call a service by URL
    pub async fn call_url(
        &self,
        url: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let address = ServiceAddress::from_url(url)
            .ok_or_else(|| MeshError::ServiceNotFound(url.to_string()))?;

        let request = Request::new(method, params).with_caller(self.did());

        let response = self.transport.send(&address, request).await?;

        response
            .into_result()
            .map_err(|e| MeshError::Transport(e.to_string()))
    }

    /// Get the transport for direct access
    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    /// Get the registry for direct access
    pub fn registry(&self) -> &ServiceRegistry {
        &self.registry
    }

    /// Create a client for a specific service
    pub fn client(&self, service_id: ServiceId) -> MeshClient<'_> {
        MeshClient {
            node: self,
            service_id,
        }
    }

    /// Create a service builder
    pub fn service(&self, service_id: ServiceId) -> ServiceBuilder<'_> {
        ServiceBuilder {
            node: self,
            service_id,
            methods: vec![],
            version: "0.1.0".to_string(),
        }
    }

    /// Subscribe to a gossip topic
    #[cfg(feature = "p2p")]
    pub async fn subscribe(&self, topic: String) -> Result<broadcast::Receiver<(String, Vec<u8>)>> {
        self.transport.subscribe(topic).await
    }

    /// Publish to a gossip topic
    #[cfg(feature = "p2p")]
    pub async fn publish(&self, topic: String, message: Vec<u8>) -> Result<()> {
        self.transport.publish(topic, message).await
    }

    /// Put data into DHT
    #[cfg(feature = "p2p")]
    pub async fn dht_put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        self.transport.dht_put(key, value).await
    }

    /// Get data from DHT
    #[cfg(feature = "p2p")]
    pub async fn dht_get(&self, key: Vec<u8>) -> Result<Vec<Vec<u8>>> {
        self.transport.dht_get(key).await
    }

    /// Get P2P listeners
    #[cfg(feature = "p2p")]
    pub async fn p2p_listeners(&self) -> Vec<String> {
        self.transport.listeners().await
    }

    /// Count of currently-connected mesh peers (FED-04 federation status).
    /// Returns 0 when no P2P transport is attached.
    #[cfg(feature = "p2p")]
    pub async fn connected_peer_count(&self) -> usize {
        self.transport.connected_peer_count().await
    }

    /// Register this node's DID in the Kademlia DHT.
    ///
    /// Call after `MeshNode::start()` to make this node discoverable by DID.
    /// Stores a JSON-serialized `AgentRecord` keyed by the node's DID bytes.
    #[cfg(feature = "p2p")]
    pub async fn agent_register(&self, agent_type: &str) -> Result<()> {
        let did = self.identity.did().to_string();
        let peer_id = self
            .identity
            .to_libp2p_keypair()
            .public()
            .to_peer_id()
            .to_string();
        let listeners = self.transport.listeners().await;

        let record = crate::agent_registry::AgentRecord::new(
            did.clone(),
            peer_id,
            listeners,
            agent_type.to_string(),
        );

        let key = did.into_bytes();
        let value = serde_json::to_vec(&record)
            .map_err(|e| MeshError::Transport(format!("Serialization error: {}", e)))?;

        self.dht_put(key, value).await
    }

    /// Trigger DHT bootstrap to connect to known peers and propagate records
    #[cfg(feature = "p2p")]
    pub async fn bootstrap(&self) -> Result<()> {
        if let Some(p2p) = &self.p2p {
            p2p.bootstrap().await?;
        }
        Ok(())
    }

    /// Dial a peer by multiaddr
    #[cfg(feature = "p2p")]
    pub async fn dial(&self, addr: &str) -> Result<()> {
        if let Some(p2p) = &self.p2p {
            let multiaddr: Multiaddr = addr
                .parse()
                .map_err(|e| MeshError::Transport(format!("Invalid address: {}", e)))?;

            // Extract peer ID from multiaddr if present
            let peer_id = multiaddr.iter().find_map(|p| match p {
                libp2p::multiaddr::Protocol::P2p(pid) => Some(pid),
                _ => None,
            });

            if let Some(peer_id) = peer_id {
                p2p.dial(peer_id, Some(multiaddr)).await?;
            } else {
                // No peer ID in address - can't dial without it
                return Err(MeshError::Transport(
                    "Address must contain /p2p/ peer ID".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Discover an agent by DID from the Kademlia DHT.
    ///
    /// Returns the `AgentRecord` if found, or `MeshError::ServiceNotFound` if not in DHT.
    #[cfg(feature = "p2p")]
    pub async fn agent_discover(&self, did: &str) -> Result<crate::agent_registry::AgentRecord> {
        let key = did.as_bytes().to_vec();
        let values = self.dht_get(key).await?;

        values
            .into_iter()
            .find_map(|v| serde_json::from_slice::<crate::agent_registry::AgentRecord>(&v).ok())
            .ok_or_else(|| MeshError::ServiceNotFound(did.to_string()))
    }

    /// Publish a signed RFC-0012-v2 `NameClaim` to the Kademlia DHT (EXO-173).
    ///
    /// The record is stored at the RFC-0012 v1 alias key —
    /// `sha256("myc-alias:" + federation_id + ":" + name)`, derived in
    /// `mesh_proto::name_claim::name_alias_key` so publisher and resolver
    /// agree across implementations. The value is the claim as opaque JSON
    /// bytes: kad `Record.value` is a raw byte field that is never re-encoded
    /// by a structured codec, so the EXO-157 cbor4ii `Null → []` trap does not
    /// apply on this path (it bit `request_response` payloads, not DHT
    /// records); `expires_at: None` survives verbatim.
    ///
    /// Publishing does NOT verify the claim — the trust boundary is
    /// verify-on-receive ([`Self::name_resolve`]); a relay cannot launder a
    /// forged claim past a resolver.
    #[cfg(feature = "p2p")]
    pub async fn name_publish(
        &self,
        claim: &mesh_proto::NameClaim,
        federation_id: &str,
    ) -> Result<()> {
        mesh_proto::name_claim::validate_name(&claim.name)
            .map_err(|e| MeshError::Encoding(e.to_string()))?;
        let key = mesh_proto::name_alias_key(federation_id, &claim.name).to_vec();
        let value = serde_json::to_vec(claim)?;
        self.dht_put(key, value).await
    }

    /// Resolve a name from the DHT and verify it before returning (EXO-173).
    ///
    /// Fetches the `NameClaim` at the RFC-0012 alias key, then walks parent
    /// claims (`gpu.bob` → `bob` → …) fetching each from the DHT, and verifies
    /// the whole chain — ownership links plus Ed25519 signatures — against
    /// `root_registrar_pubkey_mb`, the federation registrar's z-base58btc
    /// public key (a parameter, never hardcoded). A claim that fails
    /// verification is REJECTED with `MeshError::AuthFailed`; an unverified
    /// claim is never returned to the caller.
    #[cfg(feature = "p2p")]
    pub async fn name_resolve(
        &self,
        name: &str,
        federation_id: &str,
        root_registrar_pubkey_mb: &str,
    ) -> Result<mesh_proto::NameClaim> {
        mesh_proto::name_claim::validate_name(name)
            .map_err(|e| MeshError::Encoding(e.to_string()))?;

        // Leaf-first chain: claims[i + 1] is the parent of claims[i]. Depth is
        // bounded by the name's label count, so a malicious parent pointer
        // cannot send the resolver on an unbounded walk.
        let mut chain: Vec<mesh_proto::NameClaim> = Vec::new();
        let mut cursor = name.to_string();
        for _ in 0..=name.matches('.').count() {
            let claim = self.name_fetch(&cursor, federation_id).await?;
            let parent = claim.parent.clone();
            chain.push(claim);
            if parent.is_empty() {
                break;
            }
            cursor = parent;
        }

        mesh_proto::verify_chain(&chain, root_registrar_pubkey_mb)
            .map_err(|e| MeshError::AuthFailed(format!("name claim for '{name}' rejected: {e}")))?;

        Ok(chain.into_iter().next().expect("chain verified non-empty"))
    }

    /// Fetch a single raw `NameClaim` record from the DHT (no verification —
    /// internal helper for [`Self::name_resolve`]).
    #[cfg(feature = "p2p")]
    async fn name_fetch(
        &self,
        name: &str,
        federation_id: &str,
    ) -> Result<mesh_proto::NameClaim> {
        let key = mesh_proto::name_alias_key(federation_id, name).to_vec();
        let values = self.dht_get(key).await?;
        values
            .into_iter()
            .find_map(|v| serde_json::from_slice::<mesh_proto::NameClaim>(&v).ok())
            .ok_or_else(|| MeshError::ServiceNotFound(format!("name claim '{name}'")))
    }

    /// Invoke a method on a remote agent identified by DID.
    ///
    /// Steps:
    /// 1. Discover the agent's P2P identity via Kademlia DHT (`agent_discover`)
    /// 2. Parse the PeerId from the returned `AgentRecord`
    /// 3. Dial the peer's first listener address if provided (non-fatal if already connected)
    /// 4. Send the method + params via request_response protocol
    /// 5. Return the response value
    ///
    /// Register an async handler for an inbound mesh request method.
    /// Each peer that calls `call_agent(did, method, …)` will route here on
    /// the receiving side. Methods without a registered handler get the
    /// legacy `{"status": "received"}` stub. (s198 — used by XipTransport to
    /// wire `xip.dispatch` so cross-host XIP RPC reaches a local dispatcher.)
    pub fn register_inbound_handler<F, Fut>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = serde_json::Value> + Send + 'static,
    {
        #[cfg(feature = "p2p")]
        if let Some(p2p) = self.p2p.as_ref() {
            p2p.register_inbound_handler(method, handler);
        }
    }

    /// Used by `XipTransport.send_via_mesh()` for A2A invocation.
    ///
    /// NOTE: Phase 2 response is a fire-and-forget acknowledgment `{"status":"sent"}`.
    /// Full request-response correlation (tracking OutboundRequestId → caller via oneshot)
    /// is Phase 3 work.
    ///
    /// Returns `MeshError::Transport` if P2P transport is not enabled or not configured.
    pub async fn call_agent(
        &self,
        did: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        #[cfg(feature = "p2p")]
        {
            let p2p = self
                .p2p
                .as_ref()
                .ok_or_else(|| MeshError::Transport("P2P transport not enabled".to_string()))?;

            // Step 1+2: Discover agent's P2P identity via DHT. If DHT discovery
            // fails (e.g. Kademlia routing didn't populate over a one-directional-
            // dialable link — ufw blocking the reverse-dial), fall back to
            // deriving the PeerId directly from the DID. Ed25519 DIDs encode the
            // public key, and a libp2p PeerId is deterministically derived from
            // that key, so a peer reachable over an existing connection (e.g.
            // one opened by bootstrap) can still be routed to without DHT. (s198)
            let peer_id: libp2p::PeerId = match self.agent_discover(did).await {
                Ok(agent_record) => {
                    let pid: libp2p::PeerId = agent_record.peer_id.parse().map_err(|e| {
                        MeshError::Transport(format!(
                            "Invalid PeerId in AgentRecord for DID {}: {}",
                            did, e
                        ))
                    })?;
                    // Step 3: Ensure a connection exists — but ONLY dial if we
                    // aren't already connected. A redundant dial to an existing
                    // peer is cancelled by libp2p (`PeerCondition::Disconnected`)
                    // and, over proot's seccomp/ptrace network layer, races and
                    // churns the live connection so the request_response reply
                    // never returns (the s198 failure mode → call times out →
                    // null). When already connected (e.g. via bootstrap), skip
                    // the dial and let request_response reuse the open link.
                    if !p2p.is_connected(&pid).await {
                        for addr_str in &agent_record.listeners {
                            if let Ok(addr) = addr_str.parse::<libp2p::Multiaddr>() {
                                let _ = p2p.dial(pid, Some(addr)).await;
                                break;
                            }
                        }
                    }
                    pid
                }
                Err(disco_err) => {
                    tracing::debug!(
                        "agent_discover failed for {} ({}); falling back to DID-derived PeerId",
                        did,
                        disco_err
                    );
                    peer_id_from_did(did)?
                }
            };

            // Step 4: Send the method + params via P2P request_response and
            // WAIT for the correlated reply. `send_and_wait` parks a oneshot
            // under a correlation_id and resolves it when the peer's response
            // arrives; the inbound handler echoes that correlation_id as the
            // response id so the match succeeds. (Previously this used the
            // fire-and-forget `send_to_peer`, which returned a `{"status":
            // "sent"}` stub and dropped the real reply. s198.)
            let request = crate::message::Request::new(method, params).with_caller(self.did());
            let message = crate::message::MeshMessage::Request(request);
            let response = p2p.send_and_wait(peer_id, message, None).await?;

            // Step 5: Return the response payload
            return Ok(response
                .into_result()
                .unwrap_or_else(|e| serde_json::json!({ "error": e.to_string() })));
        }

        #[cfg(not(feature = "p2p"))]
        {
            let _ = (did, method, params);
            Err(MeshError::Transport(
                "call_agent requires the 'p2p' feature to be enabled".to_string(),
            ))
        }
    }
}

/// Client for calling a specific service
pub struct MeshClient<'a> {
    node: &'a MeshNode,
    service_id: ServiceId,
}

impl<'a> MeshClient<'a> {
    /// Call a method on the service
    pub async fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        self.node.call(&self.service_id, method, params).await
    }
}

/// Builder for registering a service
pub struct ServiceBuilder<'a> {
    node: &'a MeshNode,
    service_id: ServiceId,
    methods: Vec<String>,
    version: String,
}

impl<'a> ServiceBuilder<'a> {
    pub fn method(mut self, name: impl Into<String>) -> Self {
        self.methods.push(name.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Register the service and start serving
    pub async fn serve<F, Fut>(self, handler: F) -> Result<()>
    where
        F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send,
    {
        let addr = self
            .node
            .register_service(self.service_id.clone(), self.methods, &self.version);

        let server = crate::local::LocalServer::bind(&addr.local_socket_path()).await?;
        server.serve(handler).await
    }
}

// Helper module for home directory
mod dirs {
    use std::path::PathBuf;

    pub fn data_dir() -> Option<PathBuf> {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_node_creation() {
        let mut temp_file = NamedTempFile::new().unwrap();

        let config = MeshConfig {
            identity_path: temp_file.path().to_path_buf(),
            enable_p2p: false,
            enable_lan: false,
            ..Default::default()
        };

        let node = MeshNode::start(config).await.unwrap();
        assert!(node.did().starts_with("did:key:"));
    }

    #[test]
    fn test_service_registration() {
        let identity = NodeIdentity::generate();
        let registry = ServiceRegistry::new(identity.did().to_string());

        let id = ServiceId::new("test", "service");
        registry.register_local(id.clone(), vec!["method1".to_string()], "0.1.0".to_string());

        assert!(registry.find(&id).is_some());
    }
}

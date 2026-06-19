//! P2P transport using libp2p
//!
//! Provides internet-wide connectivity via:
//! - QUIC transport with Noise encryption
//! - Kademlia DHT for peer discovery
//! - Relay protocol for NAT traversal

use crate::address::ServiceId;
use crate::identity::NodeIdentity;
use crate::message::{MeshMessage, Request, Response};
use crate::MeshError;

use futures::StreamExt;
// A-4b cfg-gate: `libp2p::quic` import is gated behind the `quic` sub-feature
// to keep the rcgen/E0119 conflict out of the default-build libp2p tree.
#[cfg(feature = "quic")]
use libp2p::quic;
use libp2p::request_response::{self, cbor};
use libp2p::tcp;
use libp2p::StreamProtocol;
use libp2p::Transport;
// A-4b cfg-gate: `libp2p::relay` moved out of the `use libp2p::{...}`
// group so we can cfg-gate it independently of the always-on core imports.
#[cfg(feature = "relay")]
use libp2p::relay;
use libp2p::{
    identity::Keypair,
    kad::{self, store::MemoryStore},
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};

/// Async handler for inbound mesh requests. Takes the request params and
/// returns a JSON result. Stored boxed + pinned in the inbound handler
/// registry. Registered per-method via `register_inbound_handler`. (s198)
pub type InboundHandler = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn std::future::Future<Output = serde_json::Value> + Send>>
        + Send
        + Sync,
>;

/// Extract the wire method name + params from an outbound `MeshMessage`.
///
/// For `MeshMessage::Request` this is the inner request's `method` + `params`,
/// so the receiver's per-method inbound handler routing sees the real method
/// (e.g. "xip.dispatch") instead of a hardcoded "mesh.message". Other message
/// variants serialize whole under the generic "mesh.message" method. (s198)
fn message_method_and_params(message: &MeshMessage) -> (String, serde_json::Value) {
    match message {
        MeshMessage::Request(req) => (req.method.clone(), req.params.clone()),
        other => (
            "mesh.message".to_string(),
            serde_json::to_value(other).unwrap_or(serde_json::Value::Null),
        ),
    }
}

pub use mesh_proto::{MeshRequest, MeshResponse};

/// P2P network behaviour combining multiple protocols
#[derive(NetworkBehaviour)]
pub struct MeshBehaviour {
    /// Kademlia DHT for peer discovery
    pub kademlia: kad::Behaviour<MemoryStore>,
    /// Relay client for NAT traversal. cfg-gated for A-4b; see
    /// `Cargo.toml` `[features]` block for the default-off `relay`
    /// sub-feature, and `crates/arniko/Cargo.toml` for the matching arniko
    /// opt-in (`arniko --features networking,relay`).
    #[cfg(feature = "relay")]
    pub relay: relay::client::Behaviour,
    /// Identify protocol
    pub identify: libp2p::identify::Behaviour,
    /// Gossipsub for pub/sub (optional but recommended)
    pub gossipsub: libp2p::gossipsub::Behaviour,
    /// Request-response for point-to-point messaging (implements SendMessage)
    pub request_response: cbor::Behaviour<MeshRequest, MeshResponse>,
}

/// P2P transport layer
pub struct P2pTransport {
    /// Our identity
    identity: NodeIdentity,
    /// libp2p swarm command sender
    command_tx: mpsc::Sender<P2pCommand>,
    /// Pending requests waiting for responses (keyed by request ID)
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<Response>>>>,
    /// Pending DHT queries
    pending_dht: Arc<RwLock<HashMap<kad::QueryId, oneshot::Sender<Vec<Vec<u8>>>>>>,
    /// Known peer DIDs -> PeerIds
    peer_map: Arc<RwLock<HashMap<String, PeerId>>>,
    /// Listen addresses
    listeners: Arc<RwLock<Vec<Multiaddr>>>,
    /// Currently-connected peers (distinct PeerIds with ≥1 live connection).
    /// Mirrors the `listeners` shared-state pattern so the connected-peer count
    /// is observable without a swarm round-trip — surfaced by federation status
    /// (FED-04). Maintained in the swarm event loop on
    /// `ConnectionEstablished`/`ConnectionClosed`.
    connected_peers: Arc<RwLock<HashSet<PeerId>>>,
    /// Gossip subscribers
    gossip_tx: broadcast::Sender<(String, Vec<u8>)>,
    /// Pending correlated responses — callers waiting for a response matched by correlation_id
    pending_correlated: Arc<dashmap::DashMap<String, oneshot::Sender<Response>>>,
    /// Per-method handlers for inbound mesh requests. Set via
    /// `register_inbound_handler`; consulted in the swarm event loop on each
    /// incoming `RRMsg::Request`. Methods without a registered handler fall
    /// back to the "method not found" stub. (s198)
    inbound_handlers: Arc<dashmap::DashMap<String, InboundHandler>>,
    /// Pending `ResponseChannel`s for inbound RR requests whose async handler
    /// is still running. The handler task signals completion via
    /// `P2pCommand::SendRRResponse`, which pulls the channel out of this map
    /// and writes the response on the swarm thread. (s198)
    pending_rr_responses: Arc<
        dashmap::DashMap<String, request_response::ResponseChannel<MeshResponse>>,
    >,
}

enum P2pCommand {
    Dial {
        peer_id: PeerId,
        addr: Option<Multiaddr>,
    },
    SendMessage {
        peer_id: PeerId,
        message: MeshMessage,
    },
    /// Send a message with a correlation ID for response tracking
    SendCorrelated {
        peer_id: PeerId,
        message: MeshMessage,
        correlation_id: String,
    },
    FindPeer {
        did: String,
        reply: oneshot::Sender<Option<PeerId>>,
    },
    Subscribe {
        topic: String,
    },
    Publish {
        topic: String,
        message: Vec<u8>,
    },
    DhtPut {
        key: Vec<u8>,
        value: Vec<u8>,
    },
    DhtGet {
        key: Vec<u8>,
        reply: oneshot::Sender<Vec<Vec<u8>>>,
    },
    Bootstrap,
    Shutdown,
    /// Async-handler-task completion signal. Carries the response that should
    /// be sent on the `ResponseChannel` previously parked in
    /// `pending_rr_responses` under `request_id`. (s198)
    SendRRResponse {
        request_id: String,
        response: MeshResponse,
    },
}

impl P2pTransport {
    /// Create and start the P2P transport
    pub async fn start(
        identity: NodeIdentity,
        bootstrap_peers: Vec<Multiaddr>,
    ) -> crate::Result<Self> {
        let keypair = identity.to_libp2p_keypair();
        let peer_id = keypair.public().to_peer_id();

        tracing::info!("Starting P2P node with PeerId: {}", peer_id);

        // Build the swarm
        let swarm = Self::build_swarm(keypair.clone())?;

        let (command_tx, command_rx) = mpsc::channel(256);
        let pending = Arc::new(RwLock::new(HashMap::new()));
        let pending_dht = Arc::new(RwLock::new(HashMap::new()));
        let peer_map = Arc::new(RwLock::new(HashMap::new()));
        let listeners = Arc::new(RwLock::new(Vec::new()));
        let connected_peers = Arc::new(RwLock::new(HashSet::new()));
        let (gossip_tx, _) = broadcast::channel(1024);

        let pending_correlated = Arc::new(dashmap::DashMap::new());
        let inbound_handlers: Arc<dashmap::DashMap<String, InboundHandler>> =
            Arc::new(dashmap::DashMap::new());
        let pending_rr_responses: Arc<
            dashmap::DashMap<String, request_response::ResponseChannel<MeshResponse>>,
        > = Arc::new(dashmap::DashMap::new());

        // Spawn the swarm event loop
        let pending_clone = Arc::clone(&pending);
        let pending_dht_clone = Arc::clone(&pending_dht);
        let peer_map_clone = Arc::clone(&peer_map);
        let listeners_clone = Arc::clone(&listeners);
        let connected_peers_clone = Arc::clone(&connected_peers);
        let gossip_tx_clone = gossip_tx.clone();
        let correlated_clone = Arc::clone(&pending_correlated);
        let inbound_handlers_clone = Arc::clone(&inbound_handlers);
        let pending_rr_responses_clone = Arc::clone(&pending_rr_responses);
        let command_tx_for_handlers = command_tx.clone();

        tokio::spawn(async move {
            Self::run_swarm(
                swarm,
                command_rx,
                pending_clone,
                pending_dht_clone,
                peer_map_clone,
                listeners_clone,
                connected_peers_clone,
                gossip_tx_clone,
                bootstrap_peers,
                correlated_clone,
                inbound_handlers_clone,
                pending_rr_responses_clone,
                command_tx_for_handlers,
            )
            .await;
        });

        Ok(Self {
            identity,
            command_tx,
            pending,
            pending_dht,
            peer_map,
            listeners,
            connected_peers,
            gossip_tx,
            pending_correlated,
            inbound_handlers,
            pending_rr_responses,
        })
    }

    fn build_swarm(keypair: Keypair) -> crate::Result<Swarm<MeshBehaviour>> {
        let peer_id = keypair.public().to_peer_id();

        // Kademlia for DHT.
        // Default mode in libp2p 0.55+ is Client (initiate only, reject inbound kad streams).
        // For mutual DHT participation across processes we need Server mode on both sides,
        // otherwise both peers reject /ipfs/kad/1.0.0 and the routing table never populates.
        let store = MemoryStore::new(peer_id);
        let kademlia_config = kad::Config::default();
        let mut kademlia = kad::Behaviour::with_config(peer_id, store, kademlia_config);
        kademlia.set_mode(Some(kad::Mode::Server));

        // Gossipsub
        let gossipsub_config = libp2p::gossipsub::Config::default();
        let gossipsub = libp2p::gossipsub::Behaviour::new(
            libp2p::gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| MeshError::Transport(format!("Gossipsub init failed: {}", e)))?;

        // Relay client for NAT traversal. The full (transport, behaviour)
        // setup is cfg-gated for A-4b: activating libp2p's `relay` feature
        // transitively pulls `x509-parser` -> `rcgen` -> E0119 blanket-impl
        // conflict with newer `time`. Default arniko networking does NOT
        // enable this; opt-in via `arniko --features networking,relay`.
        #[cfg(feature = "relay")]
        let (relay_transport, relay_behaviour) = {
            let (transport, behaviour) = relay::client::new(peer_id);
            let transport = transport
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(noise::Config::new(&keypair).expect("signing keypair"))
                .multiplex(libp2p::yamux::Config::default());
            (transport, behaviour)
        };

        // Identify protocol - includes observed addresses for NAT traversal
        let identify = libp2p::identify::Behaviour::new(
            libp2p::identify::Config::new("/exo-mesh/1.0.0".to_string(), keypair.public())
                .with_agent_version(format!("exosphere/{}", env!("CARGO_PKG_VERSION"))),
        );

        // Request-response for point-to-point SendMessage implementation
        let rr_proto = [(
            StreamProtocol::new(mesh_proto::MESH_PROTOCOL_ID),
            request_response::ProtocolSupport::Full,
        )];
        let request_response = cbor::Behaviour::new(rr_proto, request_response::Config::default());

        let behaviour = MeshBehaviour {
            kademlia,
            // A-4b cfg-gate: `relay` field only exists when the `relay`
            // sub-feature is enabled (mirrors the `pub relay` field above).
            #[cfg(feature = "relay")]
            relay: relay_behaviour,
            identify,
            gossipsub,
            request_response,
        };

        // Build TCP transport with Noise encryption and Yamux muxing.
        // TCP is always present (the safe base transport; quic and relay are
        // cfg-gated below so libp2p's `quic` path -> rcgen -> E0119 conflict
        // doesn't bite the default `networking` build).
        let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default())
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&keypair).expect("signing keypair"))
            .multiplex(libp2p::yamux::Config::default());

        // A-4b cfg-gate: Box TCP into a uniform `Boxed<(PeerId, Muxer)>`, then
        // stitch cfg-conditional `or_transport(quic)` / `or_transport(relay)`
        // additions via `cfg_if`. The cfg_if macro stitches the chain at
        // compile time, sidestepping libp2p's chained-builder mid-chain `#[cfg]`
        // restriction (which the docs flagged in the A-4b hazard note).
        let transport: libp2p::core::transport::Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)> =
            tcp_transport
                .map(|(peer_id, muxer), _| {
                    (peer_id, libp2p::core::muxing::StreamMuxerBox::new(muxer))
                })
                .boxed();

        // A-4b cfg-gate: extend the bare-TCP transport chain with cfg-gated
        // `or_transport` additions for `quic` and `relay`. Each addition is
        // its own cfg-gated block (the `cfg_if!` macro used in earlier
        // drafts tripped stable Rust's E0658 `stmt_expr_attributes` hedge)
        // and each addition does its own Either → tuple flattening via
        // `.map`, so the cumulative transport stays uniformly
        // `Boxed<(PeerId, StreamMuxerBox)>` regardless of which sub-features
        // are enabled. `quic::tokio::Transport::new` takes a `quic::Config`
        // directly, so we also call `.expect("quic config")` here to unwrap
        // the `Result` returned by `quic::Config::new(&keypair)` in libp2p
        // 0.55 (the pre-A-4b vendored form dropped the `.expect`, which
        // produced a distinct E0308 type mismatch against the expected
        // `(PeerId, StreamMuxerBox)` output downstream).
        let mut transport: libp2p::core::transport::Boxed<(PeerId, libp2p::core::muxing::StreamMuxerBox)> = transport;

        #[cfg(feature = "quic")]
        {
            let quic_boxed = quic::tokio::Transport::new(quic::Config::new(&keypair).expect("quic config"))
                .map(|out, _| match out {
                    Ok((peer_id, muxer)) => (peer_id, libp2p::core::muxing::StreamMuxerBox::new(muxer)),
                    Err(_) => unreachable!("quic transport error is fatal"),
                })
                .boxed();
            transport = transport
                .or_transport(quic_boxed)
                .map(|either, _| match either {
                    futures::future::Either::Left((peer_id, muxer)) => (peer_id, muxer),
                    futures::future::Either::Right((peer_id, muxer)) => (peer_id, muxer),
                })
                .boxed();
        }

        #[cfg(feature = "relay")]
        {
            let relay_boxed = relay_transport
                .map(|out, _| match out {
                    Ok((peer_id, muxer)) => (peer_id, libp2p::core::muxing::StreamMuxerBox::new(muxer)),
                    Err(_) => unreachable!("relay transport error is fatal"),
                })
                .boxed();
            transport = transport
                .or_transport(relay_boxed)
                .map(|either, _| match either {
                    futures::future::Either::Left((peer_id, muxer)) => (peer_id, muxer),
                    futures::future::Either::Right((peer_id, muxer)) => (peer_id, muxer),
                })
                .boxed();
        }

        // A-4b cfg-gate follow-up: `transport` is already uniformly
        // `Boxed<(PeerId, StreamMuxerBox)>` after the cfg-conditional blocks
        // above — pass it directly to `Swarm::new` (previously there was a
        // second `.map(|either, _| match Either::Left/Right { ... }).boxed()`
        // Either-flattening pass here, which produced an E0308 against the
        // already-uniform tuple when neither quic nor relay was enabled,
        // because the closure expected an `Either` but the input was a
        // tuple).
        let swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor()
                .with_idle_connection_timeout(Duration::from_secs(60)),
        );

        Ok(swarm)
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_swarm(
        mut swarm: Swarm<MeshBehaviour>,
        mut command_rx: mpsc::Receiver<P2pCommand>,
        pending: Arc<RwLock<HashMap<String, oneshot::Sender<Response>>>>,
        pending_dht: Arc<RwLock<HashMap<kad::QueryId, oneshot::Sender<Vec<Vec<u8>>>>>>,
        peer_map: Arc<RwLock<HashMap<String, PeerId>>>,
        listeners: Arc<RwLock<Vec<Multiaddr>>>,
        connected_peers: Arc<RwLock<HashSet<PeerId>>>,
        gossip_tx: broadcast::Sender<(String, Vec<u8>)>,
        bootstrap_peers: Vec<Multiaddr>,
        pending_correlated: Arc<dashmap::DashMap<String, oneshot::Sender<Response>>>,
        inbound_handlers: Arc<dashmap::DashMap<String, InboundHandler>>,
        pending_rr_responses: Arc<
            dashmap::DashMap<String, request_response::ResponseChannel<MeshResponse>>,
        >,
        command_tx_for_handlers: mpsc::Sender<P2pCommand>,
    ) {
        // Listen on QUIC (for external peers) — unless TCP-only mode is set.
        // MESH_TCP_ONLY restricts the node to TCP listeners so peers only
        // advertise + dial TCP addresses. This avoids the multi-transport
        // (TCP+QUIC) dial churn that destabilizes connections over proot's
        // network emulation (rapid connect/disconnect / ApplicationClosed /
        // BrokenPipe cycles). The QUIC transport stays in the stack but is
        // simply not listened on. (s198 cross-host experiment.)
        let tcp_only = std::env::var("MESH_TCP_ONLY")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if tcp_only {
            tracing::info!("MESH_TCP_ONLY set — skipping QUIC listener (TCP-only mode)");
        } else {
            // A-4b cfg-gate: only register the QUIC listener when the `quic`
            // sub-feature is enabled. Without this the multiaddr `/quic-v1`
            // pattern is still valid (libp2p semantics) but the listener
            // setup would dangle because the quic transport is a no-op when
            // the feature is off (see the transport-chain rebuild above).
            #[cfg(feature = "quic")]
            {
                let listen_quic: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap();
                if let Err(e) = swarm.listen_on(listen_quic) {
                    tracing::error!("Failed to listen on QUIC: {}", e);
                }
            }
            #[cfg(not(feature = "quic"))]
            tracing::info!("QUIC listener skipped (exo-mesh `quic` sub-feature off; see A-4b cfg-gate)");
        }

        // Listen on TCP localhost (for same-machine peers) — always ephemeral.
        let listen_tcp: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().unwrap();
        if let Err(e) = swarm.listen_on(listen_tcp) {
            tracing::error!("Failed to listen on TCP localhost: {}", e);
        }

        // Also listen on the any-interface TCP listener for cross-host discovery.
        // MESH_TCP_PORT pins this externally-dialable listener to a fixed port so
        // a node's external multiaddr is stable across restarts (its PeerId is
        // already stable via the identity keyfile). Default 0 = OS-assigned
        // ephemeral port (prior behavior). Pin it on nodes that must be reachable
        // at a known address after a supervisor restart — e.g. the phone-claude
        // durable peer, where re-discovering an ephemeral port per restart isn't
        // viable over a churning NAT/proot link. (s199)
        let tcp_port: u16 = std::env::var("MESH_TCP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        if tcp_port != 0 {
            tracing::info!("MESH_TCP_PORT={tcp_port} — pinning external TCP listener");
        }
        let listen_tcp_any: Multiaddr =
            format!("/ip4/0.0.0.0/tcp/{tcp_port}").parse().unwrap();
        if let Err(e) = swarm.listen_on(listen_tcp_any) {
            tracing::error!("Failed to listen on TCP: {}", e);
        }

        // Bootstrap to known peers
        for addr in bootstrap_peers {
            let addr_str = addr.to_string();
            // Skip /p2p/ addresses - they're not dialable directly
            if addr_str.starts_with("/p2p/") {
                tracing::warn!("Skipping non-dialable bootstrap peer: {}", addr_str);
                continue;
            }
            if let Some(peer_id) = extract_peer_id(&addr) {
                swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
                if let Err(e) = swarm.dial(addr) {
                    tracing::warn!("Failed to dial bootstrap peer: {}", e);
                }
            }
        }

        // Start DHT bootstrap
        if let Err(e) = swarm.behaviour_mut().kademlia.bootstrap() {
            tracing::debug!("Kademlia bootstrap: {:?}", e);
        }

        loop {
            tokio::select! {
                event = swarm.select_next_some() => {
                    Self::handle_swarm_event(event, &mut swarm, &pending, &pending_dht, &peer_map, &listeners, &connected_peers, &gossip_tx, &pending_correlated, &inbound_handlers, &pending_rr_responses, &command_tx_for_handlers).await;
                }
                Some(cmd) = command_rx.recv() => {
                    match cmd {
                        P2pCommand::Dial { peer_id, addr } => {
                            if let Some(addr) = addr {
                                swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                            }
                            if let Err(e) = swarm.dial(peer_id) {
                                tracing::error!("Dial failed: {}", e);
                            }
                        }
                        P2pCommand::SendMessage { peer_id, message } => {
                            let (method, params) = message_method_and_params(&message);
                            let req = MeshRequest {
                                id: uuid::Uuid::new_v4().to_string(),
                                method,
                                params,
                                caller_did: String::new(),
                                correlation_id: None,
                            };
                            let _req_id = swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, req);
                            tracing::debug!("Sent request-response message to {}", peer_id);
                        }
                        P2pCommand::SendCorrelated { peer_id, message, correlation_id } => {
                            let (method, params) = message_method_and_params(&message);
                            let req = MeshRequest {
                                id: uuid::Uuid::new_v4().to_string(),
                                method,
                                params,
                                caller_did: String::new(),
                                correlation_id: Some(correlation_id),
                            };
                            let _req_id = swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, req);
                            tracing::debug!("Sent correlated request-response to {}", peer_id);
                        }
                        P2pCommand::FindPeer { did, reply } => {
                            let peer_id = peer_map.read().await.get(&did).cloned();
                            let _ = reply.send(peer_id);
                        }
                        P2pCommand::Subscribe { topic } => {
                            let topic = libp2p::gossipsub::IdentTopic::new(topic);
                            if let Err(e) = swarm.behaviour_mut().gossipsub.subscribe(&topic) {
                                tracing::error!("Failed to subscribe: {:?}", e);
                            }
                        }
                        P2pCommand::Publish { topic, message } => {
                            let topic = libp2p::gossipsub::IdentTopic::new(topic);
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, message) {
                                tracing::error!("Failed to publish: {:?}", e);
                            }
                        }
                        P2pCommand::DhtPut { key, value } => {
                            let record = kad::Record {
                                key: kad::RecordKey::new(&key),
                                value,
                                publisher: None,
                                expires: None,
                            };
                            match swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One) {
                                Ok(query_id) => {
                                    tracing::debug!("DHT put initiated, query_id: {:?}", query_id);
                                }
                                Err(e) => {
                                    tracing::error!("Failed to put record: {:?}", e);
                                }
                            }
                        }
                        P2pCommand::DhtGet { key, reply } => {
                            tracing::debug!("DHT get for key: {:?}", String::from_utf8_lossy(&key));
                            let query_id = swarm.behaviour_mut().kademlia.get_record(kad::RecordKey::new(&key));
                            tracing::debug!("DHT get query_id: {:?}", query_id);
                            pending_dht.write().await.insert(query_id, reply);
                        }
                        P2pCommand::Bootstrap => {
                            let _ = swarm.behaviour_mut().kademlia.bootstrap();
                        }
                        P2pCommand::Shutdown => {
                            tracing::info!("P2P transport shutting down");
                            break;
                        }
                        P2pCommand::SendRRResponse { request_id, response } => {
                            if let Some((_, channel)) = pending_rr_responses.remove(&request_id) {
                                if swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(channel, response)
                                    .is_err()
                                {
                                    tracing::warn!(
                                        "send_response failed for inbound handler request {}",
                                        request_id
                                    );
                                }
                            } else {
                                tracing::warn!(
                                    "SendRRResponse: no parked channel for request {}",
                                    request_id
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_swarm_event(
        event: SwarmEvent<MeshBehaviourEvent>,
        swarm: &mut Swarm<MeshBehaviour>,
        _pending: &RwLock<HashMap<String, oneshot::Sender<Response>>>,
        pending_dht: &RwLock<HashMap<kad::QueryId, oneshot::Sender<Vec<Vec<u8>>>>>,
        _peer_map: &RwLock<HashMap<String, PeerId>>,
        listeners: &RwLock<Vec<Multiaddr>>,
        connected_peers: &RwLock<HashSet<PeerId>>,
        gossip_tx: &broadcast::Sender<(String, Vec<u8>)>,
        pending_correlated: &dashmap::DashMap<String, oneshot::Sender<Response>>,
        inbound_handlers: &Arc<dashmap::DashMap<String, InboundHandler>>,
        pending_rr_responses: &Arc<
            dashmap::DashMap<String, request_response::ResponseChannel<MeshResponse>>,
        >,
        command_tx_for_handlers: &mpsc::Sender<P2pCommand>,
    ) {
        match event {
            SwarmEvent::Behaviour(MeshBehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed { id, result, .. },
            )) => {
                // Check if this query ID is in our pending list
                let mut pending = pending_dht.write().await;
                if let Some(reply) = pending.remove(&id) {
                    match result {
                        kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(
                            kad::PeerRecord {
                                record: kad::Record { value, .. },
                                ..
                            },
                        ))) => {
                            tracing::debug!("DHT get found record, size: {}", value.len());
                            let _ = reply.send(vec![value]);
                        }
                        kad::QueryResult::GetRecord(Ok(
                            kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. },
                        )) => {
                            tracing::debug!("DHT get finished with no records");
                            let _ = reply.send(vec![]);
                        }
                        kad::QueryResult::GetRecord(Err(e)) => {
                            tracing::debug!("DHT get error: {:?}", e);
                            let _ = reply.send(vec![]);
                        }
                        other => {
                            tracing::debug!("DHT get other result: {:?}", other);
                            let _ = reply.send(vec![]);
                        }
                    }
                }
            }
            // Handle DHT put results
            SwarmEvent::Behaviour(MeshBehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result: kad::QueryResult::PutRecord(Ok(result)),
                    ..
                },
            )) => {
                tracing::debug!("DHT put successful: {:?}", result);
            }
            SwarmEvent::Behaviour(MeshBehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed {
                    result: kad::QueryResult::PutRecord(Err(e)),
                    ..
                },
            )) => {
                tracing::debug!("DHT put failed: {:?}", e);
            }
            SwarmEvent::Behaviour(MeshBehaviourEvent::Gossipsub(
                libp2p::gossipsub::Event::Message {
                    propagation_source: _,
                    message_id: _,
                    message,
                },
            )) => {
                let topic = message.topic.into_string();
                let params = (topic, message.data);
                let _ = gossip_tx.send(params);
            }
            SwarmEvent::NewListenAddr { address, .. } => {
                tracing::info!("Listening on {}", address);
                listeners.write().await.push(address);
            }
            SwarmEvent::ConnectionEstablished {
                peer_id,
                connection_id,
                ..
            } => {
                tracing::info!("Connected to {} on {}", peer_id, connection_id);

                // FED-04: track distinct connected peers for federation status.
                connected_peers.write().await.insert(peer_id);

                // NOTE: we deliberately do NOT add our OWN listener addresses to
                // Kademlia under the remote `peer_id` here. Doing so (a former
                // "same-machine connectivity" hack) poisons the routing table:
                // it tells Kademlia the remote peer is reachable at THIS node's
                // addresses, so subsequent Kademlia-driven dials target our own
                // listeners — producing self/duplicate connections that the muxer
                // immediately tears down (`Right(Closed)` / ECONNRESET 104). In a
                // ≥3-node topology, Kademlia's active bucket-refresh lookups retry
                // those poisoned addresses indefinitely → a connection-hold flap
                // (counters into the 100s–700s/min). Empirically this removal takes
                // a 3-node loopback mesh from ~75 resets/node/75s to 0 and a stable
                // full triangle. A peer's REAL dialable addresses are learned the
                // correct way from the Identify handler below (`info.listen_addrs`).
                // (s257 flap fix — s256 stress finding #2, churn half)
                //
                // SEPARATE OPEN BUG (s256 finding #2, DHT half): Kademlia routing
                // tables never populate (no RoutingUpdated fires even after Identify),
                // so DHT `put_record` returns InsufficientPeers — DHT-based discovery
                // is non-functional and the mesh relies on the direct call_agent
                // fast-path. Tracked separately; not addressed by this churn fix.
            }
            SwarmEvent::ConnectionClosed {
                peer_id,
                cause,
                num_established,
                ..
            } => {
                tracing::info!("Disconnected from {}: {:?}", peer_id, cause);

                // FED-04: only drop the peer from the connected set once its
                // last connection closes (libp2p may hold several per peer).
                if num_established == 0 {
                    connected_peers.write().await.remove(&peer_id);
                }
            }
            SwarmEvent::Behaviour(MeshBehaviourEvent::Identify(
                libp2p::identify::Event::Received { peer_id, info, .. },
            )) => {
                tracing::debug!("Identified peer {} running {}", peer_id, info.agent_version);

                // Add addresses to Kademlia, filtering out non-dialable ones
                for addr in info.listen_addrs {
                    let addr_str = addr.to_string();
                    // Skip /p2p/ addresses - they're not dialable directly
                    // Instead, try to construct a localhost address for same-machine connectivity
                    if addr_str.starts_with("/p2p/") {
                        // For /p2p/PeerId addresses, we can't dial directly
                        // This typically happens with relayed connections
                        tracing::debug!("Skipping non-dialable /p2p/ address for {}", peer_id);
                        continue;
                    }
                    // In TCP-only mode, ignore QUIC (udp/quic) addresses so the
                    // dialer never tries the QUIC transport — avoids multi-
                    // transport dial churn over proot. (s198 experiment.)
                    let tcp_only = std::env::var("MESH_TCP_ONLY")
                        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                        .unwrap_or(false);
                    if tcp_only && (addr_str.contains("/udp/") || addr_str.contains("/quic")) {
                        continue;
                    }
                    // Only add addresses we can actually dial (ip4/ip6 tcp, udp, quic)
                    if addr_str.contains("/ip4/") || addr_str.contains("/ip6/") {
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                    }
                }
            }
            SwarmEvent::Behaviour(MeshBehaviourEvent::Kademlia(event)) => match event {
                kad::Event::RoutingUpdated { peer, .. } => {
                    tracing::debug!("Kademlia routing updated for {}", peer);
                }
                kad::Event::OutboundQueryProgressed { result, .. } => {
                    tracing::debug!("Kademlia query progress: {:?}", result);
                }
                _ => {}
            },
            SwarmEvent::Behaviour(MeshBehaviourEvent::RequestResponse(event)) => {
                use request_response::Event as RREvent;
                match event {
                    RREvent::Message { peer, message, .. } => {
                        use request_response::Message as RRMsg;
                        match message {
                            RRMsg::Request {
                                request, channel, ..
                            } => {
                                tracing::debug!(
                                    "Received mesh request from {}: method={}",
                                    peer,
                                    request.method
                                );
                                // Look up an inbound handler registered for this method.
                                // If found, park the ResponseChannel and spawn the async
                                // handler; on completion it signals via P2pCommand::SendRRResponse
                                // and the event loop above sends the response on the channel.
                                // If no handler is registered, fall back to the legacy
                                // "method received" stub so existing mesh-message paths keep working. (s198)
                                let handler = inbound_handlers
                                    .get(&request.method)
                                    .map(|h| Arc::clone(h.value()));
                                if let Some(handler) = handler {
                                    // `parking_key` (the RR request's own id) keys the
                                    // ResponseChannel so SendRRResponse can recover it.
                                    // `response_id` is what the *sender* correlates on:
                                    // its `send_and_wait` parked a oneshot under the
                                    // correlation_id, and the RR Response branch matches
                                    // by `response.id`. Echo correlation_id (falling back
                                    // to the request id when absent) so the round-trip
                                    // actually resolves. (s198)
                                    let parking_key = request.id.clone();
                                    let response_id = request
                                        .correlation_id
                                        .clone()
                                        .unwrap_or_else(|| request.id.clone());
                                    pending_rr_responses
                                        .insert(parking_key.clone(), channel);
                                    let params = request.params.clone();
                                    let command_tx = command_tx_for_handlers.clone();
                                    tokio::spawn(async move {
                                        let result = handler(params).await;
                                        let response = MeshResponse {
                                            id: response_id,
                                            result,
                                        };
                                        if let Err(e) = command_tx
                                            .send(P2pCommand::SendRRResponse {
                                                request_id: parking_key,
                                                response,
                                            })
                                            .await
                                        {
                                            tracing::warn!(
                                                "Failed to signal inbound handler completion: {}",
                                                e
                                            );
                                        }
                                    });
                                } else {
                                    let response = MeshResponse {
                                        id: request.id.clone(),
                                        result: serde_json::json!({ "status": "received" }),
                                    };
                                    let _ = swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_response(channel, response);
                                }
                            }
                            RRMsg::Response {
                                request_id,
                                response,
                            } => {
                                tracing::debug!(
                                    "Received response for request {:?}: {}",
                                    request_id,
                                    response.id
                                );
                                // Dispatch correlated response to waiting caller
                                if let Some((_, sender)) = pending_correlated.remove(&response.id) {
                                    let mesh_response = Response::success(
                                        response.id.clone(),
                                        response.result.clone(),
                                    );
                                    let _ = sender.send(mesh_response);
                                    tracing::debug!(
                                        "Dispatched correlated response for {}",
                                        response.id
                                    );
                                }
                            }
                        }
                    }
                    RREvent::OutboundFailure { peer, error, .. } => {
                        tracing::warn!("Outbound mesh request to {} failed: {:?}", peer, error);
                    }
                    RREvent::InboundFailure { peer, error, .. } => {
                        tracing::warn!("Inbound mesh request from {} failed: {:?}", peer, error);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Register an async handler for inbound mesh requests on the given method
    /// name. Methods without a registered handler get the legacy
    /// `{"status": "received"}` stub reply. Re-registering a method silently
    /// replaces the prior handler. (s198)
    pub fn register_inbound_handler<F, Fut>(&self, method: impl Into<String>, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = serde_json::Value> + Send + 'static,
    {
        let boxed: InboundHandler = Arc::new(move |params| Box::pin(handler(params)));
        self.inbound_handlers.insert(method.into(), boxed);
    }

    /// Send a request to a remote peer
    pub async fn send(
        &self,
        node_did: &str,
        _service_id: &ServiceId,
        request: Request,
    ) -> crate::Result<Response> {
        // Find peer ID from DID
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2pCommand::FindPeer {
                did: node_did.to_string(),
                reply: tx,
            })
            .await
            .map_err(|_| MeshError::NotReady)?;

        let peer_id = rx
            .await
            .map_err(|_| MeshError::NotReady)?
            .ok_or_else(|| MeshError::ServiceNotFound(format!("Peer {} not found", node_did)))?;

        // Store pending request
        let (response_tx, response_rx) = oneshot::channel();
        {
            let mut pending = self.pending.write().await;
            pending.insert(request.id.clone(), response_tx);
        }

        // Send the message
        let message = MeshMessage::Request(request.clone());
        self.command_tx
            .send(P2pCommand::SendMessage { peer_id, message })
            .await
            .map_err(|_| MeshError::NotReady)?;

        // Wait for response with timeout
        tokio::time::timeout(Duration::from_secs(30), response_rx)
            .await
            .map_err(|_| MeshError::Timeout)?
            .map_err(|_| MeshError::Transport("Response channel closed".to_string()))
    }

    /// Dial a peer at the given address.
    ///
    /// Non-fatal — logs errors but does not panic. The peer may already be connected.
    pub async fn dial(&self, peer_id: PeerId, addr: Option<Multiaddr>) -> crate::Result<()> {
        self.command_tx
            .send(P2pCommand::Dial { peer_id, addr })
            .await
            .map_err(|_| MeshError::NotReady)
    }

    /// Send a mesh message to a peer by PeerId.
    ///
    /// Returns immediate acknowledgment with a correlation_id for response tracking.
    pub async fn send_to_peer(
        &self,
        peer_id: PeerId,
        message: MeshMessage,
    ) -> crate::Result<crate::message::Response> {
        let correlation_id = uuid::Uuid::new_v4().to_string();

        self.command_tx
            .send(P2pCommand::SendMessage { peer_id, message })
            .await
            .map_err(|_| MeshError::NotReady)?;

        Ok(crate::message::Response::success(
            String::new(),
            serde_json::json!({
                "status": "sent",
                "correlation_id": correlation_id,
            }),
        ))
    }

    /// Send a mesh message and wait for the peer's response.
    ///
    /// Generates a correlation ID, dispatches the message, and blocks until
    /// the peer responds or the timeout (30s default) expires.
    pub async fn send_and_wait(
        &self,
        peer_id: PeerId,
        message: MeshMessage,
        timeout_secs: Option<u64>,
    ) -> crate::Result<Response> {
        let correlation_id = uuid::Uuid::new_v4().to_string();
        let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(30));

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();

        // Insert into pending map BEFORE sending
        self.pending_correlated.insert(correlation_id.clone(), tx);

        // Send the correlated message
        if let Err(e) = self
            .command_tx
            .send(P2pCommand::SendCorrelated {
                peer_id,
                message,
                correlation_id: correlation_id.clone(),
            })
            .await
        {
            // Clean up pending entry on send failure
            self.pending_correlated.remove(&correlation_id);
            return Err(MeshError::NotReady);
        }

        tracing::debug!(
            "Waiting for correlated response {} (timeout: {}s)",
            correlation_id,
            timeout_duration.as_secs()
        );

        // Wait for response with timeout
        match tokio::time::timeout(timeout_duration, rx).await {
            Ok(Ok(response)) => {
                tracing::debug!("Received correlated response for {}", correlation_id);
                Ok(response)
            }
            Ok(Err(_)) => {
                // Channel was dropped without sending — peer disconnected or internal error
                self.pending_correlated.remove(&correlation_id);
                Err(MeshError::Transport("Response channel closed".to_string()))
            }
            Err(_) => {
                // Timeout — clean up pending entry
                self.pending_correlated.remove(&correlation_id);
                tracing::warn!(
                    "Correlated request {} timed out after {}s",
                    correlation_id,
                    timeout_duration.as_secs()
                );
                Err(MeshError::Timeout)
            }
        }
    }

    /// Get our node's DID
    pub fn did(&self) -> &str {
        self.identity.did()
    }

    /// Shutdown the P2P transport
    pub async fn shutdown(&self) {
        let _ = self.command_tx.send(P2pCommand::Shutdown).await;
    }

    /// Subscribe to a gossip topic
    pub async fn subscribe(
        &self,
        topic: String,
    ) -> crate::Result<broadcast::Receiver<(String, Vec<u8>)>> {
        self.command_tx
            .send(P2pCommand::Subscribe {
                topic: topic.clone(),
            })
            .await
            .map_err(|_| MeshError::NotReady)?;
        Ok(self.gossip_tx.subscribe())
    }

    /// Publish to a gossip topic
    pub async fn publish(&self, topic: String, message: Vec<u8>) -> crate::Result<()> {
        self.command_tx
            .send(P2pCommand::Publish { topic, message })
            .await
            .map_err(|_| MeshError::NotReady)?;
        Ok(())
    }

    /// Trigger DHT bootstrap to connect to existing peers
    pub async fn bootstrap(&self) -> crate::Result<()> {
        self.command_tx
            .send(P2pCommand::Bootstrap)
            .await
            .map_err(|_| MeshError::NotReady)?;
        Ok(())
    }

    /// Put data into DHT
    pub async fn dht_put(&self, key: Vec<u8>, value: Vec<u8>) -> crate::Result<()> {
        self.command_tx
            .send(P2pCommand::DhtPut { key, value })
            .await
            .map_err(|_| MeshError::NotReady)?;
        Ok(())
    }

    /// Get data from DHT
    pub async fn dht_get(&self, key: Vec<u8>) -> crate::Result<Vec<Vec<u8>>> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(P2pCommand::DhtGet { key, reply: tx })
            .await
            .map_err(|_| MeshError::NotReady)?;

        // Wait for response with timeout
        tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| MeshError::Timeout)?
            .map_err(|_| MeshError::Transport("DHT channel closed".to_string()))
    }

    /// Get listening addresses
    pub async fn listeners(&self) -> Vec<Multiaddr> {
        self.listeners.read().await.clone()
    }

    /// Count of currently-connected mesh peers (FED-04 federation status).
    pub async fn connected_peer_count(&self) -> usize {
        self.connected_peers.read().await.len()
    }

    /// Whether we already hold a live connection to `peer_id`. Used to skip a
    /// redundant explicit dial (which races the existing connection and, over
    /// proot's seccomp/ptrace network layer, churns it — the s198 failure mode).
    pub async fn is_connected(&self, peer_id: &PeerId) -> bool {
        self.connected_peers.read().await.contains(peer_id)
    }
}

fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|p| {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = p {
            Some(peer_id)
        } else {
            None
        }
    })
}

/// Bootstrap peers for the Exosphere network
pub mod bootstrap {
    use libp2p::Multiaddr;

    /// Get default bootstrap peers
    ///
    /// In production, these would be well-known community nodes
    pub fn default_peers() -> Vec<Multiaddr> {
        vec![
            // Example bootstrap peers - in production these would be actual stable nodes
            // For development, we use localhost addresses
            "/ip4/127.0.0.1/udp/4001/quic-v1".parse().unwrap(),
            // Additional bootstrap peers can be added here:
            // "/ip4/1.2.3.4/udp/4001/quic-v1/p2p/12D3KooWExamplePeerId".parse().unwrap(),
            // "/dns4/bootstrap.exosphere.dev/tcp/4001/quic-v1/p2p/12D3KooWExamplePeerId2".parse().unwrap(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_peer_id() {
        let addr: Multiaddr = "/ip4/127.0.0.1/udp/4001/quic-v1".parse().unwrap();
        assert!(extract_peer_id(&addr).is_none());
    }
}

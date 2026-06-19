//! Transport abstraction layer
//!
//! Provides a unified interface over local, LAN, and P2P transports.

use crate::address::{LocationHint, ServiceAddress};
use crate::local::LocalClient;
use crate::message::{Request, Response};
use crate::{MeshError, Result};
use std::sync::Arc;

use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

/// Unified transport that routes to appropriate backend
pub struct Transport {
    #[cfg(feature = "p2p")]
    #[cfg(feature = "p2p")]
    p2p: Option<Arc<crate::p2p::P2pTransport>>,
    pool: Arc<ConnectionPool>,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "p2p")]
            #[cfg(feature = "p2p")]
            p2p: None,
            pool: Arc::new(ConnectionPool::new(10)),
        }
    }

    #[cfg(feature = "p2p")]
    pub fn with_p2p(mut self, p2p: Arc<crate::p2p::P2pTransport>) -> Self {
        self.p2p = Some(p2p);
        self
    }

    /// Send a request to a service
    pub async fn send(&self, address: &ServiceAddress, request: Request) -> Result<Response> {
        match address.location {
            LocationHint::Local => self.send_local(address, request).await,
            LocationHint::Lan => {
                // For LAN, we still use local transport if on same machine,
                // otherwise fall through to P2P
                if address.node_did.is_none() {
                    self.send_local(address, request).await
                } else {
                    self.send_remote(address, request).await
                }
            }
            LocationHint::Remote => self.send_remote(address, request).await,
            LocationHint::Unknown => {
                // Try local first, then remote
                match self.send_local(address, request.clone()).await {
                    Ok(resp) => Ok(resp),
                    Err(_) => self.send_remote(address, request).await,
                }
            }
        }
    }

    async fn send_local(&self, address: &ServiceAddress, request: Request) -> Result<Response> {
        let socket_path = address.local_socket_path();
        let mut client_guard = self.pool.get_local(&socket_path).await?;
        client_guard.call(request).await
    }

    async fn send_remote(&self, address: &ServiceAddress, request: Request) -> Result<Response> {
        #[cfg(feature = "p2p")]
        {
            if let Some(p2p) = &self.p2p {
                let node_did = address.node_did.as_ref().ok_or_else(|| {
                    MeshError::ConnectionFailed("No node DID for remote service".to_string())
                })?;

                return p2p.send(node_did, &address.service_id, request).await;
            }
        }

        Err(MeshError::ConnectionFailed(
            "P2P transport not available".to_string(),
        ))
    }

    #[cfg(feature = "p2p")]
    pub async fn subscribe(&self, topic: String) -> Result<broadcast::Receiver<(String, Vec<u8>)>> {
        if let Some(p2p) = &self.p2p {
            p2p.subscribe(topic).await
        } else {
            Err(MeshError::ConnectionFailed(
                "P2P transport not available".to_string(),
            ))
        }
    }

    #[cfg(feature = "p2p")]
    pub async fn publish(&self, topic: String, message: Vec<u8>) -> Result<()> {
        if let Some(p2p) = &self.p2p {
            p2p.publish(topic, message).await
        } else {
            Err(MeshError::ConnectionFailed(
                "P2P transport not available".to_string(),
            ))
        }
    }

    #[cfg(feature = "p2p")]
    pub async fn dht_put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        if let Some(p2p) = &self.p2p {
            p2p.dht_put(key, value).await
        } else {
            Err(MeshError::ConnectionFailed(
                "P2P transport not available".to_string(),
            ))
        }
    }

    #[cfg(feature = "p2p")]
    pub async fn dht_get(&self, key: Vec<u8>) -> Result<Vec<Vec<u8>>> {
        if let Some(p2p) = &self.p2p {
            p2p.dht_get(key).await
        } else {
            Err(MeshError::ConnectionFailed(
                "P2P transport not available".to_string(),
            ))
        }
    }

    #[cfg(feature = "p2p")]
    pub async fn listeners(&self) -> Vec<String> {
        if let Some(p2p) = &self.p2p {
            p2p.listeners()
                .await
                .iter()
                .map(|a| a.to_string())
                .collect()
        } else {
            vec![]
        }
    }

    /// Count of currently-connected mesh peers (FED-04). Zero when the P2P
    /// transport is not attached.
    #[cfg(feature = "p2p")]
    pub async fn connected_peer_count(&self) -> usize {
        if let Some(p2p) = &self.p2p {
            p2p.connected_peer_count().await
        } else {
            0
        }
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::new()
    }
}

/// Connection pool for reusing connections

/// A pooled client with last used timestamp
struct PooledClient {
    client: LocalClient,
    last_used: std::time::Instant,
    is_active: bool,
}

pub struct ConnectionPool {
    local_clients: RwLock<std::collections::HashMap<String, PooledClient>>,
    max_idle: usize,
    keepalive_interval: Duration,
}

impl ConnectionPool {
    pub fn new(max_idle: usize) -> Self {
        Self {
            local_clients: RwLock::new(std::collections::HashMap::new()),
            max_idle,
            keepalive_interval: Duration::from_secs(30),
        }
    }

    /// Get or create a local client
    pub async fn get_local(self: &Arc<Self>, socket_path: &str) -> Result<PooledClientGuard> {
        let mut clients = self.local_clients.write().await;

        // Try to reuse existing connection
        if let Some(pooled) = clients.get_mut(socket_path) {
            if pooled.is_active {
                pooled.last_used = std::time::Instant::now();
                return Ok(PooledClientGuard {
                    socket_path: socket_path.to_string(),
                    pool: self.clone(),
                    is_reused: true,
                });
            } else {
                // Remove stale connection
                clients.remove(socket_path);
            }
        }

        // Check pool size limit
        if clients.len() >= self.max_idle {
            self.cleanup_stale(&mut clients).await;
        }

        // Create new connection
        let client = LocalClient::connect(socket_path).await?;
        let pooled = PooledClient {
            client,
            last_used: std::time::Instant::now(),
            is_active: true,
        };

        clients.insert(socket_path.to_string(), pooled);

        Ok(PooledClientGuard {
            socket_path: socket_path.to_string(),
            pool: self.clone(),
            is_reused: false,
        })
    }

    async fn cleanup_stale(&self, clients: &mut std::collections::HashMap<String, PooledClient>) {
        let now = std::time::Instant::now();
        clients.retain(|_, pooled| now.duration_since(pooled.last_used) < self.keepalive_interval);
    }

    async fn return_client(&self, socket_path: &str, _client: LocalClient) {
        let mut clients = self.local_clients.write().await;
        if let Some(pooled) = clients.get_mut(socket_path) {
            pooled.last_used = std::time::Instant::now();
            pooled.is_active = true;
        }
    }

    async fn mark_inactive(&self, socket_path: &str) {
        let mut clients = self.local_clients.write().await;
        if let Some(pooled) = clients.get_mut(socket_path) {
            pooled.is_active = false;
        }
    }
}

/// Guard that manages the client's lifecycle in the pool
pub struct PooledClientGuard {
    socket_path: String,
    pool: Arc<ConnectionPool>,
    is_reused: bool,
}

impl PooledClientGuard {
    pub async fn call(&mut self, request: Request) -> Result<Response> {
        let mut clients = self.pool.local_clients.write().await;
        if let Some(pooled) = clients.get_mut(&self.socket_path) {
            pooled.client.call(request).await
        } else {
            Err(MeshError::ConnectionFailed(
                "Client not found in pool".to_string(),
            ))
        }
    }
}

impl Drop for PooledClientGuard {
    fn drop(&mut self) {
        let socket_path = self.socket_path.clone();
        let pool = self.pool.clone();

        // Return client to pool or mark as inactive based on connection health
        tokio::spawn(async move {
            // In a real implementation, we'd check connection health here
            // For now, we assume connections are healthy and return them
            pool.mark_inactive(&socket_path).await;
        });
    }
}

/// Service endpoint for easy service hosting
pub struct ServiceEndpoint {
    pub service_id: crate::address::ServiceId,
    pub methods: Vec<String>,
    pub version: String,
}

impl ServiceEndpoint {
    pub fn new(service_id: crate::address::ServiceId) -> Self {
        Self {
            service_id,
            methods: Vec::new(),
            version: "0.1.0".to_string(),
        }
    }

    pub fn method(mut self, name: impl Into<String>) -> Self {
        self.methods.push(name.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Get the socket path for this endpoint
    pub fn socket_path(&self) -> String {
        format!("/run/exosphere/{}.sock", self.service_id)
    }

    /// Start serving with the given handler
    pub async fn serve<F, Fut>(self, handler: F) -> Result<()>
    where
        F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send,
    {
        let server = crate::local::LocalServer::bind(&self.socket_path()).await?;
        server.serve(handler).await
    }
}

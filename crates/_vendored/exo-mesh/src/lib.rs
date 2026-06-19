//! Exosphere P2P Service Mesh
//!
//! Zero-configuration networking for capsules. Address services by DID or capsule ID,
//! not by IP addresses or ports.
//!
//! # Layers
//!
//! 1. **Local** - Unix sockets for same-machine IPC
//! 2. **LAN** - mDNS discovery for local network
//! 3. **P2P** - libp2p + DHT for internet-wide connectivity
//!
//! # Addressing
//!
//! Services are addressed by:
//! - Capsule ID: `capsule://system.wallet`
//! - DID: `did:key:z6MkhaXgBZD...`
//!
//! The mesh automatically resolves these to the appropriate transport.

pub mod address;
pub mod address_bridge;
pub mod agent_registry;
pub mod identity;
pub mod local;
pub mod message;
pub mod node;
pub mod registry;
pub mod transport;

#[cfg(feature = "p2p")]
pub mod p2p;

#[cfg(feature = "lan")]
pub mod lan;

pub use address::{ServiceAddress, ServiceId};
pub use address_bridge::AddressBridgeError;
pub use agent_registry::AgentRecord;
pub use identity::NodeIdentity;
pub use message::{Request, Response};
pub use node::{MeshConfig, MeshNode};
pub use registry::ServiceRegistry;
// RFC-0012-v2 name-claim wire type (EXO-173) — re-exported so mesh consumers
// publish/resolve names without a direct mesh-proto dependency.
pub use mesh_proto::{name_alias_key, verify_chain, NameClaim};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[cfg(feature = "p2p")]
    #[error("Connection denied: {0}")]
    ConnectionDenied(#[from] libp2p::swarm::ConnectionDenied),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Capability denied: {0}")]
    CapabilityDenied(String),

    #[error("Timeout")]
    Timeout,

    #[error("Node not ready")]
    NotReady,
}

pub type Result<T> = std::result::Result<T, MeshError>;

/// Re-export for convenience
pub mod prelude {
    pub use crate::{
        MeshConfig, MeshError, MeshNode, NodeIdentity, Request, Response, Result, ServiceAddress,
        ServiceId,
    };
}

//! DHT-backed agent registry for decentralized agent discovery.
//!
//! Agents register their DID→PeerId mapping in the Kademlia DHT.
//! Peers discover agents by DID using dht_get().

use serde::{Deserialize, Serialize};

/// Agent registration record stored in the Kademlia DHT.
/// Key: DID bytes. Value: JSON-serialized AgentRecord.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    /// Agent's DID (e.g. "did:exo:capsule:<uuid>")
    pub did: String,
    /// libp2p PeerId as base58-encoded string
    pub peer_id: String,
    /// Multiaddr strings where this agent can be reached
    pub listeners: Vec<String>,
    /// Agent type hint (e.g. "sentry", "hawk", "custom")
    pub agent_type: String,
    /// Registration timestamp (Unix seconds)
    pub registered_at: i64,
}

impl AgentRecord {
    /// Create a new AgentRecord with the current timestamp.
    pub fn new(did: String, peer_id: String, listeners: Vec<String>, agent_type: String) -> Self {
        Self {
            did,
            peer_id,
            listeners,
            agent_type,
            registered_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_record_creation() {
        let record = AgentRecord::new(
            "did:key:z6MkTest".to_string(),
            "12D3KooWTest".to_string(),
            vec!["/ip4/127.0.0.1/udp/4001/quic-v1".to_string()],
            "sentry".to_string(),
        );

        assert_eq!(record.did, "did:key:z6MkTest");
        assert_eq!(record.agent_type, "sentry");
        assert!(record.registered_at > 0);
    }

    #[test]
    fn test_agent_record_serialization() {
        let record = AgentRecord::new(
            "did:key:z6MkTest".to_string(),
            "12D3KooWTest".to_string(),
            vec![],
            "hawk".to_string(),
        );

        let json = serde_json::to_vec(&record).unwrap();
        let deserialized: AgentRecord = serde_json::from_slice(&json).unwrap();
        assert_eq!(deserialized.did, record.did);
        assert_eq!(deserialized.agent_type, record.agent_type);
    }
}

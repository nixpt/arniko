//! LAN discovery via mDNS
//!
//! Automatically discovers Exosphere nodes on the local network.

use crate::address::ServiceId;
use crate::message::ServiceInfo;
use crate::registry::ServiceRegistry;
use std::sync::Arc;

#[cfg(any(feature = "mdns", feature = "lan"))]
use libp2p::mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig, Event as MdnsEvent};

/// mDNS service type for Exosphere
pub const MDNS_SERVICE_TYPE: &str = "_exosphere._udp.local";

/// LAN discovery service
pub struct LanDiscovery {
    registry: Arc<ServiceRegistry>,
    node_did: String,
    /// Node identity — used to derive the stable PeerId so mDNS and P2P peers match
    identity: Arc<crate::identity::NodeIdentity>,
}

impl LanDiscovery {
    pub fn new(
        registry: Arc<ServiceRegistry>,
        node_did: String,
        identity: Arc<crate::identity::NodeIdentity>,
    ) -> Self {
        Self {
            registry,
            node_did,
            identity,
        }
    }

    #[cfg(any(feature = "mdns", feature = "lan"))]
    pub async fn start(&self) -> crate::Result<()> {
        use tokio::sync::mpsc;

        tracing::info!("Starting mDNS discovery");

        // Derive PeerId from the node's Ed25519 keypair so the mDNS-announced identity
        // matches the P2P transport identity (no more PeerId::random() mismatch).
        let keypair = self.identity.to_libp2p_keypair();
        let peer_id = keypair.public().to_peer_id();

        let config = MdnsConfig::default();
        let _mdns = Mdns::new(config, peer_id)?;

        let (_tx, mut rx) = mpsc::channel(100);

        // Spawn task to handle mDNS events
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    MdnsEvent::Discovered(list) => {
                        for (peer_id, addr) in list {
                            tracing::debug!("Discovered peer {} at {}", peer_id, addr);
                        }
                    }
                    MdnsEvent::Expired(list) => {
                        for (peer_id, addr) in list {
                            tracing::debug!("Peer {} at {} expired", peer_id, addr);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Announce a local service to the LAN
    pub fn announce(&self, service_id: &ServiceId) {
        if let Some(info) = self.registry.to_service_info(service_id, "lan") {
            tracing::info!("Announcing service {} on LAN", service_id);

            let record = ExosphereServiceRecord::new(&info.node_did, &self.node_did)
                .with_service(&service_id.to_string());

            let txt_records = record.to_txt_records();
            tracing::debug!("mDNS TXT records: {:?}", txt_records);
        }
    }

    /// Stop announcing a service
    pub fn withdraw(&self, service_id: &ServiceId) {
        tracing::info!("Withdrawing service {} from LAN", service_id);

        let _record = ExosphereServiceRecord::new(&self.node_did, &self.node_did)
            .with_service(&service_id.to_string());

        // In a real implementation, this would send a "goodbye" mDNS packet
        // For now, we just log the withdrawal
        tracing::debug!("Service {} withdrawn from mDNS broadcast", service_id);
    }

    /// Handle discovered peer
    fn on_peer_discovered(&self, peer_did: String, services: Vec<ServiceInfo>) {
        for service in services {
            self.registry.add_discovered(ServiceInfo {
                node_did: peer_did.clone(),
                location: "lan".to_string(),
                latency_hint_ms: Some(1), // LAN is very fast
                ..service
            });
        }
    }

    /// Handle peer expiration
    fn on_peer_expired(&self, peer_did: &str) {
        // Mark all services from this peer as unhealthy
        // They'll be cleaned up by the registry's stale cleanup
        tracing::debug!("LAN peer expired: {}", peer_did);
    }
}

/// DNS-SD service record for Exosphere
#[derive(Debug, Clone)]
pub struct ExosphereServiceRecord {
    /// Instance name (e.g., "alice-desktop")
    pub instance: String,
    /// Node DID
    pub did: String,
    /// Services offered
    pub services: Vec<String>,
    /// Port (always 0 for us - we use DIDs)
    pub port: u16,
    /// Additional TXT records
    pub txt: Vec<(String, String)>,
}

impl ExosphereServiceRecord {
    pub fn new(instance: impl Into<String>, did: impl Into<String>) -> Self {
        Self {
            instance: instance.into(),
            did: did.into(),
            services: vec![],
            port: 0,
            txt: vec![],
        }
    }

    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.services.push(service.into());
        self
    }

    /// Convert to mDNS TXT records
    pub fn to_txt_records(&self) -> Vec<String> {
        let mut records = vec![format!("did={}", self.did), format!("version=1")];

        for svc in &self.services {
            records.push(format!("svc={}", svc));
        }

        for (k, v) in &self.txt {
            records.push(format!("{}={}", k, v));
        }

        records
    }

    /// Parse from mDNS TXT records
    pub fn from_txt_records(instance: String, records: &[String]) -> Option<Self> {
        let mut did = None;
        let mut services = vec![];
        let mut txt = vec![];

        for record in records {
            if let Some((key, value)) = record.split_once('=') {
                match key {
                    "did" => did = Some(value.to_string()),
                    "svc" => services.push(value.to_string()),
                    _ => txt.push((key.to_string(), value.to_string())),
                }
            }
        }

        Some(Self {
            instance,
            did: did?,
            services,
            port: 0,
            txt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_record() {
        let record = ExosphereServiceRecord::new("alice-desktop", "did:key:z6MkTest")
            .with_service("system.wallet")
            .with_service("ai.brain");

        let txt = record.to_txt_records();
        assert!(txt.contains(&"did=did:key:z6MkTest".to_string()));
        assert!(txt.contains(&"svc=system.wallet".to_string()));
    }

    #[test]
    fn test_parse_txt_records() {
        let txt = vec![
            "did=did:key:z6MkTest".to_string(),
            "version=1".to_string(),
            "svc=ai.brain".to_string(),
        ];

        let record = ExosphereServiceRecord::from_txt_records("test".to_string(), &txt).unwrap();
        assert_eq!(record.did, "did:key:z6MkTest");
        assert!(record.services.contains(&"ai.brain".to_string()));
    }
}

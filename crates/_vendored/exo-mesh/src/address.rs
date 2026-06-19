//! Service addressing
//!
//! Unified addressing scheme that works across local, LAN, and P2P transports.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Unique identifier for a service instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId {
    /// Category (system, tools, ai, data, etc.)
    pub category: String,
    /// Service name within category
    pub name: String,
    /// Optional instance suffix for multiple instances
    pub instance: Option<String>,
}

impl ServiceId {
    pub fn new(category: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            instance: None,
        }
    }

    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = Some(instance.into());
        self
    }

    /// Parse from string format "category.name" or "category.name#instance"
    pub fn parse(s: &str) -> Option<Self> {
        let (main, instance) = if let Some(idx) = s.find('#') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        let parts: Vec<&str> = main.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        Some(Self {
            category: parts[0].to_string(),
            name: parts[1..].join("."),
            instance,
        })
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.instance {
            Some(inst) => write!(f, "{}.{}#{}", self.category, self.name, inst),
            None => write!(f, "{}.{}", self.category, self.name),
        }
    }
}

impl FromStr for ServiceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ServiceId::parse(s).ok_or_else(|| format!("Invalid service ID: {}", s))
    }
}

/// Full service address including location hints
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceAddress {
    /// The service identifier
    pub service_id: ServiceId,
    /// Optional node DID (for remote services)
    pub node_did: Option<String>,
    /// Location hint (local, lan, remote)
    pub location: LocationHint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationHint {
    /// Same machine (Unix socket)
    Local,
    /// Same network (mDNS discoverable)
    Lan,
    /// Internet (DHT lookup required)
    Remote,
    /// Unknown - will be resolved
    Unknown,
}

impl Default for LocationHint {
    fn default() -> Self {
        LocationHint::Unknown
    }
}

impl ServiceAddress {
    /// Create a local service address
    pub fn local(service_id: ServiceId) -> Self {
        Self {
            service_id,
            node_did: None,
            location: LocationHint::Local,
        }
    }

    /// Create a remote service address
    pub fn remote(service_id: ServiceId, node_did: String) -> Self {
        Self {
            service_id,
            node_did: Some(node_did),
            location: LocationHint::Remote,
        }
    }

    /// Create from just a service ID (location unknown)
    pub fn from_id(service_id: ServiceId) -> Self {
        Self {
            service_id,
            node_did: None,
            location: LocationHint::Unknown,
        }
    }

    /// Parse from URL format
    /// - `capsule://system.wallet` (local)
    /// - `capsule://system.wallet@did:key:z6Mk...` (remote)
    pub fn from_url(url: &str) -> Option<Self> {
        if !url.starts_with("capsule://") {
            return None;
        }

        let rest = &url[10..];

        // Check for @did:key: pattern
        if let Some(at_idx) = rest.find('@') {
            let service_part = &rest[..at_idx];
            let did_part = &rest[at_idx + 1..];

            let service_id = ServiceId::parse(service_part)?;
            Some(Self {
                service_id,
                node_did: Some(did_part.to_string()),
                location: LocationHint::Remote,
            })
        } else {
            let service_id = ServiceId::parse(rest)?;
            Some(Self::from_id(service_id))
        }
    }

    /// Convert to URL format
    pub fn to_url(&self) -> String {
        match &self.node_did {
            Some(did) => format!("capsule://{}@{}", self.service_id, did),
            None => format!("capsule://{}", self.service_id),
        }
    }

    /// Get Unix socket path for local services
    pub fn local_socket_path(&self) -> String {
        format!("/run/exosphere/{}.sock", self.service_id)
    }
}

impl fmt::Display for ServiceAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_url())
    }
}

/// Well-known system services
pub mod services {
    use super::ServiceId;

    pub fn wallet() -> ServiceId {
        ServiceId::new("system", "wallet")
    }

    pub fn privacy_gateway() -> ServiceId {
        ServiceId::new("system", "gateway")
    }

    pub fn coordinator() -> ServiceId {
        ServiceId::new("system", "coordinator")
    }

    pub fn mesh() -> ServiceId {
        ServiceId::new("system", "mesh")
    }

    pub fn ai_brain() -> ServiceId {
        ServiceId::new("ai", "brain")
    }

    pub fn teddy() -> ServiceId {
        ServiceId::new("tools", "teddy")
    }

    pub fn vortex() -> ServiceId {
        ServiceId::new("tools", "vortex")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_id_parse() {
        let id = ServiceId::parse("system.wallet").unwrap();
        assert_eq!(id.category, "system");
        assert_eq!(id.name, "wallet");
        assert_eq!(id.instance, None);

        let id = ServiceId::parse("system.wallet#1").unwrap();
        assert_eq!(id.instance, Some("1".to_string()));
    }

    #[test]
    fn test_service_address_url() {
        let addr = ServiceAddress::from_url("capsule://system.wallet").unwrap();
        assert_eq!(addr.service_id.category, "system");
        assert_eq!(addr.service_id.name, "wallet");
        assert_eq!(addr.node_did, None);

        let addr = ServiceAddress::from_url("capsule://ai.brain@did:key:z6Mk123").unwrap();
        assert_eq!(addr.node_did, Some("did:key:z6Mk123".to_string()));
    }

    #[test]
    fn test_roundtrip() {
        let addr = ServiceAddress::remote(
            ServiceId::new("ai", "brain"),
            "did:key:z6MkTest".to_string(),
        );
        let url = addr.to_url();
        let parsed = ServiceAddress::from_url(&url).unwrap();
        assert_eq!(addr, parsed);
    }
}

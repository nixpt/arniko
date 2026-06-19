//! Service registry for tracking local and discovered services

use crate::address::{LocationHint, ServiceAddress, ServiceId};
use crate::message::ServiceInfo;
use dashmap::DashMap;
use std::time::{Duration, Instant};

/// Service registry with automatic expiration
pub struct ServiceRegistry {
    /// Local services (registered on this node)
    local: DashMap<String, LocalService>,
    /// Discovered services (from LAN/P2P)
    discovered: DashMap<String, DiscoveredService>,
    /// Our node's DID
    node_did: String,
}

#[derive(Debug, Clone)]
pub struct LocalService {
    pub service_id: ServiceId,
    pub methods: Vec<String>,
    pub version: String,
    pub socket_path: String,
    pub registered_at: Instant,
}

#[derive(Debug, Clone)]
pub struct DiscoveredService {
    pub info: ServiceInfo,
    pub discovered_at: Instant,
    pub last_seen: Instant,
    pub healthy: bool,
}

impl ServiceRegistry {
    pub fn new(node_did: String) -> Self {
        Self {
            local: DashMap::new(),
            discovered: DashMap::new(),
            node_did,
        }
    }

    /// Register a local service
    pub fn register_local(
        &self,
        service_id: ServiceId,
        methods: Vec<String>,
        version: String,
    ) -> ServiceAddress {
        let socket_path = format!("/run/exosphere/{}.sock", service_id);

        let service = LocalService {
            service_id: service_id.clone(),
            methods,
            version,
            socket_path: socket_path.clone(),
            registered_at: Instant::now(),
        };

        self.local.insert(service_id.to_string(), service);

        ServiceAddress::local(service_id)
    }

    /// Unregister a local service
    pub fn unregister_local(&self, service_id: &ServiceId) -> bool {
        self.local.remove(&service_id.to_string()).is_some()
    }

    /// Add a discovered service
    pub fn add_discovered(&self, info: ServiceInfo) {
        let key = format!("{}@{}", info.service_id, info.node_did);
        let now = Instant::now();

        self.discovered.insert(
            key,
            DiscoveredService {
                info,
                discovered_at: now,
                last_seen: now,
                healthy: true,
            },
        );
    }

    /// Update last_seen for a discovered service
    pub fn touch_discovered(&self, service_id: &str, node_did: &str) {
        let key = format!("{}@{}", service_id, node_did);
        if let Some(mut entry) = self.discovered.get_mut(&key) {
            entry.last_seen = Instant::now();
            entry.healthy = true;
        }
    }

    /// Mark a discovered service as unhealthy
    pub fn mark_unhealthy(&self, service_id: &str, node_did: &str) {
        let key = format!("{}@{}", service_id, node_did);
        if let Some(mut entry) = self.discovered.get_mut(&key) {
            entry.healthy = false;
        }
    }

    /// Find a service by ID (prefers local, then LAN, then remote)
    pub fn find(&self, service_id: &ServiceId) -> Option<ServiceAddress> {
        let id_str = service_id.to_string();

        // Check local first
        if self.local.contains_key(&id_str) {
            return Some(ServiceAddress::local(service_id.clone()));
        }

        // Check discovered services
        let mut best: Option<(ServiceAddress, u32)> = None;

        for entry in self.discovered.iter() {
            let svc = entry.value();
            if svc.info.service_id == id_str && svc.healthy {
                let latency = svc.info.latency_hint_ms.unwrap_or(1000);
                let location = match svc.info.location.as_str() {
                    "local" => LocationHint::Local,
                    "lan" => LocationHint::Lan,
                    _ => LocationHint::Remote,
                };

                let addr = ServiceAddress {
                    service_id: service_id.clone(),
                    node_did: Some(svc.info.node_did.clone()),
                    location,
                };

                match &best {
                    None => best = Some((addr, latency)),
                    Some((_, best_latency)) if latency < *best_latency => {
                        best = Some((addr, latency));
                    }
                    _ => {}
                }
            }
        }

        best.map(|(addr, _)| addr)
    }

    /// Find all instances of a service
    pub fn find_all(&self, service_id: &ServiceId) -> Vec<ServiceAddress> {
        let id_str = service_id.to_string();
        let mut results = Vec::new();

        // Local
        if self.local.contains_key(&id_str) {
            results.push(ServiceAddress::local(service_id.clone()));
        }

        // Discovered
        for entry in self.discovered.iter() {
            let svc = entry.value();
            if svc.info.service_id == id_str && svc.healthy {
                let location = match svc.info.location.as_str() {
                    "local" => LocationHint::Local,
                    "lan" => LocationHint::Lan,
                    _ => LocationHint::Remote,
                };

                results.push(ServiceAddress {
                    service_id: service_id.clone(),
                    node_did: Some(svc.info.node_did.clone()),
                    location,
                });
            }
        }

        results
    }

    /// Get info for a local service
    pub fn get_local(&self, service_id: &ServiceId) -> Option<LocalService> {
        self.local.get(&service_id.to_string()).map(|e| e.clone())
    }

    /// List all local services
    pub fn list_local(&self) -> Vec<LocalService> {
        self.local.iter().map(|e| e.value().clone()).collect()
    }

    /// Clean up stale discovered services
    pub fn cleanup_stale(&self, max_age: Duration) {
        let now = Instant::now();
        self.discovered
            .retain(|_, v| now.duration_since(v.last_seen) < max_age);
    }

    /// Get service info for broadcasting
    pub fn to_service_info(&self, service_id: &ServiceId, location: &str) -> Option<ServiceInfo> {
        self.local
            .get(&service_id.to_string())
            .map(|svc| ServiceInfo {
                service_id: svc.service_id.to_string(),
                node_did: self.node_did.clone(),
                location: location.to_string(),
                methods: svc.methods.clone(),
                version: svc.version.clone(),
                latency_hint_ms: None,
            })
    }

    /// Get our node's DID
    pub fn node_did(&self) -> &str {
        &self.node_did
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_find() {
        let registry = ServiceRegistry::new("did:key:test".to_string());

        let id = ServiceId::new("system", "wallet");
        registry.register_local(id.clone(), vec!["sign".to_string()], "0.1.0".to_string());

        let found = registry.find(&id);
        assert!(found.is_some());
        assert!(matches!(found.unwrap().location, LocationHint::Local));
    }

    #[test]
    fn test_discovered_service() {
        let registry = ServiceRegistry::new("did:key:local".to_string());

        let info = ServiceInfo {
            service_id: "ai.brain".to_string(),
            node_did: "did:key:remote".to_string(),
            location: "lan".to_string(),
            methods: vec!["generate".to_string()],
            version: "0.1.0".to_string(),
            latency_hint_ms: Some(5),
        };

        registry.add_discovered(info);

        let id = ServiceId::new("ai", "brain");
        let found = registry.find(&id);
        assert!(found.is_some());
        assert!(matches!(found.unwrap().location, LocationHint::Lan));
    }
}

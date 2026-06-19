//! RESOLVE-02 — reconciliation bridge between the two parallel addressing models.
//!
//! # Why this file exists
//!
//! Exosphere grew two addressing models in parallel, unaware of each other:
//!
//! - [`crate::address::ServiceAddress`] (this crate) — a **located, routable endpoint**:
//!   a [`ServiceId`](crate::address::ServiceId) (`category.name#instance`) plus an optional
//!   `node_did` and a [`LocationHint`](crate::address::LocationHint) (`local`/`lan`/`remote`),
//!   with `local_socket_path()` for Unix-socket dispatch. **This is the model that real
//!   routing uses today** (`node.rs`, `local.rs`, `registry.rs`, `transport.rs`, and
//!   `exo-bliss-net`).
//! - [`exo_identity::ExoAddress`] (the `exo-identity` crate) — a **name/URI to be resolved**:
//!   a `scheme` (`exo`/`capsule`/`service`) + `identity` + `path[]` + `@version` + `query{}`,
//!   fed into the capability-gated, session-stable [`AddressResolver`](exo_identity::AddressResolver).
//!   It is the richer naming grammar built by the ADDR-01..05 track, but has **zero production
//!   callers** — only the resolver and tests construct it.
//!
//! # The reconciliation decision: a bridge, NOT one canonical model
//!
//! Neither model is a superset of the other, and they sit at different stages of the routing
//! lifecycle:
//!
//! - `ExoAddress` is *pre-resolution* — a name plus addressing grammar (path/version/query)
//!   and the capability machinery that gates resolving it.
//! - `ServiceAddress` is *located* — it already carries where (`node_did` + `LocationHint`)
//!   and how (`local_socket_path()`) to reach a concrete endpoint.
//!
//! Collapsing to one canonical type would either discard `ExoAddress`'s path/version/query +
//! the whole gated/session-stable resolver, or discard `ServiceAddress`'s node/location/socket
//! routing semantics. The v1.8 spec is explicit: **reconcile, do not rebuild** — and
//! `ServiceAddress` is load-bearing. So the reconciliation is an **explicit, documented,
//! bidirectional bridge**: [`From<ServiceAddress> for ExoAddress`] and
//! [`TryFrom<ExoAddress> for ServiceAddress`]. With both conversions named in code, no path
//! can silently pick one model over the other — crossing between them is always an explicit
//! `.into()` / `.try_into()`.
//!
//! RESOLVE-01 (transport wiring) consumes this: `ServiceAddress` stays the routing model;
//! `ExoAddress` is the naming/resolution model that is *bridged into* a `ServiceAddress`
//! before dispatch.
//!
//! # The mapping and its lossy edges
//!
//! | `ExoAddress` field | `ServiceAddress` field | Notes |
//! |---|---|---|
//! | `identity` (`"cat.name#inst"`) | `service_id` | via [`ServiceId::parse`]/`Display` |
//! | `scheme` | (always `capsule://` in `ServiceAddress`'s URL form) | `Capsule`/`Service` accepted inbound; `Exo` rejected |
//! | `query["node"]` | `node_did` | preserved across the bridge |
//! | `query["location"]` | `location` | preserved across the bridge |
//! | `path[]` | — | **lossy**: no `ServiceAddress` home |
//! | `version` | — | **lossy**: no `ServiceAddress` home |
//! | other `query` keys | — | **lossy** |
//!
//! `ServiceAddress -> ExoAddress` is **information-preserving**: `node_did` and `location` are
//! stashed into the `query` map (keys `node` / `location`) so that the reverse conversion can
//! reconstruct them exactly. The round-trip `ServiceAddress -> ExoAddress -> ServiceAddress` is
//! therefore **exact** (see [`tests::roundtrip_service_address_is_exact`]).
//!
//! `ExoAddress -> ServiceAddress` is **lossy by design**: an `ExoAddress` may carry a `path`,
//! `@version`, or extra `query` keys that have no representation in a `ServiceAddress`; those
//! are dropped. The reverse round-trip therefore preserves only the recoverable core
//! (`service_id` + `node_did` + `location`). It is also **fallible** — an `ExoAddress` whose
//! `identity` is not a valid `category.name` `ServiceId`, or whose `scheme` is `exo`, has no
//! `ServiceAddress` representation and yields an [`AddressBridgeError`].

use crate::address::{LocationHint, ServiceAddress, ServiceId};
use exo_identity::{ExoAddress, ExoScheme};

/// Query-map key under which [`ServiceAddress::node_did`] is preserved when bridging to
/// [`ExoAddress`], so the reverse conversion can reconstruct it.
const QUERY_KEY_NODE: &str = "node";
/// Query-map key under which [`ServiceAddress::location`] is preserved when bridging to
/// [`ExoAddress`].
const QUERY_KEY_LOCATION: &str = "location";

/// Error from the lossy/fallible [`ExoAddress`] -> [`ServiceAddress`] direction of the bridge.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AddressBridgeError {
    /// The `ExoAddress` `identity` is not a parseable `category.name` [`ServiceId`].
    #[error("identity '{0}' is not a valid service id (expected 'category.name')")]
    NotAServiceId(String),
    /// The `ExoAddress` scheme has no `ServiceAddress` representation (only `capsule`/`service`
    /// map; `exo` is an identity-naming scheme with no routable service form).
    #[error("scheme '{0}' has no ServiceAddress representation (expected capsule or service)")]
    UnsupportedScheme(String),
}

impl LocationHint {
    /// Lowercase token used when a [`LocationHint`] is preserved in an [`ExoAddress`] query map.
    fn as_query_token(self) -> &'static str {
        match self {
            LocationHint::Local => "local",
            LocationHint::Lan => "lan",
            LocationHint::Remote => "remote",
            LocationHint::Unknown => "unknown",
        }
    }

    /// Inverse of [`LocationHint::as_query_token`]; unrecognized tokens fall back to `Unknown`.
    fn from_query_token(s: &str) -> Self {
        match s {
            "local" => LocationHint::Local,
            "lan" => LocationHint::Lan,
            "remote" => LocationHint::Remote,
            _ => LocationHint::Unknown,
        }
    }
}

/// `ServiceAddress -> ExoAddress`: total and information-preserving.
///
/// The `service_id` becomes the `identity` (its `category.name#instance` `Display` form), the
/// scheme is fixed to [`ExoScheme::Capsule`] (the scheme whose `ServiceAddress` URL form is
/// `capsule://`), and the routing locators (`node_did`, `location`) are stashed in the `query`
/// map so the reverse conversion is exact. `path`/`version` are empty — a `ServiceAddress` has
/// no analogue.
impl From<ServiceAddress> for ExoAddress {
    fn from(addr: ServiceAddress) -> Self {
        let mut query = std::collections::HashMap::new();
        if let Some(did) = &addr.node_did {
            query.insert(QUERY_KEY_NODE.to_string(), did.clone());
        }
        query.insert(
            QUERY_KEY_LOCATION.to_string(),
            addr.location.as_query_token().to_string(),
        );
        ExoAddress {
            scheme: ExoScheme::Capsule,
            identity: addr.service_id.to_string(),
            path: Vec::new(),
            version: None,
            query,
        }
    }
}

/// `ExoAddress -> ServiceAddress`: fallible and lossy by design.
///
/// Recovers `service_id` from `identity` (via [`ServiceId::parse`]), and `node_did`/`location`
/// from the preserved query keys when present (otherwise `node_did = None` and the location is
/// derived from `node_did` presence, mirroring [`ServiceAddress::from_url`]). `path`, `version`,
/// and any extra query keys are **dropped**. Fails with [`AddressBridgeError`] when the identity
/// is not a `category.name` service id, or the scheme is [`ExoScheme::Exo`].
impl TryFrom<ExoAddress> for ServiceAddress {
    type Error = AddressBridgeError;

    fn try_from(addr: ExoAddress) -> Result<Self, Self::Error> {
        match addr.scheme {
            ExoScheme::Capsule | ExoScheme::Service => {}
            ExoScheme::Exo => {
                return Err(AddressBridgeError::UnsupportedScheme(
                    addr.scheme.as_str().to_string(),
                ))
            }
        }

        let service_id = ServiceId::parse(&addr.identity)
            .ok_or_else(|| AddressBridgeError::NotAServiceId(addr.identity.clone()))?;

        let node_did = addr.query.get(QUERY_KEY_NODE).cloned();
        let location = match addr.query.get(QUERY_KEY_LOCATION) {
            Some(tok) => LocationHint::from_query_token(tok),
            // No preserved hint: derive from node presence, matching `ServiceAddress::from_url`
            // (a `@did` makes it Remote; otherwise the location is Unknown).
            None if node_did.is_some() => LocationHint::Remote,
            None => LocationHint::Unknown,
        };

        Ok(ServiceAddress {
            service_id,
            node_did,
            location,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_service_address_is_exact() {
        // DONE-CONDITION: ServiceAddress -> ExoAddress -> ServiceAddress preserves every field,
        // for both a remote (node_did + Remote) and a local (no node_did) address.
        let remote = ServiceAddress::remote(
            ServiceId::new("ai", "brain").with_instance("2"),
            "did:key:z6MkRoundTrip".to_string(),
        );
        let exo: ExoAddress = remote.clone().into();
        let back: ServiceAddress = exo.try_into().unwrap();
        assert_eq!(remote, back);

        let local = ServiceAddress::local(ServiceId::new("system", "wallet"));
        let exo: ExoAddress = local.clone().into();
        let back: ServiceAddress = exo.try_into().unwrap();
        assert_eq!(local, back);
    }

    #[test]
    fn service_address_to_exo_address_field_mapping() {
        let svc = ServiceAddress::remote(
            ServiceId::new("ai", "brain"),
            "did:key:z6MkABC".to_string(),
        );
        let exo: ExoAddress = svc.into();

        assert_eq!(exo.scheme, ExoScheme::Capsule);
        assert_eq!(exo.identity, "ai.brain");
        assert!(exo.path.is_empty());
        assert_eq!(exo.version, None);
        // Routing locators preserved in the query map so the reverse is exact.
        assert_eq!(exo.query.get(QUERY_KEY_NODE).unwrap(), "did:key:z6MkABC");
        assert_eq!(exo.query.get(QUERY_KEY_LOCATION).unwrap(), "remote");
    }

    #[test]
    fn exo_address_to_service_address_drops_lossy_fields() {
        // path + @version + extra query keys have no ServiceAddress home and are dropped, but
        // the conversion still succeeds on the recoverable core.
        let exo = ExoAddress::parse("capsule://system.wallet@v3/sub/path?theme=dark").unwrap();
        assert!(!exo.path.is_empty());
        assert_eq!(exo.version, Some("v3".to_string()));

        let svc: ServiceAddress = exo.try_into().unwrap();
        assert_eq!(svc.service_id, ServiceId::new("system", "wallet"));
        assert_eq!(svc.node_did, None);
        // No preserved location hint + no node -> Unknown (mirrors from_url).
        assert_eq!(svc.location, LocationHint::Unknown);
    }

    #[test]
    fn exo_address_recovers_node_and_location_from_query() {
        // An ExoAddress carrying the preserved query keys reconstructs node_did + location.
        let exo =
            ExoAddress::parse("service://tools.vortex?node=did:key:z6MkXYZ&location=lan").unwrap();
        let svc: ServiceAddress = exo.try_into().unwrap();
        assert_eq!(svc.service_id, ServiceId::new("tools", "vortex"));
        assert_eq!(svc.node_did, Some("did:key:z6MkXYZ".to_string()));
        assert_eq!(svc.location, LocationHint::Lan);
    }

    #[test]
    fn exo_scheme_has_no_service_address_form() {
        // The `exo://` identity-naming scheme is not a routable service form.
        let exo = ExoAddress::parse("exo://alice/notes").unwrap();
        let err = ServiceAddress::try_from(exo).unwrap_err();
        assert_eq!(err, AddressBridgeError::UnsupportedScheme("exo".to_string()));
    }

    #[test]
    fn non_service_id_identity_is_rejected() {
        // A single-segment identity is not a `category.name` ServiceId.
        let exo = ExoAddress::parse("capsule://editor@v1").unwrap();
        let err = ServiceAddress::try_from(exo).unwrap_err();
        assert_eq!(err, AddressBridgeError::NotAServiceId("editor".to_string()));
    }
}

//! Capsule:// scheme handler
//!
//! Routes requests to Exosphere services via mesh networking.
//!
//! URL format: `capsule://<category>.<service>[/<path>][?<query>]`
//!
//! Examples:
//! - `capsule://system.assets/logo.png` - Load asset from system.assets service
//! - `capsule://ai.brain/complete` - Call AI completion endpoint
//! - `capsule://system.wallet@did:key:z6Mk.../balance` - Remote service call

use std::sync::Arc;

use bliss_traits::net::{NetHandler, Request};
use bytes::Bytes;
use exo_mesh::{MeshNode, ServiceAddress};
use tokio::runtime::Handle;

use crate::SchemeHandler;

/// Handler for capsule:// URLs
pub struct CapsuleSchemeHandler {
    mesh_node: Arc<MeshNode>,
    rt: Handle,
}

impl CapsuleSchemeHandler {
    pub fn new(mesh_node: Arc<MeshNode>) -> Self {
        Self {
            mesh_node,
            rt: Handle::current(),
        }
    }

    pub fn with_runtime(mesh_node: Arc<MeshNode>, rt: Handle) -> Self {
        Self { mesh_node, rt }
    }
}

impl SchemeHandler for CapsuleSchemeHandler {
    fn schemes(&self) -> &[&str] {
        &["capsule"]
    }

    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url_str = request.url.to_string();

        // Parse the capsule URL
        let Some(service_addr) = ServiceAddress::from_url(&url_str) else {
            tracing::warn!(url = %url_str, "invalid capsule URL");
            handler.error(format!("Invalid capsule URL: {}", url_str));
            return;
        };

        // Extract path from URL (this becomes the method name)
        let method = extract_path(&request.url);

        // Build params from query string
        let params = extract_query_params(&request.url);

        let mesh_node = self.mesh_node.clone();
        let service_id = service_addr.service_id.clone();

        tracing::debug!(
            url = %url_str,
            service = %service_addr,
            method = %method,
            doc_id,
            "routing capsule request"
        );

        self.rt.spawn(async move {
            // Use the mesh node's call method
            match mesh_node.call(&service_id, &method, params).await {
                Ok(result) => {
                    // Serialize result to bytes
                    let bytes = Bytes::from(serde_json::to_vec(&result).unwrap_or_default());

                    tracing::debug!(
                        url = %url_str,
                        size = bytes.len(),
                        "capsule request completed"
                    );

                    handler.bytes(url_str, bytes);
                }
                Err(err) => {
                    tracing::warn!(url = %url_str, %err, "capsule request failed");
                    handler.error(err.to_string());
                }
            }
        });
    }
}

/// Extract path from capsule URL (portion after service ID)
fn extract_path(url: &url::Url) -> String {
    url.path().trim_start_matches('/').to_string()
}

/// Extract query parameters as JSON object
fn extract_query_params(url: &url::Url) -> serde_json::Value {
    let pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    if pairs.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::json!(pairs.into_iter().collect::<std::collections::HashMap<_, _>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path() {
        let url = url::Url::parse("capsule://system.assets/images/logo.png").unwrap();
        assert_eq!(extract_path(&url), "images/logo.png");

        let url = url::Url::parse("capsule://ai.brain").unwrap();
        assert_eq!(extract_path(&url), "");
    }

    #[test]
    fn test_extract_query_params() {
        let url = url::Url::parse("capsule://ai.brain/complete?prompt=hello&max_tokens=100").unwrap();
        let params = extract_query_params(&url);
        assert_eq!(params["prompt"], "hello");
        assert_eq!(params["max_tokens"], "100");
    }
}

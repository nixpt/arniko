//! HTTP fallback scheme handler
//!
//! Provides HTTP/HTTPS fetching as a fallback when no other scheme handler matches.

use bliss_traits::net::{NetHandler, Request};
use bytes::Bytes;
use tokio::runtime::Handle;

use crate::SchemeHandler;

/// Handler for http:// and https:// URLs
pub struct HttpSchemeHandler {
    client: reqwest::Client,
    rt: Handle,
}

impl HttpSchemeHandler {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            rt: Handle::current(),
        }
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            rt: Handle::current(),
        }
    }

    pub fn with_runtime(client: reqwest::Client, rt: Handle) -> Self {
        Self { client, rt }
    }
}

impl Default for HttpSchemeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemeHandler for HttpSchemeHandler {
    fn schemes(&self) -> &[&str] {
        &["http", "https"]
    }

    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        let url = request.url.clone();
        let url_str = url.to_string();
        let client = self.client.clone();

        tracing::debug!(url = %url_str, doc_id, "fetching via HTTP fallback");

        self.rt.spawn(async move {
            match fetch_http(client, url).await {
                Ok(bytes) => {
                    tracing::debug!(
                        url = %url_str,
                        size = bytes.len(),
                        "HTTP fetch completed"
                    );
                    handler.bytes(url_str, bytes);
                }
                Err(err) => {
                    tracing::warn!(url = %url_str, %err, "HTTP fetch failed");
                    handler.error(err.to_string());
                }
            }
        });
    }
}

/// Perform HTTP fetch
async fn fetch_http(client: reqwest::Client, url: url::Url) -> anyhow::Result<Bytes> {
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_scheme_handler_schemes() {
        let handler = HttpSchemeHandler::new();
        let schemes = handler.schemes();
        assert!(schemes.contains(&"http"));
        assert!(schemes.contains(&"https"));
    }
}
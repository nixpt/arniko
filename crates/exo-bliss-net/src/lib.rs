use std::sync::Arc;

use bliss_traits::net::{DefaultNetPolicy, NetHandler, NetPolicy, NetProvider, Request};

mod scheme;

#[cfg(feature = "capsule")]
mod capsule;

#[cfg(feature = "http-fallback")]
mod http;

pub use scheme::{DataSchemeHandler, FileSchemeHandler, SchemeHandler};

#[cfg(feature = "capsule")]
pub use capsule::CapsuleSchemeHandler;

#[cfg(feature = "http-fallback")]
pub use http::HttpSchemeHandler;

pub struct ExoNetProvider {
    scheme_handlers: Vec<Box<dyn SchemeHandler>>,
    fallback: Option<Arc<dyn NetProvider>>,
    policy: Arc<dyn NetPolicy>,
}

impl ExoNetProvider {
    pub fn builder() -> ExoNetProviderBuilder {
        ExoNetProviderBuilder::new()
    }

    pub fn find_handler(&self, scheme: &str) -> Option<&dyn SchemeHandler> {
        self.scheme_handlers
            .iter()
            .find(|h| h.schemes().iter().any(|s| s.eq_ignore_ascii_case(scheme)))
            .map(|h| h.as_ref())
    }
}

impl NetProvider for ExoNetProvider {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        if let Err(reason) = self.policy.allow_fetch(doc_id, &request.url) {
            tracing::warn!(
                url = %request.url,
                doc_id,
                reason = %reason,
                "fetch denied by policy"
            );
            return;
        }

        let scheme = request.url.scheme();

        if let Some(scheme_handler) = self.find_handler(scheme) {
            tracing::debug!(url = %request.url, scheme, doc_id, "routing to scheme handler");
            scheme_handler.fetch(doc_id, request, handler);
            return;
        }

        if let Some(ref fallback) = self.fallback {
            tracing::debug!(url = %request.url, scheme, doc_id, "routing to fallback provider");
            fallback.fetch(doc_id, request, handler);
            return;
        }

        tracing::warn!(url = %request.url, scheme, doc_id, "no handler for scheme");
    }
}

pub struct ExoNetProviderBuilder {
    scheme_handlers: Vec<Box<dyn SchemeHandler>>,
    fallback: Option<Arc<dyn NetProvider>>,
    policy: Option<Arc<dyn NetPolicy>>,
}

impl ExoNetProviderBuilder {
    pub fn new() -> Self {
        Self {
            scheme_handlers: Vec::new(),
            fallback: None,
            policy: None,
        }
    }

    pub fn add_handler(mut self, handler: impl SchemeHandler) -> Self {
        self.scheme_handlers.push(Box::new(handler));
        self
    }

    pub fn fallback(mut self, provider: Arc<dyn NetProvider>) -> Self {
        self.fallback = Some(provider);
        self
    }

    pub fn policy(mut self, policy: Arc<dyn NetPolicy>) -> Self {
        self.policy = Some(policy);
        self
    }

    pub fn build(self) -> ExoNetProvider {
        ExoNetProvider {
            scheme_handlers: self.scheme_handlers,
            fallback: self.fallback,
            policy: self.policy.unwrap_or_else(|| Arc::new(DefaultNetPolicy)),
        }
    }
}

impl Default for ExoNetProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

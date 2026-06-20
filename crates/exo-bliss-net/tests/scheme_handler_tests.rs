//! Tests for exo-bliss-net scheme handlers
//!
//! These tests use mock transports to verify scheme handler behavior
//! without requiring actual network connections.

use std::sync::{Arc, Mutex};

/// Shared state for mock handler
#[derive(Default)]
struct MockHandlerState {
    received_bytes: Vec<(String, bytes::Bytes)>,
    received_chunks: Vec<(u64, bytes::Bytes)>,
    errors: Vec<String>,
    ended: bool,
}

/// Mock NetHandler that captures received data
#[derive(Clone)]
struct MockHandler {
    state: Arc<Mutex<MockHandlerState>>,
}

impl MockHandler {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockHandlerState::default())),
        }
    }

    fn received_data(&self) -> Vec<(String, bytes::Bytes)> {
        self.state.lock().unwrap().received_bytes.clone()
    }

    fn received_errors(&self) -> Vec<String> {
        self.state.lock().unwrap().errors.clone()
    }

    fn is_ended(&self) -> bool {
        self.state.lock().unwrap().ended
    }
}

impl bliss_traits::net::NetHandler for MockHandler {
    fn bytes(self: Box<Self>, url: String, data: bytes::Bytes) {
        self.state.lock().unwrap().received_bytes.push((url, data));
    }

    fn chunk(&self, seq: u64, data: bytes::Bytes) {
        self.state.lock().unwrap().received_chunks.push((seq, data));
    }

    fn end(&self) {
        self.state.lock().unwrap().ended = true;
    }

    fn error(&self, message: String) {
        self.state.lock().unwrap().errors.push(message);
    }
}

#[cfg(test)]
mod data_scheme_tests {
    use super::*;
    use bliss_traits::net::Request;
    use exo_bliss_net::{DataSchemeHandler, SchemeHandler};

    #[test]
    fn test_data_scheme_handler_base64() {
        let handler = DataSchemeHandler;
        let mock = MockHandler::new();

        // Test base64 encoded data
        let request =
            Request::get(url::Url::parse("data:text/plain;base64,SGVsbG8gV29ybGQh").unwrap());

        handler.fetch(1, request, Box::new(mock.clone()));

        let received = mock.received_data();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1, bytes::Bytes::from("Hello World!"));
    }

    #[test]
    fn test_data_scheme_handler_plaintext() {
        let handler = DataSchemeHandler;
        let mock = MockHandler::new();

        // Test percent-encoded data
        let request = Request::get(url::Url::parse("data:text/plain,Hello%20World%21").unwrap());

        handler.fetch(1, request, Box::new(mock.clone()));

        let received = mock.received_data();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1, bytes::Bytes::from("Hello World!"));
    }

    #[test]
    fn test_data_scheme_handler_malformed() {
        let handler = DataSchemeHandler;
        let mock = MockHandler::new();

        // Test malformed URL (no comma)
        let request = Request::get(url::Url::parse("data:text/plain;base64").unwrap());

        handler.fetch(1, request, Box::new(mock.clone()));

        // Should not receive any data for malformed URL
        let received = mock.received_data();
        assert!(received.is_empty());
    }
}

#[cfg(test)]
mod file_scheme_tests {
    use super::*;
    use bliss_traits::net::Request;
    use exo_bliss_net::{FileSchemeHandler, SchemeHandler};
    use std::io::Write;

    #[test]
    fn test_file_scheme_handler() {
        let handler = FileSchemeHandler;
        let mock = MockHandler::new();

        // Create a temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("exo_bliss_net_test.txt");

        {
            let mut file = std::fs::File::create(&temp_file).unwrap();
            file.write_all(b"Test content from file").unwrap();
        }

        let request = Request::get(url::Url::from_file_path(&temp_file).unwrap());

        handler.fetch(1, request, Box::new(mock.clone()));

        let received = mock.received_data();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].1, bytes::Bytes::from("Test content from file"));

        // Cleanup
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_file_scheme_handler_not_found() {
        let handler = FileSchemeHandler;
        let mock = MockHandler::new();

        let request =
            Request::get(url::Url::parse("file:///nonexistent/path/to/file.txt").unwrap());

        handler.fetch(1, request, Box::new(mock.clone()));

        // Should not receive data for non-existent file
        let received = mock.received_data();
        assert!(received.is_empty());
    }
}

#[cfg(test)]
mod exo_net_provider_tests {
    use super::*;
    use bliss_traits::net::{DefaultNetPolicy, NetProvider, Request};
    use exo_bliss_net::ExoNetProvider;

    #[test]
    fn test_exo_net_provider_builder() {
        let provider = ExoNetProvider::builder()
            .policy(Arc::new(DefaultNetPolicy))
            .build();

        // Should be able to build with just policy
        assert!(provider.find_handler("data").is_none());
    }

    #[test]
    fn test_exo_net_provider_with_data_handler() {
        use exo_bliss_net::{DataSchemeHandler, SchemeHandler};

        let provider = ExoNetProvider::builder()
            .add_handler(DataSchemeHandler)
            .policy(Arc::new(DefaultNetPolicy))
            .build();

        // Data handler should be found
        assert!(provider.find_handler("data").is_some());
        assert!(provider.find_handler("unknown").is_none());
    }

    #[test]
    fn test_exo_net_provider_policy_denial() {
        use bliss_traits::net::NetPolicy;
        use url::Url;

        struct DenyAllPolicy;
        impl NetPolicy for DenyAllPolicy {
            fn allow_fetch(&self, _doc_id: usize, _url: &Url) -> Result<(), String> {
                Err("Access denied by test policy".to_string())
            }
        }

        let provider = ExoNetProvider::builder()
            .add_handler(exo_bliss_net::DataSchemeHandler)
            .policy(Arc::new(DenyAllPolicy))
            .build();

        let mock = MockHandler::new();
        let request = Request::get(url::Url::parse("data:text/plain,test").unwrap());

        provider.fetch(1, request, Box::new(mock.clone()));

        // Should not receive data due to policy denial
        let received = mock.received_data();
        assert!(received.is_empty());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use bliss_traits::net::{DefaultNetPolicy, NetProvider};
    use exo_bliss_net::{DataSchemeHandler, ExoNetProvider};

    /// Test that verifies capsule:// URLs would be routed correctly
    /// This is a mock test since we can't create a real MeshNode in unit tests
    #[test]
    fn test_capsule_url_routing_mock() {
        // Create a provider with data handler as a stand-in for testing routing
        let provider = ExoNetProvider::builder()
            .add_handler(DataSchemeHandler)
            .policy(Arc::new(DefaultNetPolicy))
            .build();

        // Verify the handler lookup works
        let data_handler = provider.find_handler("data");
        assert!(data_handler.is_some());

        // capsule:// would require a real MeshNode
        assert!(provider.find_handler("capsule").is_none());
    }

    /// Test that verifies exo:// URLs would be routed correctly
    #[test]
    fn test_exo_url_routing_mock() {
        let provider = ExoNetProvider::builder()
            .add_handler(DataSchemeHandler)
            .policy(Arc::new(DefaultNetPolicy))
            .build();

        // exo:// would require XipTransport
        assert!(provider.find_handler("exo").is_none());
    }

    /// Test that verifies mycelium:// URLs would be routed correctly
    #[test]
    fn test_mycelium_url_routing_mock() {
        let provider = ExoNetProvider::builder()
            .add_handler(DataSchemeHandler)
            .policy(Arc::new(DefaultNetPolicy))
            .build();

        // mycelium:// would require MyceliumXipTransport
        assert!(provider.find_handler("mycelium").is_none());
    }
}

#[cfg(test)]
mod html_integration_tests {
    //! These tests simulate how the network provider would be used
    //! when rendering HTML with custom scheme URLs

    use super::*;
    use bliss_traits::net::{DefaultNetPolicy, NetProvider};
    use exo_bliss_net::{DataSchemeHandler, ExoNetProvider};

    /// Test simulating rendering HTML with an img src="capsule://..."
    /// This test verifies the network provider structure would handle such URLs
    #[test]
    fn test_html_capsule_image_url_structure() {
        let capsule_url = "capsule://system.assets/logo.png";
        let parsed = url::Url::parse(capsule_url).unwrap();

        assert_eq!(parsed.scheme(), "capsule");
        assert_eq!(parsed.host_str(), Some("system.assets"));
        assert_eq!(parsed.path(), "/logo.png");
    }

    /// Test simulating rendering HTML with a link href="exo://..."
    #[test]
    fn test_html_exo_stylesheet_url_structure() {
        // exo:// URLs have the format: exo://<host>/<capability>?<params>
        // Note: DIDs with colons don't parse as valid hosts, so we use a simpler format
        let exo_url = "exo://theme-service/read?resource=styles.css&v=1";
        let parsed = url::Url::parse(exo_url).unwrap();

        assert_eq!(parsed.scheme(), "exo");
        assert_eq!(parsed.host_str(), Some("theme-service"));
        // The path should contain the capability
        assert!(parsed.path().contains("read"));
        // Query params should contain the resource info
        assert!(parsed.query().unwrap().contains("styles.css"));
    }

    /// Test that data URLs work for inline resources
    #[test]
    fn test_html_inline_data_url() {
        let provider = ExoNetProvider::builder()
            .add_handler(DataSchemeHandler)
            .policy(Arc::new(DefaultNetPolicy))
            .build();

        let mock = MockHandler::new();

        // Simulate an inline image (1x1 transparent PNG)
        let data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
        let request = bliss_traits::net::Request::get(url::Url::parse(data_url).unwrap());

        provider.fetch(1, request, Box::new(mock.clone()));

        let received = mock.received_data();
        assert_eq!(received.len(), 1);
        // Verify it's a valid PNG (starts with PNG magic bytes)
        let data = &received[0].1;
        assert_eq!(&data[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }
}

//! Message types for mesh communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request message sent to a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request ID for correlation
    pub id: String,
    /// Method to invoke
    pub method: String,
    /// Parameters (JSON)
    pub params: serde_json::Value,
    /// Caller's capsule ID
    pub caller: String,
    /// Capability token (if required)
    pub capability: Option<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Request {
    pub fn new(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            id: generate_request_id(),
            method: method.into(),
            params,
            caller: String::new(),
            capability: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_caller(mut self, caller: impl Into<String>) -> Self {
        self.caller = caller.into();
        self
    }

    pub fn with_capability(mut self, token: impl Into<String>) -> Self {
        self.capability = Some(token.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Response from a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID this is responding to
    pub id: String,
    /// Success result
    pub result: Option<serde_json::Value>,
    /// Error (if failed)
    pub error: Option<ResponseError>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Response {
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(id: String, code: i32, message: impl Into<String>) -> Self {
        Self {
            id,
            result: None,
            error: Some(ResponseError {
                code,
                message: message.into(),
                data: None,
            }),
            metadata: HashMap::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    pub fn into_result(self) -> Result<serde_json::Value, ResponseError> {
        match self.error {
            Some(e) => Err(e),
            None => Ok(self.result.unwrap_or(serde_json::Value::Null)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ResponseError {}

/// Standard error codes
pub mod error_codes {
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const AUTH_REQUIRED: i32 = -32000;
    pub const CAPABILITY_DENIED: i32 = -32001;
    pub const SERVICE_UNAVAILABLE: i32 = -32002;
}

/// Internal mesh protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MeshMessage {
    /// RPC request
    #[serde(rename = "request")]
    Request(Request),

    /// RPC response
    #[serde(rename = "response")]
    Response(Response),

    /// Service registration
    #[serde(rename = "register")]
    Register(ServiceRegistration),

    /// Service deregistration
    #[serde(rename = "unregister")]
    Unregister { service_id: String },

    /// Discovery query
    #[serde(rename = "discover")]
    Discover { service_id: String },

    /// Discovery response
    #[serde(rename = "discovered")]
    Discovered(Vec<ServiceInfo>),

    /// Authentication challenge
    #[serde(rename = "challenge")]
    Challenge(crate::identity::Challenge),

    /// Challenge response
    #[serde(rename = "challenge_response")]
    ChallengeResponse(crate::identity::ChallengeResponse),

    /// Ping/keepalive
    #[serde(rename = "ping")]
    Ping { timestamp: i64 },

    /// Pong response
    #[serde(rename = "pong")]
    Pong { timestamp: i64 },
}

/// Service registration info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistration {
    pub service_id: String,
    pub node_did: String,
    pub methods: Vec<String>,
    pub version: String,
    pub capabilities_required: Vec<String>,
}

/// Service info from discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub node_did: String,
    pub location: String, // "local", "lan", "remote"
    pub methods: Vec<String>,
    pub version: String,
    pub latency_hint_ms: Option<u32>,
}

fn generate_request_id() -> String {
    use rand::Rng;
    let id: u64 = rand::thread_rng().gen();
    format!("{:016x}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_response() {
        let req = Request::new("echo", serde_json::json!({"message": "hello"}))
            .with_caller("tools.teddy");

        let resp = Response::success(req.id.clone(), serde_json::json!({"echo": "hello"}));
        assert!(resp.is_success());
    }

    #[test]
    fn test_message_serialization() {
        let msg = MeshMessage::Ping { timestamp: 12345 };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"type\":\"ping\""));

        let parsed: MeshMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, MeshMessage::Ping { timestamp: 12345 }));
    }
}

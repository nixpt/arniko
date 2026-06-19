//! Local transport via Unix sockets

use crate::address::ServiceAddress;
use crate::message::{MeshMessage, Request, Response};
use crate::{MeshError, Result};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// Local IPC client for connecting to services on the same machine
pub struct LocalClient {
    stream: UnixStream,
}

impl LocalClient {
    /// Connect to a local service by socket path
    pub async fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path).await.map_err(|e| {
            MeshError::ConnectionFailed(format!("Failed to connect to {}: {}", socket_path, e))
        })?;

        Ok(Self { stream })
    }

    /// Connect to a service by address
    pub async fn connect_to(address: &ServiceAddress) -> Result<Self> {
        Self::connect(&address.local_socket_path()).await
    }

    /// Send a request and wait for response
    pub async fn call(&mut self, request: Request) -> Result<Response> {
        let msg = MeshMessage::Request(request);
        let json = serde_json::to_string(&msg)?;

        self.stream.write_all(json.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: MeshMessage = serde_json::from_str(&line)?;

        match response {
            MeshMessage::Response(r) => Ok(r),
            _ => Err(MeshError::Transport("Unexpected response type".to_string())),
        }
    }
}

/// Local IPC server for hosting a service
pub struct LocalServer {
    listener: UnixListener,
    socket_path: String,
}

impl LocalServer {
    /// Create a new server at the given socket path
    pub async fn bind(socket_path: &str) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(socket_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Remove existing socket
        tokio::fs::remove_file(socket_path).await.ok();

        let listener = UnixListener::bind(socket_path)?;

        // Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o666)).ok();
        }

        Ok(Self {
            listener,
            socket_path: socket_path.to_string(),
        })
    }

    /// Accept connections and handle them with the provided handler
    pub async fn serve<F, Fut>(self, handler: F) -> Result<()>
    where
        F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
        Fut: std::future::Future<Output = Response> + Send,
    {
        tracing::info!("Local server listening on {}", self.socket_path);

        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let handler = handler.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, handler).await {
                            tracing::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Accept error: {}", e);
                }
            }
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &str {
        &self.socket_path
    }
}

async fn handle_connection<F, Fut>(stream: UnixStream, handler: F) -> Result<()>
where
    F: Fn(Request) -> Fut,
    Fut: std::future::Future<Output = Response>,
{
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line).await?;

        if bytes == 0 {
            break; // Connection closed
        }

        let message: MeshMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                let error_resp = Response::error(
                    String::new(),
                    crate::message::error_codes::INVALID_REQUEST,
                    format!("Invalid message: {}", e),
                );
                let json = serde_json::to_string(&MeshMessage::Response(error_resp))?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                continue;
            }
        };

        match message {
            MeshMessage::Request(req) => {
                let response = handler(req).await;
                let json = serde_json::to_string(&MeshMessage::Response(response))?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
            MeshMessage::Ping { timestamp } => {
                let pong = MeshMessage::Pong { timestamp };
                let json = serde_json::to_string(&pong)?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
            _ => {
                let error_resp = Response::error(
                    String::new(),
                    crate::message::error_codes::INVALID_REQUEST,
                    "Unexpected message type",
                );
                let json = serde_json::to_string(&MeshMessage::Response(error_resp))?;
                writer.write_all(json.as_bytes()).await?;
                writer.write_all(b"\n").await?;
            }
        }
    }

    Ok(())
}

impl Drop for LocalServer {
    fn drop(&mut self) {
        std::fs::remove_file(&self.socket_path).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_roundtrip() {
        let socket_path = "/tmp/exo-mesh-test.sock";

        // Start server
        let server = LocalServer::bind(socket_path).await.unwrap();

        tokio::spawn(async move {
            server
                .serve(|req| async move {
                    Response::success(req.id, serde_json::json!({"echo": req.params}))
                })
                .await
                .unwrap();
        });

        // Give server time to start
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Connect client
        let mut client = LocalClient::connect(socket_path).await.unwrap();

        let request = Request::new("test", serde_json::json!({"hello": "world"}));
        let response = client.call(request).await.unwrap();

        assert!(response.is_success());

        // Cleanup
        std::fs::remove_file(socket_path).ok();
    }
}

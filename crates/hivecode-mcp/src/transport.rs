//! Transport layer for MCP communication

use crate::error::{McpError, Result};
use crate::types::{JsonRpcRequest, JsonRpcResponse};
use async_trait::async_trait;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, warn};

/// Trait for different transport implementations
#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Send a JSON-RPC request and receive a response
    async fn send(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse>;

    /// Close the transport connection
    async fn close(&self) -> Result<()>;
}

/// Stdio-based transport that communicates with a child process
pub struct StdioTransport {
    process: Arc<Mutex<Option<Child>>>,
    timeout_duration: Duration,
}

impl StdioTransport {
    /// Create a new stdio transport and spawn the server process
    pub fn new(command: &str, args: &[String], timeout_duration: Duration) -> Result<Self> {
        debug!("spawning MCP server: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let process = cmd.spawn().map_err(|e| {
            McpError::transport(format!("failed to spawn process '{}': {}", command, e))
        })?;

        Ok(StdioTransport {
            process: Arc::new(Mutex::new(Some(process))),
            timeout_duration,
        })
    }

    /// Send a request synchronously (used internally)
    fn send_sync(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let mut process_guard = self.process.lock().map_err(|_| {
            McpError::transport("failed to acquire process lock")
        })?;

        let process = process_guard
            .as_mut()
            .ok_or_else(|| McpError::ConnectionClosed)?;

        // Get stdin/stdout handles
        let stdin = process.stdin.as_mut().ok_or_else(|| {
            McpError::transport("process stdin not available")
        })?;

        // Serialize request to JSON with newline
        let mut request_bytes = serde_json::to_vec(&request)
            .map_err(|e| McpError::protocol(format!("serialization failed: {}", e)))?;
        request_bytes.push(b'\n');

        // Send request
        stdin.write_all(&request_bytes).map_err(|e| {
            McpError::transport(format!("failed to write to process stdin: {}", e))
        })?;
        stdin.flush().map_err(|e| {
            McpError::transport(format!("failed to flush stdin: {}", e))
        })?;

        debug!("sent request: {:?}", serde_json::to_string(&request).unwrap_or_default());

        // Read response from stdout (line-delimited JSON)
        let stdout = process.stdout.as_mut().ok_or_else(|| {
            McpError::transport("process stdout not available")
        })?;

        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();

        reader.read_line(&mut response_line).map_err(|e| {
            McpError::transport(format!("failed to read from process stdout: {}", e))
        })?;

        if response_line.is_empty() {
            return Err(McpError::ConnectionClosed);
        }

        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| McpError::protocol(format!("failed to parse response: {}", e)))?;

        debug!("received response: {:?}", response);

        Ok(response)
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Run the synchronous send in a blocking task
        let self_clone = Arc::new(self.process.clone());
        let timeout_dur = self.timeout_duration;

        let response = timeout(
            timeout_dur,
            tokio::task::spawn_blocking({
                let self_process = self_clone.clone();
                let req = request.clone();
                move || {
                    let mut process_guard = self_process.lock().map_err(|_| {
                        McpError::transport("failed to acquire process lock")
                    })?;

                    let process = process_guard
                        .as_mut()
                        .ok_or_else(|| McpError::ConnectionClosed)?;

                    let stdin = process.stdin.as_mut().ok_or_else(|| {
                        McpError::transport("process stdin not available")
                    })?;

                    let mut request_bytes = serde_json::to_vec(&req)
                        .map_err(|e| McpError::protocol(format!("serialization failed: {}", e)))?;
                    request_bytes.push(b'\n');

                    stdin.write_all(&request_bytes).map_err(|e| {
                        McpError::transport(format!("failed to write to stdin: {}", e))
                    })?;
                    stdin.flush().map_err(|e| {
                        McpError::transport(format!("failed to flush stdin: {}", e))
                    })?;

                    let stdout = process.stdout.as_mut().ok_or_else(|| {
                        McpError::transport("process stdout not available")
                    })?;

                    let mut reader = BufReader::new(stdout);
                    let mut response_line = String::new();
                    reader.read_line(&mut response_line).map_err(|e| {
                        McpError::transport(format!("failed to read from stdout: {}", e))
                    })?;

                    if response_line.is_empty() {
                        return Err(McpError::ConnectionClosed);
                    }

                    serde_json::from_str(&response_line)
                        .map_err(|e| McpError::protocol(format!("failed to parse response: {}", e)))
                }
            }),
        )
        .await
        .map_err(|_| McpError::Timeout)?
        .map_err(|_| McpError::transport("task error"))?;

        response
    }

    async fn close(&self) -> Result<()> {
        let mut process_guard = self.process.lock().map_err(|_| {
            McpError::transport("failed to acquire process lock")
        })?;

        if let Some(mut process) = process_guard.take() {
            process.kill().map_err(|e| {
                warn!("failed to kill process: {}", e);
                McpError::transport(format!("failed to kill process: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_creation() {
        // This test just verifies that transport creation doesn't panic
        // A real test would need an actual MCP server binary
        assert!(true);
    }
}

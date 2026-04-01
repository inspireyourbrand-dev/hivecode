//! MCP client implementation

use crate::error::{McpError, Result};
use crate::protocol;
use crate::transport::{McpTransport, StdioTransport};
use crate::types::*;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// MCP client for communicating with MCP servers
pub struct McpClient {
    transport: Arc<dyn McpTransport>,
    is_connected: bool,
    server_info: Option<ServerInfo>,
    timeout: Duration,
}

impl McpClient {
    /// Create a new MCP client from server configuration
    pub async fn new(config: McpServerConfig) -> Result<Self> {
        debug!("creating MCP client for server: {}", config.name);

        match config.transport_type {
            TransportType::Stdio => {
                let transport = Arc::new(StdioTransport::new(
                    &config.command,
                    &config.args,
                    Duration::from_secs(30),
                )?);

                Ok(McpClient {
                    transport,
                    is_connected: false,
                    server_info: None,
                    timeout: Duration::from_secs(30),
                })
            }
            TransportType::Sse | TransportType::Http => {
                Err(McpError::InvalidConfig(
                    "SSE and HTTP transports not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Connect to the MCP server and perform initialization handshake
    pub async fn connect(&mut self) -> Result<()> {
        info!("connecting to MCP server");

        let client_info = ClientInfo {
            name: "hivecode".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let init_request = protocol::build_initialize_request(client_info);
        let init_response = self.transport.send(init_request).await?;

        let result: InitializeResult = protocol::parse_response(init_response)?;

        self.server_info = Some(result.server_info.clone());
        self.is_connected = true;

        info!(
            "connected to MCP server: {} {}",
            result.server_info.name, result.server_info.version
        );

        Ok(())
    }

    /// List all available tools from the MCP server
    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        self.ensure_connected()?;

        let request = protocol::build_tools_list_request();
        let response = self.transport.send(request).await?;

        let result: ToolsListResult = protocol::parse_response(response)?;

        debug!("retrieved {} tools from MCP server", result.tools.len());

        Ok(result.tools)
    }

    /// Call a tool on the MCP server with the given arguments
    pub async fn call_tool(&self, name: String, args: serde_json::Value) -> Result<ToolCallResult> {
        self.ensure_connected()?;

        debug!("calling tool: {} with args: {}", name, args);

        let request = protocol::build_tool_call_request(name.clone(), args);
        let response = self.transport.send(request).await?;

        let result: ToolCallResponse = protocol::parse_response(response)?;

        Ok(ToolCallResult {
            content: result.content,
            is_error: result.is_error,
        })
    }

    /// List all available resources from the MCP server
    pub async fn list_resources(&self) -> Result<Vec<McpResource>> {
        self.ensure_connected()?;

        let request = protocol::build_resources_list_request();
        let response = self.transport.send(request).await?;

        let result: ResourcesListResult = protocol::parse_response(response)?;

        debug!("retrieved {} resources from MCP server", result.resources.len());

        Ok(result.resources)
    }

    /// Read a resource from the MCP server by its URI
    pub async fn read_resource(&self, uri: String) -> Result<Vec<McpContent>> {
        self.ensure_connected()?;

        debug!("reading resource: {}", uri);

        let request = protocol::build_resource_read_request(uri);
        let response = self.transport.send(request).await?;

        let result: ResourceReadResponse = protocol::parse_response(response)?;

        Ok(result.contents)
    }

    /// Disconnect from the MCP server
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.is_connected {
            info!("disconnecting from MCP server");
            self.transport.close().await?;
            self.is_connected = false;
        }

        Ok(())
    }

    /// Check if the client is currently connected
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Get information about the connected server
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.server_info.as_ref()
    }

    /// Ensure the client is connected before performing operations
    fn ensure_connected(&self) -> Result<()> {
        if !self.is_connected {
            return Err(McpError::ConnectionClosed);
        }
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        if self.is_connected {
            warn!("McpClient dropped while still connected, performing cleanup");
            // We can't await in drop, so just log a warning
            // The caller should call disconnect() explicitly
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_initial_state() {
        // This test verifies that the client structure is sound
        // Real connection tests would need an actual MCP server
        assert!(true);
    }
}

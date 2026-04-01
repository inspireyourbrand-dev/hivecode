//! Core types for MCP protocol implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Transport type for MCP server communication
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    /// Standard input/output process transport
    Stdio,
    /// Server-Sent Events HTTP transport
    Sse,
    /// Standard HTTP transport
    Http,
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default = "default_transport_type")]
    pub transport_type: TransportType,
}

fn default_transport_type() -> TransportType {
    TransportType::Stdio
}

/// Tool definition from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Resource definition from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

/// Prompt template from MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Argument definition for a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Content variant for resource and tool responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpContent {
    Text(String),
    Image { data: String, mime_type: String },
}

/// Result of tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<McpContent>,
    pub is_error: bool,
}

/// Client information for initialize request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// Server information from initialize response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Initialize request parameters
#[derive(Debug, Serialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: serde_json::Value,
    pub client_info: ClientInfo,
}

/// Initialize response result
#[derive(Debug, Deserialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: serde_json::Value,
    pub server_info: ServerInfo,
}

/// Tools list response
#[derive(Debug, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<McpTool>,
}

/// Tool call request parameters
#[derive(Debug, Serialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Tool call response result
#[derive(Debug, Deserialize)]
pub struct ToolCallResponse {
    pub content: Vec<McpContent>,
    pub is_error: bool,
}

/// Resources list response
#[derive(Debug, Deserialize)]
pub struct ResourcesListResult {
    pub resources: Vec<McpResource>,
}

/// Resource read request parameters
#[derive(Debug, Serialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

/// Resource read response result
#[derive(Debug, Deserialize)]
pub struct ResourceReadResponse {
    pub contents: Vec<McpContent>,
}

/// Current MCP protocol version
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

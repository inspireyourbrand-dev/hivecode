//! Error types for MCP client operations

use thiserror::Error;

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, McpError>;

/// Errors that can occur during MCP client operations
#[derive(Error, Debug)]
pub enum McpError {
    #[error("transport error: {0}")]
    TransportError(String),

    #[error("protocol error: {0}")]
    ProtocolError(String),

    #[error("server error: {0}")]
    ServerError(String),

    #[error("timeout")]
    Timeout,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("tool not found: {0}")]
    ToolNotFound(String),

    #[error("resource not found: {0}")]
    ResourceNotFound(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("internal error: {0}")]
    InternalError(String),
}

impl McpError {
    /// Create a transport error
    pub fn transport(msg: impl Into<String>) -> Self {
        Self::TransportError(msg.into())
    }

    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::ProtocolError(msg.into())
    }

    /// Create a server error
    pub fn server(msg: impl Into<String>) -> Self {
        Self::ServerError(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}

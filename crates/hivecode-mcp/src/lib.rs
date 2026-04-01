//! HiveCode MCP (Model Context Protocol) integration
//!
//! This crate provides client implementations for communicating with MCP servers,
//! enabling HiveCode to access tools, resources, and prompts from external services.

pub mod client;
pub mod config;
pub mod error;
pub mod protocol;
pub mod transport;
pub mod types;

pub use client::McpClient;
pub use config::parse_mcp_config;
pub use error::{McpError, Result};
pub use types::{McpServerConfig, McpTool};

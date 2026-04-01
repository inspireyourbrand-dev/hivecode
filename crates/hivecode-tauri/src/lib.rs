//! HiveCode Tauri desktop application
//!
//! This crate provides the Tauri-based desktop UI for HiveCode, a Rust+Tauri
//! AI coding assistant with integrated MCP support and security controls.

pub mod agent_commands;
pub mod auth_commands;
pub mod commands;
pub mod compact_commands;
pub mod context_commands;
pub mod events;
pub mod history_commands;
pub mod image_commands;
pub mod memory_commands;
pub mod notification_commands;
pub mod plan_commands;
pub mod query_engine;
pub mod state;
pub mod plugin_commands;
pub mod update_commands;
pub mod hooks_commands;
pub mod branch_commands;
pub mod thinking_commands;
pub mod offline_commands;
pub mod project_commands;
pub mod replay_commands;
pub mod cost_optimizer_commands;
pub mod diff_commands;

pub use agent_commands::*;
pub use auth_commands::*;
pub use context_commands::*;
pub use plan_commands::*;
pub use plugin_commands::*;
pub use update_commands::*;
pub use state::TauriAppState;

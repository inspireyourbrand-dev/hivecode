//! CLI configuration utilities

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// CLI-specific configuration (separate from core HiveCode config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Default model for CLI use
    pub default_model: String,

    /// Default provider
    pub default_provider: String,

    /// Whether to stream responses by default
    pub stream_by_default: bool,

    /// Theme for terminal (light, dark, auto)
    pub theme: String,

    /// Pagination for long responses
    pub pagination_enabled: bool,

    /// Lines per page
    pub lines_per_page: u32,

    /// Whether to save history
    pub save_history: bool,

    /// History file location
    pub history_file: Option<PathBuf>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            default_model: "claude-sonnet-4".to_string(),
            default_provider: "anthropic".to_string(),
            stream_by_default: true,
            theme: "auto".to_string(),
            pagination_enabled: true,
            lines_per_page: 25,
            save_history: true,
            history_file: None,
        }
    }
}

impl CliConfig {
    /// Get the history file path, creating directory if needed
    pub fn get_history_file(&self) -> PathBuf {
        if let Some(path) = &self.history_file {
            path.clone()
        } else {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".hivecode").join("cli_history")
        }
    }
}

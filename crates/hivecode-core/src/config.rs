//! Configuration system for HiveCode
//!
//! Loads configuration from ~/.hivecode/config.toml with support for:
//! - Environment variable expansion in config values
//! - Project-level configuration overrides
//! - Provider and tool definitions
//! - Security and permission settings

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Main HiveCode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveConfig {
    /// Application metadata
    #[serde(default)]
    pub app: AppConfig,

    /// Provider configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

    /// Tool definitions
    #[serde(default)]
    pub tools: HashMap<String, ToolConfig>,

    /// Security and permission settings
    #[serde(default)]
    pub security: SecurityConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UiConfig,

    /// Custom settings
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// Application-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,

    /// Application version
    #[serde(default = "default_version")]
    pub version: String,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Default working directory
    #[serde(default)]
    pub default_work_dir: Option<String>,

    /// Maximum context window size in tokens
    #[serde(default = "default_max_context")]
    pub max_context_tokens: u64,

    /// Custom app settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: default_version(),
            log_level: default_log_level(),
            default_work_dir: None,
            max_context_tokens: default_max_context(),
            settings: HashMap::new(),
        }
    }
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Whether this provider is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// API key or authentication token (supports ${VAR} expansion)
    pub api_key: Option<String>,

    /// API base URL
    pub base_url: Option<String>,

    /// Default model to use
    pub default_model: Option<String>,

    /// List of available models
    #[serde(default)]
    pub models: Vec<String>,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Rate limit configuration
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,

    /// Custom provider settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Human-readable name for the tool
    pub name: String,

    /// Description of what this tool does
    pub description: String,

    /// Whether this tool is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Tool category
    #[serde(default)]
    pub category: String,

    /// Input schema as JSON Schema
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,

    /// Custom tool settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

/// Security and permission settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// File access permissions
    #[serde(default)]
    pub file_access: FileAccessConfig,

    /// Network access permissions
    #[serde(default)]
    pub network_access: NetworkAccessConfig,

    /// Tool execution permissions
    #[serde(default)]
    pub tool_permissions: HashMap<String, bool>,

    /// Maximum file size to read (in bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// Allowed file extensions (empty = all allowed)
    #[serde(default)]
    pub allowed_extensions: Vec<String>,

    /// Denied file extensions
    #[serde(default)]
    pub denied_extensions: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            file_access: FileAccessConfig::default(),
            network_access: NetworkAccessConfig::default(),
            tool_permissions: HashMap::new(),
            max_file_size: default_max_file_size(),
            allowed_extensions: Vec::new(),
            denied_extensions: Vec::new(),
        }
    }
}

/// File access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessConfig {
    /// Whether file reading is allowed
    #[serde(default = "default_enabled")]
    pub read_enabled: bool,

    /// Whether file writing is allowed
    #[serde(default = "default_enabled")]
    pub write_enabled: bool,

    /// Allowed directories (empty = all allowed)
    #[serde(default)]
    pub allowed_dirs: Vec<String>,

    /// Denied directories
    #[serde(default)]
    pub denied_dirs: Vec<String>,
}

impl Default for FileAccessConfig {
    fn default() -> Self {
        Self {
            read_enabled: true,
            write_enabled: true,
            allowed_dirs: Vec::new(),
            denied_dirs: Vec::new(),
        }
    }
}

/// Network access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAccessConfig {
    /// Whether network access is allowed
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Allowed domains (empty = all allowed)
    #[serde(default)]
    pub allowed_domains: Vec<String>,

    /// Denied domains
    #[serde(default)]
    pub denied_domains: Vec<String>,

    /// Maximum request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

impl Default for NetworkAccessConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_domains: Vec::new(),
            denied_domains: Vec::new(),
            timeout_secs: default_timeout(),
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme (light, dark, auto)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: u32,

    /// Whether to enable animations
    #[serde(default = "default_animations")]
    pub animations_enabled: bool,

    /// Custom UI settings
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font_size: default_font_size(),
            animations_enabled: default_animations(),
            settings: HashMap::new(),
        }
    }
}

impl HiveConfig {
    /// Load configuration from the default location (~/.hivecode/config.toml)
    pub fn load() -> Result<Self> {
        let path = Self::default_config_path()?;
        Self::load_from(&path)
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        if path.exists() {
            let contents = fs::read_to_string(path)
                .map_err(|e| HiveCodeError::ConfigFileError(e))?;
            let config: HiveConfig = toml::from_str(&contents)
                .map_err(|e| HiveCodeError::TomlError(e))?;
            Ok(config.expand_env_vars()?)
        } else {
            Ok(Self::default())
        }
    }

    /// Get the default configuration file path
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| HiveCodeError::ConfigError("Cannot determine home directory".to_string()))?;
        Ok(home.join(".hivecode").join("config.toml"))
    }

    /// Expand environment variables in configuration values
    fn expand_env_vars(mut self) -> Result<Self> {
        // Expand provider API keys
        for provider in self.providers.values_mut() {
            if let Some(ref mut api_key) = provider.api_key {
                *api_key = Self::expand_value(api_key)?;
            }
        }
        Ok(self)
    }

    /// Expand a single value, replacing ${VAR} with environment variables
    fn expand_value(value: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = value.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                let mut var_name = String::new();
                while let Some(c) = chars.next() {
                    if c == '}' {
                        break;
                    }
                    var_name.push(c);
                }

                let var_value = std::env::var(&var_name)
                    .map_err(|_| HiveCodeError::EnvError(
                        format!("Environment variable '{}' not found", var_name)
                    ))?;
                result.push_str(&var_value);
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| HiveCodeError::Internal(e.to_string()))?;
        fs::write(path, contents)
            .map_err(|e| HiveCodeError::ConfigFileError(e))?;
        Ok(())
    }

    /// Get a provider configuration by ID
    pub fn get_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.get(id)
    }

    /// Get a tool configuration by ID
    pub fn get_tool(&self, id: &str) -> Option<&ToolConfig> {
        self.tools.get(id)
    }
}

impl Default for HiveConfig {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            providers: HashMap::new(),
            tools: HashMap::new(),
            security: SecurityConfig::default(),
            ui: UiConfig::default(),
            custom: HashMap::new(),
        }
    }
}

// Default value functions
fn default_app_name() -> String {
    "HiveCode".to_string()
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_context() -> u64 {
    8192
}

fn default_enabled() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

fn default_max_file_size() -> u64 {
    100 * 1024 * 1024 // 100 MB
}

fn default_theme() -> String {
    "auto".to_string()
}

fn default_font_size() -> u32 {
    14
}

fn default_animations() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = HiveConfig::default();
        assert_eq!(config.app.name, "HiveCode");
        assert_eq!(config.app.max_context_tokens, 8192);
    }

    #[test]
    fn test_expand_env_var() {
        std::env::set_var("TEST_VAR", "test_value");
        let expanded = HiveConfig::expand_value("prefix_${TEST_VAR}_suffix").unwrap();
        assert_eq!(expanded, "prefix_test_value_suffix");
    }

    #[test]
    fn test_expand_missing_env_var() {
        let result = HiveConfig::expand_value("${NONEXISTENT_VAR}");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = HiveConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let restored: HiveConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(restored.app.name, config.app.name);
    }

    #[test]
    fn test_provider_config() {
        let mut config = HiveConfig::default();
        let provider = ProviderConfig {
            enabled: true,
            api_key: Some("test_key".to_string()),
            base_url: Some("https://api.example.com".to_string()),
            default_model: Some("model-1".to_string()),
            models: vec!["model-1".to_string(), "model-2".to_string()],
            timeout_secs: 30,
            rate_limit: None,
            settings: HashMap::new(),
        };
        config.providers.insert("test".to_string(), provider);

        let retrieved = config.get_provider("test");
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().enabled);
    }

    #[test]
    fn test_security_config() {
        let config = HiveConfig::default();
        assert!(config.security.file_access.read_enabled);
        assert!(config.security.file_access.write_enabled);
        assert!(config.security.network_access.enabled);
    }
}

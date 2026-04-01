//! Integration tests for HiveCode core configuration system
//!
//! Tests configuration loading, defaults, serialization/deserialization,
//! and environment variable expansion.

use hivecode_core::config::{
    AppConfig, FileAccessConfig, HiveConfig, NetworkAccessConfig, ProviderConfig,
    RateLimitConfig, SecurityConfig, ToolConfig, UiConfig,
};
use std::collections::HashMap;
use tempfile::NamedTempFile;

#[test]
fn test_default_app_config() {
    let config = AppConfig::default();
    assert_eq!(config.name, "HiveCode");
    assert_eq!(config.version, "0.1.0");
    assert_eq!(config.log_level, "info");
    assert_eq!(config.max_context_tokens, 8192);
    assert_eq!(config.default_work_dir, None);
    assert!(config.settings.is_empty());
}

#[test]
fn test_default_security_config() {
    let config = SecurityConfig::default();
    assert!(config.file_access.read_enabled);
    assert!(config.file_access.write_enabled);
    assert!(config.network_access.enabled);
    assert_eq!(config.max_file_size, 100 * 1024 * 1024); // 100 MB
    assert!(config.allowed_extensions.is_empty());
    assert!(config.denied_extensions.is_empty());
    assert!(config.tool_permissions.is_empty());
}

#[test]
fn test_default_file_access_config() {
    let config = FileAccessConfig::default();
    assert!(config.read_enabled);
    assert!(config.write_enabled);
    assert!(config.allowed_dirs.is_empty());
    assert!(config.denied_dirs.is_empty());
}

#[test]
fn test_default_network_access_config() {
    let config = NetworkAccessConfig::default();
    assert!(config.enabled);
    assert!(config.allowed_domains.is_empty());
    assert!(config.denied_domains.is_empty());
    assert_eq!(config.timeout_secs, 30);
}

#[test]
fn test_default_ui_config() {
    let config = UiConfig::default();
    assert_eq!(config.theme, "auto");
    assert_eq!(config.font_size, 14);
    assert!(config.animations_enabled);
    assert!(config.settings.is_empty());
}

#[test]
fn test_default_hive_config() {
    let config = HiveConfig::default();
    assert_eq!(config.app.name, "HiveCode");
    assert_eq!(config.app.max_context_tokens, 8192);
    assert!(config.providers.is_empty());
    assert!(config.tools.is_empty());
    assert!(config.custom.is_empty());
}

#[test]
fn test_provider_config_creation() {
    let provider = ProviderConfig {
        enabled: true,
        api_key: Some("test_api_key".to_string()),
        base_url: Some("https://api.example.com".to_string()),
        default_model: Some("gpt-4".to_string()),
        models: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        timeout_secs: 60,
        rate_limit: None,
        settings: HashMap::new(),
    };

    assert!(provider.enabled);
    assert_eq!(provider.api_key, Some("test_api_key".to_string()));
    assert_eq!(provider.default_model, Some("gpt-4".to_string()));
    assert_eq!(provider.models.len(), 2);
    assert_eq!(provider.timeout_secs, 60);
}

#[test]
fn test_provider_config_with_rate_limit() {
    let rate_limit = RateLimitConfig {
        requests_per_minute: Some(100),
        tokens_per_minute: Some(90000),
    };

    let provider = ProviderConfig {
        enabled: true,
        api_key: None,
        base_url: None,
        default_model: None,
        models: Vec::new(),
        timeout_secs: 30,
        rate_limit: Some(rate_limit.clone()),
        settings: HashMap::new(),
    };

    assert!(provider.rate_limit.is_some());
    let limit = provider.rate_limit.unwrap();
    assert_eq!(limit.requests_per_minute, Some(100));
    assert_eq!(limit.tokens_per_minute, Some(90000));
}

#[test]
fn test_tool_config_creation() {
    let tool = ToolConfig {
        name: "file_reader".to_string(),
        description: "Reads files from disk".to_string(),
        enabled: true,
        category: "file_ops".to_string(),
        input_schema: None,
        settings: HashMap::new(),
    };

    assert_eq!(tool.name, "file_reader");
    assert_eq!(tool.description, "Reads files from disk");
    assert!(tool.enabled);
    assert_eq!(tool.category, "file_ops");
}

#[test]
fn test_tool_config_with_schema() {
    let mut schema = HashMap::new();
    schema.insert("type".to_string(), serde_json::json!("object"));

    let tool = ToolConfig {
        name: "command_runner".to_string(),
        description: "Executes shell commands".to_string(),
        enabled: true,
        category: "shell".to_string(),
        input_schema: Some(serde_json::json!({"type": "object"})),
        settings: HashMap::new(),
    };

    assert!(tool.input_schema.is_some());
    assert_eq!(
        tool.input_schema.unwrap().get("type").unwrap().as_str(),
        Some("object")
    );
}

#[test]
fn test_hive_config_with_providers_and_tools() {
    let mut config = HiveConfig::default();

    let provider = ProviderConfig {
        enabled: true,
        api_key: Some("key123".to_string()),
        base_url: None,
        default_model: Some("claude-3".to_string()),
        models: vec!["claude-3".to_string()],
        timeout_secs: 30,
        rate_limit: None,
        settings: HashMap::new(),
    };

    let tool = ToolConfig {
        name: "web_search".to_string(),
        description: "Searches the web".to_string(),
        enabled: true,
        category: "search".to_string(),
        input_schema: None,
        settings: HashMap::new(),
    };

    config.providers.insert("anthropic".to_string(), provider);
    config.tools.insert("web_search".to_string(), tool);

    assert!(config.get_provider("anthropic").is_some());
    assert!(config.get_tool("web_search").is_some());
    assert_eq!(config.get_provider("anthropic").unwrap().enabled, true);
}

#[test]
fn test_hive_config_serialization() {
    let original = HiveConfig::default();
    let toml_str = toml::to_string(&original).expect("Failed to serialize config");
    let restored: HiveConfig =
        toml::from_str(&toml_str).expect("Failed to deserialize config");

    assert_eq!(original.app.name, restored.app.name);
    assert_eq!(original.app.version, restored.app.version);
    assert_eq!(
        original.app.max_context_tokens,
        restored.app.max_context_tokens
    );
    assert_eq!(original.security.max_file_size, restored.security.max_file_size);
}

#[test]
fn test_app_config_with_work_dir() {
    let mut config = AppConfig::default();
    config.default_work_dir = Some("/home/user/projects".to_string());

    assert_eq!(
        config.default_work_dir,
        Some("/home/user/projects".to_string())
    );
}

#[test]
fn test_security_config_with_denied_extensions() {
    let mut config = SecurityConfig::default();
    config.denied_extensions =
        vec!["exe".to_string(), "dll".to_string(), "sh".to_string()];
    config.allowed_extensions = vec!["txt".to_string(), "json".to_string()];

    assert_eq!(config.denied_extensions.len(), 3);
    assert_eq!(config.allowed_extensions.len(), 2);
    assert!(config.denied_extensions.contains(&"exe".to_string()));
}

#[test]
fn test_security_config_with_tool_permissions() {
    let mut config = SecurityConfig::default();
    config.tool_permissions.insert("dangerous_tool".to_string(), false);
    config.tool_permissions.insert("safe_tool".to_string(), true);

    assert_eq!(
        config.tool_permissions.get("dangerous_tool"),
        Some(&false)
    );
    assert_eq!(config.tool_permissions.get("safe_tool"), Some(&true));
}

#[test]
fn test_file_access_config_with_denied_dirs() {
    let mut config = FileAccessConfig::default();
    config.denied_dirs = vec!["/etc".to_string(), "/root".to_string()];
    config.allowed_dirs = vec!["/home".to_string(), "/tmp".to_string()];

    assert_eq!(config.denied_dirs.len(), 2);
    assert_eq!(config.allowed_dirs.len(), 2);
    assert!(config.denied_dirs.contains(&"/etc".to_string()));
}

#[test]
fn test_network_access_config_with_domains() {
    let mut config = NetworkAccessConfig::default();
    config.allowed_domains =
        vec!["api.openai.com".to_string(), "api.anthropic.com".to_string()];
    config.denied_domains = vec!["malicious.com".to_string()];

    assert_eq!(config.allowed_domains.len(), 2);
    assert_eq!(config.denied_domains.len(), 1);
    assert!(config.allowed_domains.contains(&"api.anthropic.com".to_string()));
}

#[test]
fn test_provider_config_with_custom_settings() {
    let mut settings = HashMap::new();
    settings.insert(
        "max_retries".to_string(),
        serde_json::json!({"value": 3}),
    );
    settings.insert("custom_param".to_string(), serde_json::json!("param_value"));

    let provider = ProviderConfig {
        enabled: true,
        api_key: None,
        base_url: None,
        default_model: None,
        models: Vec::new(),
        timeout_secs: 30,
        rate_limit: None,
        settings,
    };

    assert_eq!(provider.settings.len(), 2);
    assert!(provider.settings.contains_key("max_retries"));
    assert!(provider.settings.contains_key("custom_param"));
}

#[test]
fn test_hive_config_custom_settings() {
    let mut config = HiveConfig::default();
    config
        .custom
        .insert("custom_key".to_string(), serde_json::json!("custom_value"));
    config.custom.insert(
        "nested".to_string(),
        serde_json::json!({"key": "value"}),
    );

    assert_eq!(config.custom.len(), 2);
    assert!(config.custom.contains_key("custom_key"));
    assert!(config.custom.contains_key("nested"));
}

#[test]
fn test_config_expand_env_var_success() {
    std::env::set_var("TEST_API_KEY", "secret_key_123");

    let toml_str = r#"
[app]
name = "HiveCode"

[providers.test]
api_key = "${TEST_API_KEY}"
"#;

    let config: HiveConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
    let expanded = config.expand_env_vars().expect("Failed to expand env vars");

    let provider = expanded.get_provider("test").expect("Provider not found");
    assert_eq!(
        provider.api_key,
        Some("secret_key_123".to_string()),
        "Environment variable should be expanded"
    );
}

#[test]
fn test_ui_config_serialization() {
    let original = UiConfig {
        theme: "dark".to_string(),
        font_size: 16,
        animations_enabled: false,
        settings: HashMap::new(),
    };

    let toml_str = toml::to_string(&original).expect("Failed to serialize");
    let restored: UiConfig = toml::from_str(&toml_str).expect("Failed to deserialize");

    assert_eq!(original.theme, restored.theme);
    assert_eq!(original.font_size, restored.font_size);
    assert_eq!(original.animations_enabled, restored.animations_enabled);
}

#[test]
fn test_provider_config_default_enabled() {
    let provider: ProviderConfig = toml::from_str("").expect("Failed to parse empty TOML");
    assert!(provider.enabled, "Provider should be enabled by default");
}

#[test]
fn test_tool_config_default_enabled() {
    let tool: ToolConfig =
        toml::from_str(r#"name = "test" description = "test tool""#)
            .expect("Failed to parse TOML");
    assert!(tool.enabled, "Tool should be enabled by default");
}

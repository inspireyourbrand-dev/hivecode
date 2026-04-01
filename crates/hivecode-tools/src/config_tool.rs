use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Global configuration storage
static CONFIG_STORAGE: std::sync::OnceLock<Arc<Mutex<HashMap<String, Value>>>> =
    std::sync::OnceLock::new();

fn get_config_store() -> Arc<Mutex<HashMap<String, Value>>> {
    CONFIG_STORAGE
        .get_or_init(|| {
            let mut defaults = HashMap::new();
            defaults.insert("timeout".to_string(), json!(120));
            defaults.insert("max_retries".to_string(), json!(3));
            defaults.insert("verbose".to_string(), json!(false));
            defaults.insert("output_format".to_string(), json!("text"));
            Arc::new(Mutex::new(defaults))
        })
        .clone()
}

/// Tool for managing configuration settings
pub struct ConfigTool;

impl ConfigTool {
    pub fn new() -> Self {
        Self
    }

    /// Validate configuration value
    fn validate_value(key: &str, value: &Value) -> Result<(), String> {
        match key {
            "timeout" => {
                if !value.is_number() {
                    return Err("timeout must be a number".to_string());
                }
                let num = value.as_u64().ok_or("timeout must be a positive integer")?;
                if num == 0 {
                    return Err("timeout must be greater than 0".to_string());
                }
            }
            "max_retries" => {
                if !value.is_number() {
                    return Err("max_retries must be a number".to_string());
                }
                let num = value.as_u64().ok_or("max_retries must be a non-negative integer")?;
                if num > 100 {
                    return Err("max_retries must be <= 100".to_string());
                }
            }
            "verbose" => {
                if !value.is_boolean() {
                    return Err("verbose must be a boolean".to_string());
                }
            }
            "output_format" => {
                if let Some(format) = value.as_str() {
                    if !["text", "json", "csv"].contains(&format) {
                        return Err("output_format must be one of: text, json, csv".to_string());
                    }
                } else {
                    return Err("output_format must be a string".to_string());
                }
            }
            _ => {
                // Allow custom keys with any JSON value
            }
        }
        Ok(())
    }
}

impl Default for ConfigTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ConfigTool {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "Manage configuration settings: read, update, list, and reset configuration values"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get", "set", "list", "reset", "reset_all"],
                    "description": "Action to perform: get (read), set (write), list (all), reset (one key), or reset_all"
                },
                "key": {
                    "type": "string",
                    "description": "Configuration key (required for get/set/reset)"
                },
                "value": {
                    "type": ["string", "number", "boolean", "object"],
                    "description": "Configuration value (required for set)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        input: Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'action' field".to_string()))?;

        let store = get_config_store();
        let mut config = store.lock().unwrap();

        match action {
            "get" => {
                let key = input
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'key' for get".to_string()))?;

                let value = config
                    .get(key)
                    .ok_or_else(|| ToolError::ExecutionFailed(format!("Config key '{}' not found", key)))?
                    .clone();

                let metadata = json!({
                    "key": key,
                    "value": value
                });

                Ok(ToolResult::success_with_metadata(
                    format!("{} = {}", key, value),
                    metadata,
                ))
            }

            "set" => {
                let key = input
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'key' for set".to_string()))?;

                let value = input
                    .get("value")
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'value' for set".to_string()))?
                    .clone();

                // Validate the value
                Self::validate_value(key, &value)
                    .map_err(|e| ToolError::InvalidInput(e))?;

                config.insert(key.to_string(), value.clone());

                Ok(ToolResult::success(format!(
                    "Config '{}' updated successfully",
                    key
                )))
            }

            "list" => {
                let config_list: Vec<(String, Value)> = config
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();

                let mut output = String::new();
                output.push_str("Current Configuration:\n");
                output.push_str("======================\n");

                for (key, value) in &config_list {
                    output.push_str(&format!("{}: {}\n", key, value));
                }

                let metadata = json!({
                    "total": config_list.len(),
                    "config": config_list
                });

                Ok(ToolResult::success_with_metadata(output, metadata))
            }

            "reset" => {
                let key = input
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'key' for reset".to_string())
                    })?;

                // Only allow resetting known defaults
                let default_value = match key {
                    "timeout" => Some(json!(120)),
                    "max_retries" => Some(json!(3)),
                    "verbose" => Some(json!(false)),
                    "output_format" => Some(json!("text")),
                    _ => None,
                };

                if let Some(default) = default_value {
                    config.insert(key.to_string(), default);
                    Ok(ToolResult::success(format!(
                        "Config '{}' reset to default",
                        key
                    )))
                } else {
                    Err(ToolError::ExecutionFailed(format!(
                        "Cannot reset custom config key '{}'",
                        key
                    )))
                }
            }

            "reset_all" => {
                let count = config.len();
                config.clear();

                // Restore defaults
                config.insert("timeout".to_string(), json!(120));
                config.insert("max_retries".to_string(), json!(3));
                config.insert("verbose".to_string(), json!(false));
                config.insert("output_format".to_string(), json!("text"));

                Ok(ToolResult::success(format!(
                    "All {} configs reset to defaults",
                    count
                )))
            }

            _ => Err(ToolError::InvalidInput(format!(
                "Unknown action: {}",
                action
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_timeout_valid() {
        assert!(ConfigTool::validate_value("timeout", &json!(60)).is_ok());
    }

    #[test]
    fn test_validate_timeout_invalid() {
        assert!(ConfigTool::validate_value("timeout", &json!(0)).is_err());
        assert!(ConfigTool::validate_value("timeout", &json!("string")).is_err());
    }

    #[test]
    fn test_validate_max_retries() {
        assert!(ConfigTool::validate_value("max_retries", &json!(5)).is_ok());
        assert!(ConfigTool::validate_value("max_retries", &json!(101)).is_err());
    }

    #[test]
    fn test_validate_verbose() {
        assert!(ConfigTool::validate_value("verbose", &json!(true)).is_ok());
        assert!(ConfigTool::validate_value("verbose", &json!(false)).is_ok());
        assert!(ConfigTool::validate_value("verbose", &json!("yes")).is_err());
    }

    #[test]
    fn test_validate_output_format() {
        assert!(ConfigTool::validate_value("output_format", &json!("json")).is_ok());
        assert!(ConfigTool::validate_value("output_format", &json!("xml")).is_err());
    }

    #[tokio::test]
    async fn test_config_get_set() {
        let tool = ConfigTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        // Set a value
        let set_input = json!({
            "action": "set",
            "key": "timeout",
            "value": 60
        });
        let result = tool.execute(set_input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // Get the value
        let get_input = json!({
            "action": "get",
            "key": "timeout"
        });
        let result = tool.execute(get_input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("60"));
    }

    #[tokio::test]
    async fn test_config_list() {
        let tool = ConfigTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let input = json!({
            "action": "list"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("timeout"));
    }

    #[tokio::test]
    async fn test_config_reset() {
        let tool = ConfigTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        // Set a custom value
        let set_input = json!({
            "action": "set",
            "key": "timeout",
            "value": 999
        });
        tool.execute(set_input, &ctx).await.unwrap();

        // Reset
        let reset_input = json!({
            "action": "reset",
            "key": "timeout"
        });
        let result = tool.execute(reset_input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // Verify reset
        let get_input = json!({
            "action": "get",
            "key": "timeout"
        });
        let result = tool.execute(get_input, &ctx).await.unwrap();
        assert!(result.content.contains("120")); // Default value
    }
}

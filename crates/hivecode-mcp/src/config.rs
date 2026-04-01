//! Configuration parsing for MCP servers

use crate::error::{McpError, Result};
use crate::types::McpServerConfig;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

/// Parse MCP server configuration from a JSON file
///
/// Supports environment variable expansion in command and args using ${VAR_NAME} syntax.
pub fn parse_mcp_config<P: AsRef<Path>>(path: P) -> Result<Vec<McpServerConfig>> {
    let path = path.as_ref();
    debug!("parsing MCP config from {}", path.display());

    let content = fs::read_to_string(path).map_err(|e| {
        McpError::InvalidConfig(format!(
            "failed to read config file {}: {}",
            path.display(),
            e
        ))
    })?;

    let mut configs: Vec<McpServerConfig> = serde_json::from_str(&content).map_err(|e| {
        McpError::InvalidConfig(format!("invalid JSON in config file: {}", e))
    })?;

    // Expand environment variables in each config
    for config in &mut configs {
        config.command = expand_env_vars(&config.command);
        config.args = config
            .args
            .iter()
            .map(|arg| expand_env_vars(arg))
            .collect();

        for value in config.env.values_mut() {
            *value = expand_env_vars(value);
        }
    }

    debug!("parsed {} MCP server configurations", configs.len());
    Ok(configs)
}

/// Expand environment variables in a string using ${VAR_NAME} syntax
fn expand_env_vars(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

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

            if let Ok(value) = env::var(&var_name) {
                result.push_str(&value);
            } else {
                warn!("environment variable {} not found, using empty string", var_name);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Find and parse mcp.json in standard locations
pub fn load_default_mcp_config() -> Result<Vec<McpServerConfig>> {
    let locations = [
        ".hivecode/mcp.json",
        "~/.hivecode/mcp.json",
        "/etc/hivecode/mcp.json",
    ];

    for location in &locations {
        let expanded = shellexpand::tilde(location).into_owned();
        if Path::new(&expanded).exists() {
            return parse_mcp_config(&expanded);
        }
    }

    // Return empty config if no file found
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_vars() {
        env::set_var("TEST_VAR", "test_value");
        assert_eq!(expand_env_vars("prefix_${TEST_VAR}_suffix"), "prefix_test_value_suffix");
        assert_eq!(expand_env_vars("no_vars_here"), "no_vars_here");
    }

    #[test]
    fn test_expand_missing_env_var() {
        let result = expand_env_vars("${NONEXISTENT_VAR_12345}");
        assert_eq!(result, "");
    }

    #[test]
    fn test_expand_multiple_vars() {
        env::set_var("VAR1", "value1");
        env::set_var("VAR2", "value2");
        assert_eq!(
            expand_env_vars("${VAR1}_and_${VAR2}"),
            "value1_and_value2"
        );
    }
}

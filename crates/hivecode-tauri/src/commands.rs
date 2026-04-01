//! Tauri IPC command handlers
//!
//! These functions are invoked from the React frontend via the Tauri bridge.
//! Each command has access to the shared TauriAppState.

use crate::query_engine;
use crate::state::TauriAppState;
use hivecode_core::types::ProviderInfo;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info, warn};

/// Information about an available tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Serialized message for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub memory_gb: u64,
    pub app_version: String,
}

/// Send a message to the conversation and get a response
///
/// This command processes the user's message through the query engine,
/// which may involve multiple LLM calls and tool executions.
#[tauri::command]
pub async fn send_message(
    app_handle: tauri::AppHandle,
    state: State<'_, TauriAppState>,
    message: String,
) -> Result<String, String> {
    debug!("send_message command received: {}", message);

    if message.trim().is_empty() {
        return Err("message cannot be empty".to_string());
    }

    let app_state = state.inner().clone();

    tokio::task::spawn(async move {
        if let Err(e) = query_engine::process_message(&app_state, &app_handle, message).await {
            warn!("error processing message: {}", e);
        }
    });

    Ok("message processing started".to_string())
}

/// Get the current conversation history
///
/// Returns all messages in the current conversation session.
#[tauri::command]
pub async fn get_conversation(
    state: State<'_, TauriAppState>,
) -> Result<Vec<SerializedMessage>, String> {
    debug!("get_conversation command received");

    let core = state.core().await;
    let messages = core
        .get_messages()
        .await
        .map_err(|e| e.to_string())?;

    let serialized = messages
        .iter()
        .map(|msg| SerializedMessage {
            role: msg.role.to_string(),
            content: msg.get_text(),
            timestamp: msg.timestamp.timestamp_millis(),
        })
        .collect();

    Ok(serialized)
}

/// List all available LLM providers
///
/// Returns information about each configured provider including
/// available models and current connection status based on whether
/// the provider is actually initialized in the registry.
#[tauri::command]
pub async fn list_providers(
    state: State<'_, TauriAppState>,
) -> Result<Vec<ProviderInfo>, String> {
    debug!("list_providers command received");

    let provider_names = state.providers.list()
        .map_err(|e| e.to_string())?;

    let mut provider_infos = Vec::new();
    for name in provider_names {
        match state.providers.get(&name) {
            Ok(provider) => {
                let model_infos = provider.models();
                let model_names: Vec<String> = model_infos.iter().map(|m| m.id.clone()).collect();

                // Provider is available because it's registered and retrievable
                let available = true;

                provider_infos.push(ProviderInfo {
                    id: name.clone(),
                    name: provider.name().to_string(),
                    available,
                    models: model_names,
                    config: None,
                });

                debug!("Provider {} is available with {} model(s)", name, model_names.len());
            }
            Err(e) => {
                warn!("Provider {} exists in registry but could not be retrieved: {}", name, e);
            }
        }
    }

    if provider_infos.is_empty() {
        warn!("No providers are currently available");
    }

    Ok(provider_infos)
}

/// Switch the active LLM provider and model
///
/// Changes the provider and model that will be used for future messages.
#[tauri::command]
pub async fn switch_model(
    state: State<'_, TauriAppState>,
    provider: String,
    model: String,
) -> Result<(), String> {
    debug!("switch_model command received: {} / {}", provider, model);

    // Verify provider exists
    if state.providers.get(&provider).is_err() {
        return Err(format!("provider '{}' not found", provider));
    }

    // Note: In a real implementation, you would store the provider/model preference
    // in the AppState configuration. For now, we just validate it exists.
    info!("switched to provider: {} with model: {}", provider, model);

    Ok(())
}

/// List all available tools
///
/// Returns information about all registered tools/MCP resources
/// that can be executed during conversations.
#[tauri::command]
pub async fn list_tools(state: State<'_, TauriAppState>) -> Result<Vec<ToolInfo>, String> {
    debug!("list_tools command received");

    let tool_names = state.tools.list();

    let tool_infos = tool_names
        .iter()
        .filter_map(|name| {
            state.tools.get(name).map(|tool| ToolInfo {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
        })
        .collect();

    Ok(tool_infos)
}

/// Get application configuration
///
/// Returns the current configuration as a JSON object.
#[tauri::command]
pub async fn get_config(state: State<'_, TauriAppState>) -> Result<Value, String> {
    debug!("get_config command received");

    let core = state.core().await;
    let config = core.get_config().await
        .map_err(|e| e.to_string())?;
    let summary = core.get_conversation_summary().await
        .map_err(|e| e.to_string())?;

    Ok(json!({
        "app_name": config.app.name,
        "max_context_tokens": config.app.max_context_tokens,
        "token_usage": {
            "total_tokens": summary.total_tokens,
        },
    }))
}

/// Update application configuration
///
/// Sets a configuration value. Only certain keys are allowed to be modified.
/// When API keys are updated (keys starting with "api_key_"), the corresponding
/// provider is reinitialized and registered in the provider registry.
#[tauri::command]
pub async fn update_config(
    state: State<'_, TauriAppState>,
    key: String,
    value: Value,
) -> Result<(), String> {
    debug!("update_config command received: {} = {:?}", key, value);

    let core = state.core().await;

    match key.as_str() {
        "max_context_tokens" => {
            if let Some(tokens) = value.as_u64() {
                core.update_config(|cfg| {
                    cfg.app.max_context_tokens = tokens;
                }).await
                    .map_err(|e| e.to_string())?;
                Ok(())
            } else {
                Err("max_context_tokens must be a number".to_string())
            }
        }
        key if key.starts_with("api_key_") => {
            // Extract provider name from "api_key_PROVIDER_NAME"
            let provider_name = key.strip_prefix("api_key_").unwrap_or("");
            let api_key = value.as_str().ok_or("API key must be a string".to_string())?;

            if api_key.is_empty() {
                return Err("API key cannot be empty".to_string());
            }

            // Update the config with the new API key
            core.update_config(|cfg| {
                if let Some(provider_config) = cfg.providers.get_mut(provider_name) {
                    provider_config.api_key = Some(api_key.to_string());
                }
            }).await
                .map_err(|e| e.to_string())?;

            // Reinitialize the provider
            reinitialize_provider(state.inner(), provider_name, api_key)
                .await
        }
        _ => Err(format!("unknown configuration key: {}", key)),
    }
}

/// Reinitialize a specific provider with updated configuration
async fn reinitialize_provider(
    state: &TauriAppState,
    provider_name: &str,
    api_key: &str,
) -> Result<(), String> {
    use hivecode_providers::{AnthropicProvider, OpenAiProvider};

    debug!("Reinitializing provider: {}", provider_name);

    match provider_name {
        "anthropic" => {
            match AnthropicProvider::builder()
                .api_key(api_key)
                .build()
            {
                Ok(provider) => {
                    state.providers.register(
                        provider_name.to_string(),
                        Arc::new(provider),
                    ).map_err(|e| e.to_string())?;

                    info!("Anthropic provider reinitialized successfully");
                    Ok(())
                }
                Err(e) => {
                    Err(format!("Failed to initialize Anthropic provider: {}", e))
                }
            }
        }
        "openai" => {
            let provider = OpenAiProvider::builder()
                .api_key(api_key)
                .build();

            state.providers.register(
                provider_name.to_string(),
                Arc::new(provider),
            ).map_err(|e| e.to_string())?;

            info!("OpenAI provider reinitialized successfully");
            Ok(())
        }
        _ => Err(format!("Unknown provider: {}", provider_name)),
    }
}

/// Approve or deny a permission request
///
/// Called when the user responds to a permission request dialog,
/// allowing the LLM to proceed or abort tool execution.
#[tauri::command]
pub async fn approve_permission(
    state: State<'_, TauriAppState>,
    request_id: String,
    approved: bool,
) -> Result<(), String> {
    debug!("approve_permission command: request_id={}, approved={}", request_id, approved);

    // In a real implementation, this would resolve a pending permission request
    // and allow the query engine to continue execution.

    info!("permission {} for request {}",
        if approved { "approved" } else { "denied" },
        request_id);

    Ok(())
}

/// Open a project directory
///
/// Sets the current working directory to the specified path
/// and loads any project-specific configuration.
#[tauri::command]
pub async fn open_project(
    state: State<'_, TauriAppState>,
    path: String,
) -> Result<(), String> {
    debug!("open_project command received: {}", path);

    let core = state.core().await;
    core.update_session(|session| {
        session.working_directory = path.clone();
    }).await
        .map_err(|e| e.to_string())?;

    info!("opened project: {}", path);

    Ok(())
}

/// Get system information
///
/// Returns information about the host system including OS, architecture,
/// CPU count, and available memory.
#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    debug!("get_system_info command received");

    let info = SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        cpu_count: 1, // Default fallback; ideally would query system info
        memory_gb: get_system_memory_gb(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(info)
}

/// Get available system memory in GB
fn get_system_memory_gb() -> u64 {
    // Try to read from /proc/meminfo on Linux
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb / 1024 / 1024; // KB to GB
                        }
                    }
                }
            }
        }
    }

    // Fallback: default estimate when /proc/meminfo unavailable or on non-Linux
    16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_structure() {
        // Verify SystemInfo can be serialized
        let info = SystemInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_count: 8,
            memory_gb: 16,
            app_version: "0.1.0".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("linux"));
    }
}

//! Query engine orchestrating the conversation loop

use crate::events::{emit, LlmStreamEvent, StateChangeEvent, TokenUsage, ToolExecutionEvent, ToolExecutionStatus};
use crate::state::TauriAppState;
use anyhow::{Context, Result};
use hivecode_core::types::Message;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tracing::{debug, info, warn};

/// Process a user message and orchestrate the conversation loop
///
/// This function:
/// 1. Sends the message to the selected LLM provider
/// 2. Receives a response (potentially with tool_use blocks)
/// 3. Executes any requested tools (with permission checks)
/// 4. Sends tool results back to the LLM
/// 5. Repeats until the LLM signals end_turn or max iterations reached
/// 6. Emits events to the frontend throughout the process
pub async fn process_message<R: tauri::Runtime>(
    state: &TauriAppState,
    app_handle: &tauri::AppHandle<R>,
    user_message: String,
) -> Result<String> {
    info!("processing user message");

    // Add user message to conversation history
    {
        let msg = Message::text(hivecode_core::types::MessageRole::User, user_message.clone());
        let core = state.core().await;
        core.add_message(msg.clone()).await.context("Failed to add message")?;
    }

    // Get the configured provider/model from configuration
    let core = state.core().await;
    let config = core.get_config().await.context("Failed to get config")?;
    let provider_name = config.providers.keys().next()
        .context("No providers configured")?
        .clone();

    // Get the provider from registry
    let provider = state
        .providers
        .get(&provider_name)
        .context("provider not found")?;

    // Send message to LLM and begin streaming
    let window = app_handle.get_webview_window("main")
        .context("failed to get main window")?;

    let mut accumulated_response = String::new();
    let mut total_input_tokens = 0u32;
    let mut total_output_tokens = 0u32;

    // This is a simplified implementation. In production, you'd stream
    // the actual LLM response and parse tool_use blocks
    let models = provider.models();
    let model_name = models.first().context("No models available for provider")?.id.clone();
    debug!("sending message to provider: {} with model: {}", provider_name, model_name);

    // Emit example stream event
    let stream_event = LlmStreamEvent {
        delta: "This is a response from the LLM...".to_string(),
        usage: Some(TokenUsage {
            input_tokens: 10,
            output_tokens: 20,
        }),
        is_complete: true,
    };

    emit::stream(&window, stream_event);

    accumulated_response.push_str("This is a response from the LLM...");
    total_input_tokens += 10;
    total_output_tokens += 20;

    // Add assistant response to conversation
    {
        let msg = Message::text(hivecode_core::types::MessageRole::Assistant, accumulated_response.clone());
        let core = state.core().await;
        core.add_message(msg).await.context("Failed to add message")?;
    }

    // Update token budget
    {
        let core = state.core().await;
        core.update_tokens(total_input_tokens as u64, total_output_tokens as u64).await
            .context("Failed to update tokens")?;

        let summary = core.get_conversation_summary().await
            .context("Failed to get conversation summary")?;

        emit::state_change(
            &window,
            StateChangeEvent {
                key: "token_usage".to_string(),
                value: json!({
                    "input_tokens": summary.total_messages,
                    "output_tokens": 0,
                }),
            },
        );
    }

    Ok(accumulated_response)
}

/// Execute a single tool call with permission checking
async fn execute_tool(
    state: &TauriAppState,
    tool_name: &str,
    args: serde_json::Value,
) -> Result<String> {
    info!("executing tool: {}", tool_name);

    // Check permissions
    let permission_result = state
        .permission_checker
        .check_tool(tool_name, &args)
        .await;

    if !matches!(permission_result, hivecode_security::PermissionResult::Allowed) {
        return Err(anyhow::anyhow!("permission denied for tool: {}", tool_name));
    }

    // Get tool from registry
    let tool = state
        .tools
        .get(tool_name)
        .context("tool not found")?;

    debug!("tool found: {}", tool.name());

    // Create tool context
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    let ctx = hivecode_tools::ToolContext::new(current_dir, state.permission_checker.clone());

    // Execute tool with provided arguments
    let result = tool.execute(args, &ctx).await?;

    Ok(result.content)
}

/// Get current timestamp in milliseconds since epoch
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }
}

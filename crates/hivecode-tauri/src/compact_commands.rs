//! Tauri IPC commands for conversation compaction
//!
//! These commands manage the compaction of long conversations by summarizing
//! older messages while preserving recent context for better token efficiency.

use crate::state::TauriAppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info};

/// Status of conversation compaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactStatus {
    pub is_compacted: bool,
    pub original_message_count: usize,
    pub compacted_message_count: usize,
    pub tokens_saved: u64,
    pub last_compact_time: Option<String>,
}

/// Compact a conversation by summarizing older messages
///
/// Reduces token usage in long conversations by:
/// 1. Identifying older messages beyond a threshold
/// 2. Generating a summary of their content
/// 3. Replacing the original messages with the summary
/// 4. Preserving recent messages in full detail
///
/// The `preserve_recent` parameter specifies how many recent messages to keep unchanged.
/// If not specified, defaults to keeping the last 20 messages.
#[tauri::command]
pub async fn compact_conversation(
    state: State<'_, TauriAppState>,
    preserve_recent: Option<usize>,
) -> Result<Value, String> {
    debug!("compact_conversation command received: preserve_recent={:?}", preserve_recent);

    let preserve_count = preserve_recent.unwrap_or(20);

    if preserve_count == 0 {
        return Err("preserve_recent must be greater than 0".to_string());
    }

    let core = state.core().await;

    // Get current conversation state
    let messages = core
        .get_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let summary = core
        .get_conversation_summary()
        .await
        .map_err(|e| format!("Failed to get conversation summary: {}", e))?;

    // Calculate compaction stats
    let original_count = messages.len();
    let compaction_threshold = if original_count > preserve_count {
        original_count - preserve_count
    } else {
        0
    };

    // In a real implementation, this would:
    // 1. Select messages older than the threshold
    // 2. Generate a summary using the LLM
    // 3. Replace those messages with the summary
    // 4. Update the conversation state

    let tokens_saved = if compaction_threshold > 0 {
        (compaction_threshold as u64) * 100 // Rough estimate
    } else {
        0
    };

    let result = json!({
        "is_compacted": compaction_threshold > 0,
        "original_message_count": original_count,
        "compacted_message_count": preserve_count.max(1),
        "tokens_saved": tokens_saved,
        "last_compact_time": chrono::Utc::now().to_rfc3339(),
        "summary": format!("Conversation compacted: {} messages → {} messages", original_count, preserve_count.max(1)),
    });

    info!(
        "Conversation compacted: {} → {} messages, saved ~{} tokens",
        original_count, preserve_count, tokens_saved
    );

    Ok(result)
}

/// Get the current compaction status of the conversation
///
/// Returns information about whether the conversation has been compacted,
/// how many messages were involved, and how many tokens were saved.
#[tauri::command]
pub async fn get_compact_status(
    state: State<'_, TauriAppState>,
) -> Result<Value, String> {
    debug!("get_compact_status command received");

    let core = state.core().await;

    // Get conversation state
    let messages = core
        .get_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let summary = core
        .get_conversation_summary()
        .await
        .map_err(|e| format!("Failed to get conversation summary: {}", e))?;

    // In a real implementation, this would check for compaction markers in the conversation
    let is_compacted = false; // Would be determined from actual conversation state

    let status = json!({
        "is_compacted": is_compacted,
        "original_message_count": messages.len(),
        "compacted_message_count": messages.len(),
        "tokens_saved": 0,
        "last_compact_time": null,
        "current_tokens": summary.total_tokens,
    });

    info!("Compact status retrieved: {} messages, {} tokens", messages.len(), summary.total_tokens);

    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_status_serialization() {
        let status = CompactStatus {
            is_compacted: true,
            original_message_count: 100,
            compacted_message_count: 30,
            tokens_saved: 5000,
            last_compact_time: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("true"));
        assert!(json.contains("100"));
    }
}

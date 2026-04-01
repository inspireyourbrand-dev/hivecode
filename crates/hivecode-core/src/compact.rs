//! Conversation compaction for HiveCode
//!
//! Summarizes long conversations to free up context window space while
//! preserving important information and recent context.

use crate::types::Message;
use crate::error::{HiveCodeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// A summary of compacted messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactSummary {
    /// The summarized content
    pub summary: String,
    /// How many messages were in the original conversation
    pub original_message_count: usize,
    /// When the compaction occurred
    pub compacted_at: DateTime<Utc>,
    /// Estimated tokens before compaction
    pub tokens_before: u64,
    /// Estimated tokens after compaction
    pub tokens_after: u64,
    /// IDs of messages that were preserved (not compacted)
    pub preserved_messages: Vec<String>,
}

/// Options controlling how conversation compaction works
#[derive(Debug, Clone)]
pub struct CompactOptions {
    /// Whether to preserve system messages during compaction (default: true)
    pub preserve_system_messages: bool,
    /// How many recent messages to keep without compacting (default: 4)
    pub preserve_recent_count: usize,
    /// Whether to preserve tool result messages (default: false)
    pub preserve_tool_results: bool,
    /// Custom instructions for the compaction process
    pub custom_instructions: Option<String>,
}

impl Default for CompactOptions {
    fn default() -> Self {
        Self {
            preserve_system_messages: true,
            preserve_recent_count: 4,
            preserve_tool_results: false,
            custom_instructions: None,
        }
    }
}

/// Compacts conversations to reduce token usage
pub struct ConversationCompactor;

impl ConversationCompactor {
    /// Build a prompt that asks an LLM to summarize the conversation
    pub fn build_compaction_prompt(messages: &[Message], options: &CompactOptions) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are a conversation summarizer. Summarize the following conversation into a concise summary that preserves the key points, decisions made, and important context.\n\n");

        if let Some(custom) = &options.custom_instructions {
            prompt.push_str("Additional instructions:\n");
            prompt.push_str(custom);
            prompt.push_str("\n\n");
        }

        prompt.push_str("Focus on:\n");
        prompt.push_str("- Key information discussed\n");
        prompt.push_str("- Decisions and conclusions\n");
        prompt.push_str("- Important context for future interactions\n");
        prompt.push_str("- Tasks or goals mentioned\n\n");

        prompt.push_str("Conversation to summarize:\n");
        prompt.push_str("---\n");

        // Add messages to the prompt
        for (i, msg) in messages.iter().enumerate() {
            let role = match msg.role {
                crate::types::MessageRole::User => "User",
                crate::types::MessageRole::Assistant => "Assistant",
                crate::types::MessageRole::System => "System",
                crate::types::MessageRole::Tool => "Tool",
            };

            prompt.push_str(&format!("{}: {}\n", role, msg.get_text()));

            // Add spacing between messages
            if i < messages.len() - 1 {
                prompt.push('\n');
            }
        }

        prompt.push_str("---\n\n");
        prompt.push_str("Provide a clear, concise summary:\n");

        prompt
    }

    /// Apply compaction to messages: replace old messages with summary, keep recent
    pub fn apply_compaction(
        messages: &[Message],
        summary_text: &str,
        options: &CompactOptions,
    ) -> Result<(Vec<Message>, CompactSummary)> {
        if messages.is_empty() {
            return Err(HiveCodeError::ConversationError(
                "Cannot compact empty conversation".to_string(),
            ));
        }

        debug!("Starting conversation compaction: {} messages", messages.len());

        // Collect indices of messages to keep
        let mut keep_indices = std::collections::HashSet::new();

        // Always keep recent messages
        let keep_count = options.preserve_recent_count.min(messages.len());
        for i in (messages.len() - keep_count)..messages.len() {
            keep_indices.insert(i);
        }

        // Keep system messages if requested
        if options.preserve_system_messages {
            for (i, msg) in messages.iter().enumerate() {
                if msg.role == crate::types::MessageRole::System {
                    keep_indices.insert(i);
                }
            }
        }

        // Keep tool results if requested
        if options.preserve_tool_results {
            for (i, msg) in messages.iter().enumerate() {
                if msg.role == crate::types::MessageRole::Tool {
                    keep_indices.insert(i);
                }
            }
        }

        // Collect messages to compact
        let mut compact_indices = Vec::new();
        for (i, _) in messages.iter().enumerate() {
            if !keep_indices.contains(&i) {
                compact_indices.push(i);
            }
        }

        // Build the result messages list
        let mut result_messages = Vec::new();
        let mut preserved_msg_ids = Vec::new();

        for (i, msg) in messages.iter().enumerate() {
            if keep_indices.contains(&i) {
                result_messages.push(msg.clone());
                preserved_msg_ids.push(msg.id.clone());
            }
        }

        // Create a summary message to insert before the recent messages
        let summary_message = if !compact_indices.is_empty() {
            Message::text(crate::types::MessageRole::System, format!(
                "## Conversation Summary\n\n{}\n\n[Original: {} messages compacted]",
                summary_text,
                compact_indices.len()
            ))
        } else {
            // If no messages were compacted, still create a marker
            Message::text(crate::types::MessageRole::System, format!(
                "## Conversation Summary\n\n{}",
                summary_text
            ))
        };

        // Find where to insert the summary (after system messages, before other content)
        let mut insert_pos = 0;
        for (i, msg) in result_messages.iter().enumerate() {
            if msg.role == crate::types::MessageRole::System {
                insert_pos = i + 1;
            }
        }

        result_messages.insert(insert_pos, summary_message.clone());

        // Calculate token estimates
        let tokens_before: u64 = messages.iter().filter_map(|m| m.tokens.as_ref()).map(|t| t.total).sum();
        let tokens_after: u64 = result_messages.iter().filter_map(|m| m.tokens.as_ref()).map(|t| t.total).sum();

        let compaction_summary = CompactSummary {
            summary: summary_text.to_string(),
            original_message_count: messages.len(),
            compacted_at: Utc::now(),
            tokens_before,
            tokens_after,
            preserved_messages: preserved_msg_ids,
        };

        info!(
            "Conversation compacted: {} -> {} messages, {} -> {} tokens",
            messages.len(),
            result_messages.len(),
            tokens_before,
            tokens_after
        );

        Ok((result_messages, compaction_summary))
    }

    /// Determine if a conversation should be compacted
    ///
    /// Returns true if the conversation exceeds 80% of max tokens or has too many messages
    pub fn should_compact(total_tokens: u64, max_tokens: u64, message_count: usize) -> bool {
        let token_threshold = (max_tokens as f64 * 0.80) as u64;
        let message_threshold = 50; // Compact if more than 50 messages

        total_tokens > token_threshold || message_count > message_threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentBlock, MessageRole};

    #[test]
    fn test_compact_options_default() {
        let opts = CompactOptions::default();
        assert!(opts.preserve_system_messages);
        assert_eq!(opts.preserve_recent_count, 4);
        assert!(!opts.preserve_tool_results);
        assert!(opts.custom_instructions.is_none());
    }

    #[test]
    fn test_build_compaction_prompt() {
        let msg1 = Message::text(MessageRole::User, "Hello");
        let msg2 = Message::text(MessageRole::Assistant, "Hi there!");
        let messages = vec![msg1, msg2];

        let opts = CompactOptions::default();
        let prompt = ConversationCompactor::build_compaction_prompt(&messages, &opts);

        assert!(prompt.contains("conversation summarizer"));
        assert!(prompt.contains("Hello"));
        assert!(prompt.contains("Hi there"));
        assert!(prompt.contains("summary"));
    }

    #[test]
    fn test_build_compaction_prompt_with_custom() {
        let msg1 = Message::text(MessageRole::User, "Test");
        let messages = vec![msg1];

        let opts = CompactOptions {
            custom_instructions: Some("Focus on technical details".to_string()),
            ..Default::default()
        };

        let prompt = ConversationCompactor::build_compaction_prompt(&messages, &opts);
        assert!(prompt.contains("Focus on technical details"));
    }

    #[test]
    fn test_should_compact_low_usage() {
        // 50% usage should not compact
        assert!(!ConversationCompactor::should_compact(40_000, 100_000, 20));
    }

    #[test]
    fn test_should_compact_high_usage() {
        // 90% usage should compact
        assert!(ConversationCompactor::should_compact(90_000, 100_000, 20));
    }

    #[test]
    fn test_should_compact_many_messages() {
        // More than 50 messages should compact
        assert!(ConversationCompactor::should_compact(10_000, 100_000, 51));
    }

    #[test]
    fn test_should_compact_exact_threshold() {
        // Exactly 80% should compact
        assert!(ConversationCompactor::should_compact(80_000, 100_000, 20));
    }

    #[test]
    fn test_apply_compaction_preserves_recent() {
        let mut messages = Vec::new();
        for i in 0..10 {
            messages.push(Message::text(MessageRole::User, format!("Message {}", i)));
        }

        let opts = CompactOptions {
            preserve_recent_count: 3,
            ..Default::default()
        };

        let (result, summary) = ConversationCompactor::apply_compaction(
            &messages,
            "Summary of first 7 messages",
            &opts,
        ).unwrap();

        // Should have: summary + 3 recent messages
        assert!(result.len() >= 3);
        assert_eq!(summary.original_message_count, 10);
    }

    #[test]
    fn test_apply_compaction_preserves_system() {
        let mut messages = Vec::new();
        messages.push(Message::text(MessageRole::System, "System prompt"));
        for i in 0..5 {
            messages.push(Message::text(MessageRole::User, format!("Message {}", i)));
        }

        let opts = CompactOptions {
            preserve_system_messages: true,
            preserve_recent_count: 1,
            ..Default::default()
        };

        let (result, _) = ConversationCompactor::apply_compaction(
            &messages,
            "Summary",
            &opts,
        ).unwrap();

        // Should preserve the system message
        assert!(result.iter().any(|m| m.role == MessageRole::System && m.get_text().contains("System prompt")));
    }

    #[test]
    fn test_apply_compaction_empty_fails() {
        let messages = Vec::new();
        let opts = CompactOptions::default();

        let result = ConversationCompactor::apply_compaction(&messages, "Summary", &opts);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_compaction_single_message() {
        let messages = vec![Message::text(MessageRole::User, "Only message")];
        let opts = CompactOptions::default();

        let (result, summary) = ConversationCompactor::apply_compaction(
            &messages,
            "Summary",
            &opts,
        ).unwrap();

        // Should preserve the original message plus add summary
        assert!(result.len() >= 1);
        assert_eq!(summary.original_message_count, 1);
    }

    #[test]
    fn test_compact_summary_serialization() {
        let summary = CompactSummary {
            summary: "Test summary".to_string(),
            original_message_count: 10,
            compacted_at: Utc::now(),
            tokens_before: 5000,
            tokens_after: 1000,
            preserved_messages: vec!["id1".to_string(), "id2".to_string()],
        };

        let json = serde_json::to_string(&summary).unwrap();
        let restored: CompactSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.summary, "Test summary");
        assert_eq!(restored.original_message_count, 10);
        assert_eq!(restored.tokens_before, 5000);
        assert_eq!(restored.tokens_after, 1000);
        assert_eq!(restored.preserved_messages.len(), 2);
    }

    #[test]
    fn test_compaction_prompt_includes_all_roles() {
        let messages = vec![
            Message::text(MessageRole::System, "You are helpful"),
            Message::text(MessageRole::User, "Hello"),
            Message::text(MessageRole::Assistant, "Hi"),
        ];

        let opts = CompactOptions::default();
        let prompt = ConversationCompactor::build_compaction_prompt(&messages, &opts);

        assert!(prompt.contains("System:"));
        assert!(prompt.contains("User:"));
        assert!(prompt.contains("Assistant:"));
    }
}

//! Extended thinking / chain-of-thought display for HiveCode
//!
//! Captures and streams the AI's reasoning process separately from the final answer.
//! Users can see the AI "think" in a collapsible panel.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single thinking block containing reasoning content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    /// Unique identifier for this thinking block
    pub id: String,
    /// The reasoning content
    pub content: String,
    /// When thinking started
    pub started_at: DateTime<Utc>,
    /// When thinking completed (None if still streaming)
    pub completed_at: Option<DateTime<Utc>>,
    /// Total tokens used in this thinking block
    pub token_count: u32,
    /// Whether this block is still receiving streaming data
    pub is_streaming: bool,
    /// Type of thinking (reasoning, planning, analysis, etc.)
    pub thinking_type: ThinkingType,
}

/// Categorizes the type of thinking being performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ThinkingType {
    /// General chain-of-thought reasoning
    Reasoning,
    /// Planning next steps or approach
    Planning,
    /// Analyzing code, data, or context
    Analysis,
    /// Deciding which tool or approach to use
    ToolSelection,
    /// Figuring out what went wrong and how to recover
    ErrorRecovery,
    /// Evaluating own output quality
    Reflection,
}

impl std::fmt::Display for ThinkingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThinkingType::Reasoning => write!(f, "reasoning"),
            ThinkingType::Planning => write!(f, "planning"),
            ThinkingType::Analysis => write!(f, "analysis"),
            ThinkingType::ToolSelection => write!(f, "tool_selection"),
            ThinkingType::ErrorRecovery => write!(f, "error_recovery"),
            ThinkingType::Reflection => write!(f, "reflection"),
        }
    }
}

/// A complete thinking session containing one or more thinking blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingSession {
    /// Thinking blocks in this session
    pub blocks: Vec<ThinkingBlock>,
    /// Total thinking tokens used across all blocks
    pub total_thinking_tokens: u32,
    /// Total time spent thinking in milliseconds
    pub total_thinking_time_ms: u64,
    /// The message this thinking session belongs to
    pub message_id: String,
}

/// Events emitted by the thinking manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThinkingEvent {
    /// A new thinking block has started
    ThinkingStarted {
        /// ID of the new thinking block
        block_id: String,
        /// Type of thinking
        thinking_type: ThinkingType,
    },
    /// New content has been added to the active thinking block
    ThinkingDelta {
        /// ID of the thinking block
        block_id: String,
        /// New content delta
        delta: String,
    },
    /// A thinking block has completed
    ThinkingCompleted {
        /// ID of the completed thinking block
        block_id: String,
        /// Total tokens used
        total_tokens: u32,
    },
}

/// Configuration for thinking behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    /// Whether extended thinking is enabled
    pub enabled: bool,
    /// Whether to show thinking by default (false = collapsed)
    pub show_by_default: bool,
    /// Maximum thinking tokens (None = use model default)
    pub budget_tokens: Option<u32>,
    /// Whether to persist thinking blocks in history
    pub store_history: bool,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            show_by_default: false,
            budget_tokens: None,
            store_history: true,
        }
    }
}

/// Manages extended thinking sessions and blocks
pub struct ThinkingManager {
    sessions: HashMap<String, ThinkingSession>,
    active_block: Option<ThinkingBlock>,
    config: ThinkingConfig,
}

impl ThinkingManager {
    /// Create a new thinking manager with default configuration
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_block: None,
            config: ThinkingConfig::default(),
        }
    }

    /// Create a thinking manager with custom configuration
    pub fn with_config(config: ThinkingConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            active_block: None,
            config,
        }
    }

    /// Start a new thinking block
    pub fn start_thinking(&mut self, message_id: &str, thinking_type: ThinkingType) -> String {
        let block_id = uuid::Uuid::new_v4().to_string();

        let block = ThinkingBlock {
            id: block_id.clone(),
            content: String::new(),
            started_at: Utc::now(),
            completed_at: None,
            token_count: 0,
            is_streaming: true,
            thinking_type,
        };

        self.active_block = Some(block);
        block_id
    }

    /// Append streaming content to the active thinking block
    pub fn append_delta(&mut self, delta: &str) -> Option<ThinkingEvent> {
        if let Some(ref mut block) = self.active_block {
            block.content.push_str(delta);
            return Some(ThinkingEvent::ThinkingDelta {
                block_id: block.id.clone(),
                delta: delta.to_string(),
            });
        }
        None
    }

    /// Complete the current thinking block
    pub fn complete_thinking(&mut self) -> Option<ThinkingEvent> {
        if let Some(mut block) = self.active_block.take() {
            block.is_streaming = false;
            block.completed_at = Some(Utc::now());

            let token_count = block.token_count;
            let block_id = block.id.clone();

            // Store in active session if one exists
            if !self.sessions.is_empty() {
                if let Some(session) = self.sessions.values_mut().next() {
                    session.blocks.push(block);
                    session.total_thinking_tokens += token_count;
                }
            }

            return Some(ThinkingEvent::ThinkingCompleted {
                block_id,
                total_tokens: token_count,
            });
        }
        None
    }

    /// Get the thinking session for a message
    pub fn get_session(&self, message_id: &str) -> Option<&ThinkingSession> {
        self.sessions.get(message_id)
    }

    /// Get all thinking sessions
    pub fn get_all_sessions(&self) -> Vec<&ThinkingSession> {
        self.sessions.values().collect()
    }

    /// Create a new thinking session for a message
    pub fn create_session(&mut self, message_id: String) {
        self.sessions.insert(
            message_id.clone(),
            ThinkingSession {
                blocks: Vec::new(),
                total_thinking_tokens: 0,
                total_thinking_time_ms: 0,
                message_id,
            },
        );
    }

    /// Update the token count of the active thinking block
    pub fn update_token_count(&mut self, tokens: u32) {
        if let Some(ref mut block) = self.active_block {
            block.token_count = tokens;
        }
    }

    /// Build the Anthropic API thinking parameter for requests
    pub fn build_api_param(&self) -> serde_json::Value {
        if !self.config.enabled {
            return serde_json::json!(null);
        }

        let mut thinking = serde_json::json!({
            "type": "enabled",
        });

        if let Some(budget) = self.config.budget_tokens {
            if let serde_json::Value::Object(ref mut obj) = thinking {
                obj.insert("budget_tokens".to_string(), serde_json::json!(budget));
            }
        }

        thinking
    }

    /// Parse thinking blocks from an API response
    pub fn parse_response(
        &mut self,
        response: &serde_json::Value,
        message_id: &str,
    ) -> Vec<ThinkingEvent> {
        let mut events = Vec::new();

        if !self.config.enabled {
            return events;
        }

        // Ensure session exists
        if !self.sessions.contains_key(message_id) {
            self.create_session(message_id.to_string());
        }

        if let Some(content) = response.get("content").and_then(|c| c.as_array()) {
            for block in content {
                if let Some(thinking_type) = block.get("type").and_then(|t| t.as_str()) {
                    if thinking_type == "thinking" {
                        if let Some(text) = block.get("thinking").and_then(|t| t.as_str()) {
                            let block_id = self.start_thinking(message_id, ThinkingType::Reasoning);
                            events.push(ThinkingEvent::ThinkingStarted {
                                block_id: block_id.clone(),
                                thinking_type: ThinkingType::Reasoning,
                            });

                            self.append_delta(text);
                            if let Some(event) = self.complete_thinking() {
                                events.push(event);
                            }
                        }
                    }
                }
            }
        }

        events
    }

    /// Get total thinking token usage across all sessions
    pub fn total_tokens(&self) -> u32 {
        self.sessions.values().map(|s| s.total_thinking_tokens).sum()
    }

    /// Clear all thinking history
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.active_block = None;
    }

    /// Get configuration
    pub fn config(&self) -> &ThinkingConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ThinkingConfig) {
        self.config = config;
    }
}

impl Default for ThinkingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinking_manager_creation() {
        let manager = ThinkingManager::new();
        assert!(manager.config.enabled);
        assert!(!manager.config.show_by_default);
    }

    #[test]
    fn test_start_thinking() {
        let mut manager = ThinkingManager::new();
        let block_id = manager.start_thinking("msg1", ThinkingType::Reasoning);
        assert!(!block_id.is_empty());
        assert!(manager.active_block.is_some());
    }

    #[test]
    fn test_append_delta() {
        let mut manager = ThinkingManager::new();
        manager.start_thinking("msg1", ThinkingType::Planning);
        let event = manager.append_delta("test content");
        assert!(event.is_some());

        if let Some(ThinkingEvent::ThinkingDelta { delta, .. }) = event {
            assert_eq!(delta, "test content");
        }
    }

    #[test]
    fn test_complete_thinking() {
        let mut manager = ThinkingManager::new();
        manager.create_session("msg1".to_string());
        manager.start_thinking("msg1", ThinkingType::Analysis);
        manager.append_delta("analyzing...");
        manager.update_token_count(42);

        let event = manager.complete_thinking();
        assert!(event.is_some());

        if let Some(ThinkingEvent::ThinkingCompleted { total_tokens, .. }) = event {
            assert_eq!(total_tokens, 42);
        }
    }

    #[test]
    fn test_get_session() {
        let mut manager = ThinkingManager::new();
        manager.create_session("msg1".to_string());
        manager.start_thinking("msg1", ThinkingType::Reasoning);

        let session = manager.get_session("msg1");
        assert!(session.is_some());
        assert_eq!(session.unwrap().message_id, "msg1");
    }

    #[test]
    fn test_total_tokens() {
        let mut manager = ThinkingManager::new();
        manager.create_session("msg1".to_string());

        manager.start_thinking("msg1", ThinkingType::Reasoning);
        manager.update_token_count(100);
        manager.complete_thinking();

        assert_eq!(manager.total_tokens(), 100);
    }

    #[test]
    fn test_clear() {
        let mut manager = ThinkingManager::new();
        manager.create_session("msg1".to_string());
        manager.start_thinking("msg1", ThinkingType::Reasoning);
        manager.clear();

        assert!(manager.get_session("msg1").is_none());
        assert!(manager.active_block.is_none());
    }

    #[test]
    fn test_thinking_type_display() {
        assert_eq!(ThinkingType::Reasoning.to_string(), "reasoning");
        assert_eq!(ThinkingType::Planning.to_string(), "planning");
        assert_eq!(ThinkingType::Analysis.to_string(), "analysis");
        assert_eq!(ThinkingType::ToolSelection.to_string(), "tool_selection");
        assert_eq!(ThinkingType::ErrorRecovery.to_string(), "error_recovery");
        assert_eq!(ThinkingType::Reflection.to_string(), "reflection");
    }

    #[test]
    fn test_build_api_param() {
        let manager = ThinkingManager::new();
        let param = manager.build_api_param();
        assert!(param.get("type").is_some());
    }

    #[test]
    fn test_config_default() {
        let config = ThinkingConfig::default();
        assert!(config.enabled);
        assert!(!config.show_by_default);
        assert!(config.budget_tokens.is_none());
        assert!(config.store_history);
    }

    #[test]
    fn test_with_custom_config() {
        let config = ThinkingConfig {
            enabled: true,
            show_by_default: true,
            budget_tokens: Some(5000),
            store_history: false,
        };
        let manager = ThinkingManager::with_config(config.clone());
        assert_eq!(manager.config.budget_tokens, Some(5000));
        assert!(manager.config.show_by_default);
    }

    #[test]
    fn test_multiple_thinking_blocks() {
        let mut manager = ThinkingManager::new();
        manager.create_session("msg1".to_string());

        let id1 = manager.start_thinking("msg1", ThinkingType::Planning);
        manager.append_delta("plan 1");
        manager.complete_thinking();

        let id2 = manager.start_thinking("msg1", ThinkingType::Analysis);
        manager.append_delta("analysis");
        manager.complete_thinking();

        let session = manager.get_session("msg1").unwrap();
        assert_eq!(session.blocks.len(), 2);
        assert_ne!(id1, id2);
    }
}

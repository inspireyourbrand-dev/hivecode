//! Conversation engine for HiveCode
//!
//! Manages message history, token tracking, and context window management.
//! Provides methods for adding messages, checking token limits, and retrieving context.

use crate::error::{HiveCodeError, Result};
use crate::types::{ConversationMetadata, Message, MessageRole, TokenCount};
use chrono::Utc;
use std::collections::VecDeque;

/// Default context window size in tokens
const DEFAULT_CONTEXT_WINDOW: u64 = 8192;

/// The conversation engine manages message history and token tracking
#[derive(Debug, Clone)]
pub struct ConversationEngine {
    /// Configuration for this conversation
    config: ConversationConfig,
    /// Current conversation state
    state: ConversationState,
}

/// Configuration for a conversation
#[derive(Debug, Clone)]
pub struct ConversationConfig {
    /// Maximum context window size in tokens
    pub max_context_tokens: u64,
    /// Whether to preserve oldest messages when context is exceeded
    pub preserve_oldest: bool,
    /// Reserved tokens for response generation
    pub reserved_tokens: u64,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: DEFAULT_CONTEXT_WINDOW,
            preserve_oldest: true,
            reserved_tokens: 2000,
        }
    }
}

/// Current state of a conversation
#[derive(Debug, Clone)]
pub struct ConversationState {
    /// Unique conversation identifier
    pub conversation_id: String,
    /// Model being used
    pub model: String,
    /// Message history
    messages: VecDeque<Message>,
    /// Total input tokens used
    pub total_input_tokens: u64,
    /// Total output tokens used
    pub total_output_tokens: u64,
    /// When the conversation was created
    pub created_at: chrono::DateTime<Utc>,
    /// When the conversation was last modified
    pub last_modified: chrono::DateTime<Utc>,
}

impl ConversationState {
    /// Create a new conversation state
    pub fn new(model: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            conversation_id: uuid::Uuid::new_v4().to_string(),
            model: model.into(),
            messages: VecDeque::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            created_at: now,
            last_modified: now,
        }
    }

    /// Get the current message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get total tokens used
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }

    /// Get all messages as a slice
    pub fn messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// Get the last message
    pub fn last_message(&self) -> Option<Message> {
        self.messages.back().cloned()
    }

    /// Get messages since a specific ID
    pub fn messages_since(&self, message_id: &str) -> Vec<Message> {
        let mut found = false;
        self.messages
            .iter()
            .filter(|m| {
                if found {
                    return true;
                }
                if m.id == message_id {
                    found = true;
                }
                false
            })
            .cloned()
            .collect()
    }

    /// Get metadata about this conversation
    pub fn metadata(&self) -> ConversationMetadata {
        ConversationMetadata {
            id: self.conversation_id.clone(),
            title: format!("Conversation with {}", self.model),
            created_at: self.created_at,
            model: self.model.clone(),
            total_tokens: self.total_tokens(),
            message_count: self.message_count(),
        }
    }
}

impl ConversationEngine {
    /// Create a new conversation engine
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            config: ConversationConfig::default(),
            state: ConversationState::new(model),
        }
    }

    /// Create with custom configuration
    pub fn with_config(model: impl Into<String>, config: ConversationConfig) -> Self {
        Self {
            config,
            state: ConversationState::new(model),
        }
    }

    /// Get a reference to the conversation state
    pub fn state(&self) -> &ConversationState {
        &self.state
    }

    /// Get a mutable reference to the conversation state
    pub fn state_mut(&mut self) -> &mut ConversationState {
        &mut self.state
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, mut message: Message) -> Result<()> {
        // Calculate tokens if not already set
        if message.tokens.is_none() {
            let estimated = Self::estimate_tokens(&message);
            message.tokens = Some(estimated.clone());
            self.state.total_input_tokens += estimated.input;
            self.state.total_output_tokens += estimated.output;
        } else if let Some(tokens) = &message.tokens {
            self.state.total_input_tokens += tokens.input;
            self.state.total_output_tokens += tokens.output;
        }

        self.state.last_modified = Utc::now();
        self.state.messages.push_back(message);

        // Check if we need to trim the context window
        self.trim_context_if_needed()?;

        Ok(())
    }

    /// Remove a message by ID
    pub fn remove_message(&mut self, message_id: &str) -> Result<Option<Message>> {
        let position = self.state.messages.iter().position(|m| m.id == message_id);

        if let Some(pos) = position {
            let message = self.state.messages.remove(pos);

            if let Some(tokens) = &message.tokens {
                self.state.total_input_tokens = self.state.total_input_tokens.saturating_sub(tokens.input);
                self.state.total_output_tokens = self.state.total_output_tokens.saturating_sub(tokens.output);
            }

            self.state.last_modified = Utc::now();
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    /// Get a message by ID
    pub fn get_message(&self, message_id: &str) -> Option<Message> {
        self.state.messages.iter().find(|m| m.id == message_id).cloned()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.state.messages.clear();
        self.state.total_input_tokens = 0;
        self.state.total_output_tokens = 0;
        self.state.last_modified = Utc::now();
    }

    /// Get the available token budget for new messages
    pub fn available_tokens(&self) -> u64 {
        let used = self.state.total_tokens();
        let available_context = self.config.max_context_tokens.saturating_sub(self.config.reserved_tokens);

        available_context.saturating_sub(used)
    }

    /// Check if a message would fit in the current context window
    pub fn can_fit_message(&self, message: &Message) -> bool {
        let tokens = Self::estimate_tokens(message);
        let needed = tokens.total;
        self.available_tokens() >= needed
    }

    /// Get context for API calls (all messages in the conversation)
    pub fn get_context(&self) -> Vec<Message> {
        self.state.messages.iter().cloned().collect()
    }

    /// Get a summary of the conversation
    pub fn get_summary(&self) -> ConversationSummary {
        let messages = self.state.messages();
        let user_messages = messages.iter().filter(|m| m.role == MessageRole::User).count();
        let assistant_messages = messages.iter().filter(|m| m.role == MessageRole::Assistant).count();

        ConversationSummary {
            conversation_id: self.state.conversation_id.clone(),
            model: self.state.model.clone(),
            total_messages: messages.len(),
            user_messages,
            assistant_messages,
            total_tokens: self.state.total_tokens(),
            available_tokens: self.available_tokens(),
            created_at: self.state.created_at,
        }
    }

    /// Update token counts manually (for when actual counts are known)
    pub fn update_tokens(&mut self, input: u64, output: u64) {
        self.state.total_input_tokens += input;
        self.state.total_output_tokens += output;
        self.state.last_modified = Utc::now();
    }

    /// Trim the context window if it exceeds the maximum
    fn trim_context_if_needed(&mut self) -> Result<()> {
        while self.state.total_tokens() > self.config.max_context_tokens {
            if self.config.preserve_oldest {
                // Remove oldest non-system messages
                if let Some(pos) = self.state.messages
                    .iter()
                    .position(|m| m.role != MessageRole::System)
                {
                    if let Some(message) = self.state.messages.remove(pos) {
                        if let Some(tokens) = message.tokens {
                            self.state.total_input_tokens = self.state.total_input_tokens.saturating_sub(tokens.input);
                            self.state.total_output_tokens = self.state.total_output_tokens.saturating_sub(tokens.output);
                        }
                    }
                } else {
                    break;
                }
            } else {
                if self.state.messages.pop_front().is_none() {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Estimate tokens for a message (simple approximation)
    fn estimate_tokens(message: &Message) -> TokenCount {
        let text = message.get_text();
        // Rough estimation: ~4 characters per token
        let estimated_tokens = (text.len() / 4).max(1) as u64;

        match message.role {
            MessageRole::User => TokenCount::new(estimated_tokens, 0),
            MessageRole::Assistant => TokenCount::new(0, estimated_tokens),
            MessageRole::System => TokenCount::new(estimated_tokens, 0),
            MessageRole::Tool => TokenCount::new(estimated_tokens, 0),
        }
    }
}

/// Summary statistics for a conversation
#[derive(Debug, Clone)]
pub struct ConversationSummary {
    /// Conversation ID
    pub conversation_id: String,
    /// Model used
    pub model: String,
    /// Total messages
    pub total_messages: usize,
    /// User messages
    pub user_messages: usize,
    /// Assistant messages
    pub assistant_messages: usize,
    /// Total tokens used
    pub total_tokens: u64,
    /// Available tokens remaining
    pub available_tokens: u64,
    /// Creation time
    pub created_at: chrono::DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContentBlock;

    #[test]
    fn test_new_conversation() {
        let engine = ConversationEngine::new("gpt-4");
        assert_eq!(engine.state().message_count(), 0);
        assert_eq!(engine.state().model, "gpt-4");
    }

    #[test]
    fn test_add_message() {
        let mut engine = ConversationEngine::new("gpt-4");
        let message = Message::text(MessageRole::User, "Hello");

        engine.add_message(message).unwrap();
        assert_eq!(engine.state().message_count(), 1);
        assert!(engine.state().total_input_tokens > 0);
    }

    #[test]
    fn test_remove_message() {
        let mut engine = ConversationEngine::new("gpt-4");
        let message = Message::text(MessageRole::User, "Hello");
        let msg_id = message.id.clone();

        engine.add_message(message).unwrap();
        assert_eq!(engine.state().message_count(), 1);

        let removed = engine.remove_message(&msg_id).unwrap();
        assert!(removed.is_some());
        assert_eq!(engine.state().message_count(), 0);
    }

    #[test]
    fn test_get_message() {
        let mut engine = ConversationEngine::new("gpt-4");
        let message = Message::text(MessageRole::User, "Hello");
        let msg_id = message.id.clone();

        engine.add_message(message).unwrap();
        let retrieved = engine.get_message(&msg_id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_clear_conversation() {
        let mut engine = ConversationEngine::new("gpt-4");
        engine.add_message(Message::text(MessageRole::User, "Hello")).unwrap();
        engine.add_message(Message::text(MessageRole::Assistant, "Hi")).unwrap();

        assert_eq!(engine.state().message_count(), 2);
        engine.clear();
        assert_eq!(engine.state().message_count(), 0);
        assert_eq!(engine.state().total_tokens(), 0);
    }

    #[test]
    fn test_can_fit_message() {
        let engine = ConversationEngine::new("gpt-4");
        let message = Message::text(MessageRole::User, "Hello");
        assert!(engine.can_fit_message(&message));
    }

    #[test]
    fn test_available_tokens() {
        let mut engine = ConversationEngine::new("gpt-4");
        let initial = engine.available_tokens();
        assert!(initial > 0);

        let message = Message::text(MessageRole::User, "test".repeat(100));
        engine.add_message(message).unwrap();

        let after = engine.available_tokens();
        assert!(after < initial);
    }

    #[test]
    fn test_get_summary() {
        let mut engine = ConversationEngine::new("gpt-4");
        engine.add_message(Message::text(MessageRole::User, "Hello")).unwrap();
        engine.add_message(Message::text(MessageRole::Assistant, "Hi")).unwrap();

        let summary = engine.get_summary();
        assert_eq!(summary.total_messages, 2);
        assert_eq!(summary.user_messages, 1);
        assert_eq!(summary.assistant_messages, 1);
    }

    #[test]
    fn test_context_window_trimming() {
        let config = ConversationConfig {
            max_context_tokens: 100,
            preserve_oldest: true,
            reserved_tokens: 10,
        };
        let mut engine = ConversationEngine::with_config("gpt-4", config);

        // Add messages that will exceed context window
        for i in 0..10 {
            let msg = Message::text(MessageRole::User, format!("Message {}", i).repeat(10));
            engine.add_message(msg).unwrap();
        }

        // Should have trimmed some messages
        assert!(engine.state().total_tokens() <= 100);
    }

    #[test]
    fn test_estimate_tokens() {
        let msg = Message::text(MessageRole::User, "Hello world");
        let tokens = ConversationEngine::estimate_tokens(&msg);
        assert!(tokens.input > 0);
        assert_eq!(tokens.output, 0);
    }
}

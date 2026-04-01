//! Core type definitions for HiveCode

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the role of a message participant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// User-initiated message
    User,
    /// Assistant (AI model) response
    Assistant,
    /// System prompt or instruction
    System,
    /// Tool call or tool result
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// Represents a block of content within a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text content
    Text { content: String },
    /// Tool use invocation by the assistant
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Result from executing a tool
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
}

impl ContentBlock {
    /// Get the text representation of this content block
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { content } => Some(content),
            _ => None,
        }
    }
}

/// Represents a single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: String,
    /// Message role (user, assistant, system, or tool)
    pub role: MessageRole,
    /// Content blocks in this message
    pub content: Vec<ContentBlock>,
    /// When this message was created
    pub timestamp: DateTime<Utc>,
    /// Optional parent message ID for threading
    pub parent_id: Option<String>,
    /// Metadata about tokens consumed by this message
    pub tokens: Option<TokenCount>,
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: Vec<ContentBlock>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            parent_id: None,
            tokens: None,
        }
    }

    /// Create a simple text message
    pub fn text(role: MessageRole, text: impl Into<String>) -> Self {
        Self::new(role, vec![ContentBlock::Text { content: text.into() }])
    }

    /// Get all text content combined
    pub fn get_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| block.as_text())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Token counts for a message or conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCount {
    /// Input tokens consumed
    pub input: u64,
    /// Output tokens generated
    pub output: u64,
    /// Total tokens (input + output)
    pub total: u64,
}

impl TokenCount {
    /// Create a new token count
    pub fn new(input: u64, output: u64) -> Self {
        Self {
            input,
            output,
            total: input + output,
        }
    }

    /// Add another token count to this one
    pub fn add(&mut self, other: &TokenCount) {
        self.input += other.input;
        self.output += other.output;
        self.total += other.total;
    }
}

/// Information about an available provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    /// Provider identifier (e.g., "openai", "anthropic")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Whether this provider is currently available
    pub available: bool,
    /// List of available models for this provider
    pub models: Vec<String>,
    /// Configuration for this provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Session state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSessionState {
    /// Unique session identifier
    pub session_id: String,
    /// When the session started
    pub created_at: DateTime<Utc>,
    /// When the session was last active
    pub last_active: DateTime<Utc>,
    /// Current working directory for this session
    pub working_directory: String,
    /// Session-level environment variables
    pub env_vars: HashMap<String, String>,
}

impl AppSessionState {
    /// Create a new session
    pub fn new(working_directory: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            last_active: now,
            working_directory: working_directory.into(),
            env_vars: HashMap::new(),
        }
    }

    /// Update the last active timestamp
    pub fn touch(&mut self) {
        self.last_active = Utc::now();
    }
}

/// Metadata about a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    /// Conversation identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// When the conversation was created
    pub created_at: DateTime<Utc>,
    /// Model being used for this conversation
    pub model: String,
    /// Total tokens used in this conversation
    pub total_tokens: u64,
    /// Number of messages in this conversation
    pub message_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::System.to_string(), "system");
        assert_eq!(MessageRole::Tool.to_string(), "tool");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::text(MessageRole::User, "Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.get_text(), "Hello");
        assert!(!msg.id.is_empty());
    }

    #[test]
    fn test_content_block_as_text() {
        let block = ContentBlock::Text {
            content: "test".to_string(),
        };
        assert_eq!(block.as_text(), Some("test"));

        let tool_block = ContentBlock::ToolUse {
            id: "1".to_string(),
            name: "test".to_string(),
            input: serde_json::json!({}),
        };
        assert_eq!(tool_block.as_text(), None);
    }

    #[test]
    fn test_token_count() {
        let mut count = TokenCount::new(100, 50);
        assert_eq!(count.input, 100);
        assert_eq!(count.output, 50);
        assert_eq!(count.total, 150);

        let mut other = TokenCount::new(25, 25);
        count.add(&other);
        assert_eq!(count.input, 125);
        assert_eq!(count.output, 75);
        assert_eq!(count.total, 200);
    }

    #[test]
    fn test_session_state() {
        let mut session = AppSessionState::new("/home/user");
        assert_eq!(session.working_directory, "/home/user");
        let original_time = session.last_active;
        std::thread::sleep(std::time::Duration::from_millis(1));
        session.touch();
        assert!(session.last_active > original_time);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::text(MessageRole::User, "test");
        let json = serde_json::to_string(&msg).unwrap();
        let restored: Message = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.role, MessageRole::User);
        assert_eq!(restored.get_text(), "test");
    }
}

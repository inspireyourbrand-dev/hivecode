//! Common types for LLM providers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message sender (user, assistant, system)
    pub role: String,
    /// The content of the message
    pub content: Vec<ContentPart>,
}

impl ChatMessage {
    /// Create a user message with text content
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: vec![ContentPart::Text(text.into())],
        }
    }

    /// Create an assistant message with text content
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: vec![ContentPart::Text(text.into())],
        }
    }

    /// Create a system message with text content
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: vec![ContentPart::Text(text.into())],
        }
    }
}

/// A part of message content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content
    Text(String),

    /// Image content
    Image {
        /// URL or data URI of the image
        url: String,
        /// Media type (e.g., "image/png")
        media_type: String,
    },

    /// Tool use (function call) from the model
    ToolUse {
        /// Unique identifier for this tool use
        id: String,
        /// Name of the tool/function
        name: String,
        /// Input to the tool as JSON
        input: serde_json::Value,
    },

    /// Tool result (response from a tool call)
    ToolResult {
        /// ID of the tool use this result is for
        tool_use_id: String,
        /// The result content
        content: String,
        /// Whether this is an error result
        is_error: bool,
    },
}

/// Information about an available model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: String,
    /// Human-readable model name
    pub name: String,
    /// Maximum context window in tokens
    pub context_window: usize,
    /// Whether the model supports tool use
    pub supports_tools: bool,
    /// Whether the model supports vision/images
    pub supports_vision: bool,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
}

/// Capabilities supported by a provider.
#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether tool use is supported
    pub supports_tools: bool,
    /// Whether vision/images are supported
    pub supports_vision: bool,
    /// Whether function calling is supported
    pub supports_function_calling: bool,
    /// Maximum context window in tokens
    pub max_context_window: usize,
}

/// Token usage information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Number of input tokens used
    pub input_tokens: usize,
    /// Number of output tokens used
    pub output_tokens: usize,
    /// Number of cache creation tokens (if applicable)
    #[serde(default)]
    pub cache_creation_tokens: usize,
    /// Number of cache read tokens (if applicable)
    #[serde(default)]
    pub cache_read_tokens: usize,
}

/// The reason the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// Natural end of message
    EndTurn,
    /// Model requested tool use
    ToolUse,
    /// Maximum tokens reached
    MaxTokens,
    /// Stop sequence encountered
    StopSequence,
}

/// A tool/function definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Input schema (JSON Schema)
    pub input_schema: serde_json::Value,
}

/// A request to an LLM provider.
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// Model identifier
    pub model: String,
    /// Conversation history
    pub messages: Vec<ChatMessage>,
    /// Available tools/functions
    pub tools: Option<Vec<ToolDefinition>>,
    /// Temperature (0.0 to 2.0)
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<usize>,
    /// Whether to stream the response
    pub stream: bool,
}

impl ChatRequest {
    /// Create a new chat request with defaults
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            tools: None,
            temperature: Some(1.0),
            max_tokens: None,
            stream: false,
        }
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Add tools
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Enable streaming
    pub fn with_stream(mut self) -> Self {
        self.stream = true;
        self
    }
}

/// A response from an LLM provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Unique identifier for this response
    pub id: String,
    /// Model that generated the response
    pub model: String,
    /// Response content
    pub content: Vec<ContentPart>,
    /// Token usage information
    pub usage: Usage,
    /// Reason the model stopped generating
    pub stop_reason: StopReason,
}

/// An event in a streaming response.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Message generation started
    MessageStart {
        /// Message ID
        id: String,
        /// Model name
        model: String,
    },

    /// Text content delta
    ContentDelta(String),

    /// Tool use started
    ToolUseStart {
        /// Tool use ID
        id: String,
        /// Tool name
        name: String,
    },

    /// Tool use input delta (JSON string)
    ToolUseInputDelta(String),

    /// Tool use ended
    ToolUseEnd,

    /// Token usage information
    Usage(Usage),

    /// Message generation completed
    MessageEnd {
        /// Stop reason
        stop_reason: StopReason,
    },

    /// Error occurred
    Error(String),
}

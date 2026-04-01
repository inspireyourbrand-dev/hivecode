//! Anthropic Claude provider.

use crate::traits::LlmProvider;
use crate::types::*;
use crate::{ProviderError, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, error, warn};

const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic provider configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API key (required)
    pub api_key: String,
    /// Default model to use
    pub default_model: String,
    /// Custom base URL (optional, defaults to Anthropic's API)
    pub base_url: Option<String>,
}

impl AnthropicConfig {
    /// Create a new Anthropic configuration
    pub fn new(api_key: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            default_model: default_model.into(),
            base_url: None,
        }
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }
}

/// Anthropic provider
pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: AnthropicConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Get a builder for this provider
    pub fn builder() -> AnthropicProviderBuilder {
        AnthropicProviderBuilder::default()
    }

    fn get_base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
    }

    fn map_response(&self, resp: AnthropicMessage) -> Result<ChatResponse> {
        let id = resp.id;
        let model = resp.model;

        let content = resp
            .content
            .into_iter()
            .flat_map(|block| {
                match block {
                    ContentBlock::Text { text } => vec![ContentPart::Text(text)],
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input,
                    } => vec![ContentPart::ToolUse {
                        id,
                        name,
                        input,
                    }],
                }
            })
            .collect();

        let usage = resp.usage.into();

        let stop_reason = match resp.stop_reason.as_deref() {
            Some("end_turn") => StopReason::EndTurn,
            Some("tool_use") => StopReason::ToolUse,
            Some("max_tokens") => StopReason::MaxTokens,
            Some("stop_sequence") => StopReason::StopSequence,
            _ => StopReason::EndTurn,
        };

        Ok(ChatResponse {
            id,
            model,
            content,
            usage,
            stop_reason,
        })
    }
}

/// Builder for AnthropicProvider
pub struct AnthropicProviderBuilder {
    api_key: String,
    default_model: String,
    base_url: Option<String>,
}

impl Default for AnthropicProviderBuilder {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
        }
    }
}

impl AnthropicProviderBuilder {
    /// Set the API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = key.into();
        self
    }

    /// Set the default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Set a custom base URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Build the provider
    pub fn build(self) -> Result<AnthropicProvider> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidConfig(
                "API key is required".to_string(),
            ));
        }

        Ok(AnthropicProvider::new(AnthropicConfig {
            api_key: self.api_key,
            default_model: self.default_model,
            base_url: self.base_url,
        }))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: self.config.default_model.clone(),
            name: self.config.default_model.clone(),
            context_window: 200000,
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        }]
    }

    async fn chat(&self, mut request: ChatRequest) -> Result<ChatResponse> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        let anthropic_request = AnthropicChatRequest::from_chat_request(request, false)?;

        let url = format!("{}/v1/messages", self.get_base_url());

        let mut req_builder = self.client.post(&url).json(&anthropic_request);

        req_builder = req_builder
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION);

        debug!("Sending chat request to {}", url);

        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request: {}", e);
                e.into()
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, text);
            return Err(ProviderError::HttpError(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let anthropic_response: AnthropicMessage = response.json().await?;
        self.map_response(anthropic_response)
    }

    async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        request.stream = true;

        let anthropic_request = AnthropicChatRequest::from_chat_request(request, true)?;

        let url = format!("{}/v1/messages", self.get_base_url());

        let mut req_builder = self.client.post(&url).json(&anthropic_request);

        req_builder = req_builder
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION);

        debug!("Sending streaming chat request to {}", url);

        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request: {}", e);
                e.into()
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("API error {}: {}", status, text);
            return Err(ProviderError::HttpError(format!(
                "API error {}: {}",
                status, text
            )));
        }

        let stream = response.bytes_stream().map(move |result| {
            match result {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    let lines = text.lines();

                    let mut events = Vec::new();
                    for line in lines {
                        if line.is_empty() {
                            continue;
                        }

                        if let Some(data) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<AnthropicStreamEvent>(data) {
                                Ok(event) => {
                                    if let Some(stream_evt) = event.to_stream_event() {
                                        events.push(Ok(stream_evt));
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse SSE chunk: {}", e);
                                }
                            }
                        }
                    }

                    futures::stream::iter(events)
                }
                Err(e) => {
                    let err = Err(ProviderError::StreamError(e.to_string()));
                    futures::stream::iter(vec![err])
                }
            }
        });

        let flattened = stream.flatten();
        Ok(Box::pin(flattened))
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: true,
            supports_tools: true,
            supports_vision: true,
            supports_function_calling: true,
            max_context_window: 200000,
        }
    }
}

// ===== Anthropic API Types =====

#[derive(Debug, Serialize)]
pub struct AnthropicChatRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "is_false")]
    pub stream: bool,
}

fn is_false(b: &bool) -> bool {
    !b
}

impl AnthropicChatRequest {
    pub fn from_chat_request(req: ChatRequest, stream: bool) -> Result<Self> {
        let mut system = None;
        let mut messages = Vec::new();

        for msg in req.messages {
            if msg.role == "system" {
                system = msg
                    .content
                    .iter()
                    .find_map(|p| match p {
                        ContentPart::Text(t) => Some(t.clone()),
                        _ => None,
                    });
            } else {
                messages.push(AnthropicMessage::from_chat_message(msg)?);
            }
        }

        let tools = req.tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| AnthropicTool {
                    name: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })
                .collect()
        });

        Ok(AnthropicChatRequest {
            model: req.model,
            messages,
            system,
            tools,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: AnthropicUsage,
}

impl AnthropicMessage {
    pub fn from_chat_message(msg: ChatMessage) -> Result<Self> {
        let content = msg
            .content
            .into_iter()
            .filter_map(|part| match part {
                ContentPart::Text(text) => Some(ContentBlock::Text { text }),
                ContentPart::ToolUse { id, name, input } => {
                    Some(ContentBlock::ToolUse { id, name, input })
                }
                ContentPart::ToolResult { content, is_error, .. } => {
                    Some(ContentBlock::ToolResult {
                        content,
                        is_error,
                    })
                }
                _ => None,
            })
            .collect();

        Ok(AnthropicMessage {
            role: msg.role,
            content,
            id: uuid::Uuid::new_v4().to_string(),
            model: String::new(),
            stop_reason: None,
            usage: Default::default(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    #[serde(rename = "tool_result")]
    ToolResult { content: String, is_error: bool },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AnthropicUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl From<AnthropicUsage> for Usage {
    fn from(u: AnthropicUsage) -> Self {
        Usage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicStreamEvent {
    MessageStart { message: AnthropicMessage },
    ContentBlockStart { index: usize, content_block: ContentBlockStart },
    ContentBlockDelta { index: usize, delta: ContentBlockDelta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: MessageDelta },
    MessageStop,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockStart {
    Text,
    ToolUse { id: String, name: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDelta {
    TextDelta { text: String },
    ToolInputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
pub struct MessageDelta {
    pub stop_reason: Option<String>,
    pub usage: Option<AnthropicUsage>,
}

impl AnthropicStreamEvent {
    pub fn to_stream_event(&self) -> Option<StreamEvent> {
        match self {
            AnthropicStreamEvent::MessageStart { message } => Some(StreamEvent::MessageStart {
                id: message.id.clone(),
                model: message.model.clone(),
            }),
            AnthropicStreamEvent::ContentBlockStart { content_block, .. } => match content_block {
                ContentBlockStart::Text => None,
                ContentBlockStart::ToolUse { id, name } => Some(StreamEvent::ToolUseStart {
                    id: id.clone(),
                    name: name.clone(),
                }),
            },
            AnthropicStreamEvent::ContentBlockDelta { delta, .. } => match delta {
                ContentBlockDelta::TextDelta { text } => {
                    Some(StreamEvent::ContentDelta(text.clone()))
                }
                ContentBlockDelta::ToolInputJsonDelta { partial_json } => {
                    Some(StreamEvent::ToolUseInputDelta(partial_json.clone()))
                }
            },
            AnthropicStreamEvent::ContentBlockStop { .. } => Some(StreamEvent::ToolUseEnd),
            AnthropicStreamEvent::MessageDelta { delta } => {
                if let Some(usage) = &delta.usage {
                    Some(StreamEvent::Usage(usage.clone().into()))
                } else {
                    None
                }
            }
            AnthropicStreamEvent::MessageStop => Some(StreamEvent::MessageEnd {
                stop_reason: StopReason::EndTurn,
            }),
        }
    }
}

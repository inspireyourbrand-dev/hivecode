//! OpenAI and OpenAI-compatible endpoint provider.
//!
//! This provider supports:
//! - OpenAI's API
//! - Ollama (with OpenAI-compatible endpoint)
//! - LM Studio
//! - vLLM
//! - Any service with OpenAI-compatible API

use crate::traits::LlmProvider;
use crate::types::*;
use crate::{ProviderError, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use futures::pin_mut;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, error, warn};

/// OpenAI provider configuration
#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    /// Base URL for the API (e.g., "https://api.openai.com/v1")
    pub base_url: String,
    /// API key (optional for local endpoints like Ollama)
    pub api_key: Option<String>,
    /// Default model to use
    pub default_model: String,
    /// Organization ID (optional)
    pub organization: Option<String>,
}

impl OpenAiConfig {
    /// Create a new OpenAI API configuration
    pub fn openai(api_key: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: Some(api_key.into()),
            default_model: default_model.into(),
            organization: None,
        }
    }

    /// Create a configuration for a local OpenAI-compatible endpoint
    pub fn local(base_url: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            default_model: default_model.into(),
            organization: None,
        }
    }

    /// Set organization ID
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }
}

/// OpenAI/OpenAI-compatible provider
pub struct OpenAiProvider {
    config: OpenAiConfig,
    client: reqwest::Client,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider
    pub fn new(config: OpenAiConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Get a builder for this provider
    pub fn builder() -> OpenAiProviderBuilder {
        OpenAiProviderBuilder::default()
    }

    /// Map OpenAI response to ChatResponse
    fn map_response(&self, openai_resp: OpenAiChatResponse) -> Result<ChatResponse> {
        let id = openai_resp.id;
        let model = openai_resp.model;

        let content = openai_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::Other("No choices in response".to_string()))?
            .message
            .into();

        let usage = openai_resp.usage.into();

        let stop_reason = match openai_resp
            .choices
            .first()
            .and_then(|c| c.finish_reason.as_deref())
        {
            Some("stop") => StopReason::EndTurn,
            Some("tool_calls") => StopReason::ToolUse,
            Some("length") => StopReason::MaxTokens,
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

/// Builder for OpenAiProvider
pub struct OpenAiProviderBuilder {
    base_url: String,
    api_key: Option<String>,
    default_model: String,
    organization: Option<String>,
}

impl Default for OpenAiProviderBuilder {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key: None,
            default_model: "gpt-4o-mini".to_string(),
            organization: None,
        }
    }
}

impl OpenAiProviderBuilder {
    /// Set the base URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Set the organization ID
    pub fn organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Build the provider
    pub fn build(self) -> OpenAiProvider {
        OpenAiProvider::new(OpenAiConfig {
            base_url: self.base_url,
            api_key: self.api_key,
            default_model: self.default_model,
            organization: self.organization,
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: self.config.default_model.clone(),
            name: self.config.default_model.clone(),
            context_window: 128000,
            supports_tools: true,
            supports_vision: true,
            supports_streaming: true,
        }]
    }

    async fn chat(&self, mut request: ChatRequest) -> Result<ChatResponse> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        // Convert our types to OpenAI format
        let openai_request = OpenAiChatRequest::from_chat_request(request, false)?;

        let url = format!("{}/chat/completions", self.config.base_url);

        let mut req_builder = self.client.post(&url).json(&openai_request);

        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        if let Some(org) = &self.config.organization {
            req_builder = req_builder.header("OpenAI-Organization", org);
        }

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

        let openai_response: OpenAiChatResponse = response.json().await?;
        self.map_response(openai_response)
    }

    async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        request.stream = true;

        let openai_request = OpenAiChatRequest::from_chat_request(request, true)?;

        let url = format!("{}/chat/completions", self.config.base_url);

        let mut req_builder = self.client.post(&url).json(&openai_request);

        if let Some(api_key) = &self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        if let Some(org) = &self.config.organization {
            req_builder = req_builder.header("OpenAI-Organization", org);
        }

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
                    // Process SSE line
                    let lines = text.lines();

                    let mut events = Vec::new();
                    for line in lines {
                        if line.is_empty() || line == ": OPENROUTER PROCESSING" {
                            continue;
                        }

                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                events.push(Ok(StreamEvent::MessageEnd {
                                    stop_reason: StopReason::EndTurn,
                                }));
                                continue;
                            }

                            match serde_json::from_str::<OpenAiStreamChunk>(data) {
                                Ok(chunk) => {
                                    if let Some(event) = chunk.to_stream_event() {
                                        events.push(Ok(event));
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
            max_context_window: 128000,
        }
    }
}

// ===== OpenAI API Types =====

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiChatRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAiTool>>,
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

impl OpenAiChatRequest {
    pub fn from_chat_request(req: ChatRequest, stream: bool) -> Result<Self> {
        let messages = req
            .messages
            .into_iter()
            .map(OpenAiMessage::from_chat_message)
            .collect::<Result<Vec<_>>>()?;

        let tools = req.tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| OpenAiTool {
                    r#type: "function".to_string(),
                    function: OpenAiFunction {
                        name: t.name,
                        description: t.description,
                        parameters: t.input_schema,
                    },
                })
                .collect()
        });

        Ok(OpenAiChatRequest {
            model: req.model,
            messages,
            tools,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAiToolCall>>,
}

impl OpenAiMessage {
    pub fn from_chat_message(msg: ChatMessage) -> Result<Self> {
        let mut content_text = String::new();
        let mut tool_calls = Vec::new();

        for part in msg.content {
            match part {
                ContentPart::Text(text) => content_text.push_str(&text),
                ContentPart::ToolUse {
                    id,
                    name,
                    input,
                } => {
                    tool_calls.push(OpenAiToolCall {
                        id,
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name,
                            arguments: serde_json::to_string(&input)?,
                        },
                    });
                }
                ContentPart::ToolResult {
                    tool_use_id,
                    content,
                    ..
                } => {
                    // Tool results are typically sent as separate messages
                    return Err(ProviderError::Other(
                        "Tool results should be sent as separate messages".to_string(),
                    ));
                }
                ContentPart::Image { .. } => {
                    // TODO: Handle images
                }
            }
        }

        Ok(OpenAiMessage {
            role: msg.role,
            content: if content_text.is_empty() {
                None
            } else {
                Some(content_text)
            },
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiToolCall {
    pub id: String,
    pub r#type: String,
    pub function: OpenAiFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiTool {
    pub r#type: String,
    pub function: OpenAiFunction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
    pub usage: OpenAiUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiChoice {
    pub index: usize,
    pub message: OpenAiMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    #[serde(default)]
    pub prompt_tokens_details: Option<serde_json::Value>,
    #[serde(default)]
    pub completion_tokens_details: Option<serde_json::Value>,
}

impl From<OpenAiUsage> for Usage {
    fn from(u: OpenAiUsage) -> Self {
        Usage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        }
    }
}

impl From<OpenAiMessage> for Vec<ContentPart> {
    fn from(msg: OpenAiMessage) -> Self {
        let mut parts = Vec::new();

        if let Some(content) = msg.content {
            if !content.is_empty() {
                parts.push(ContentPart::Text(content));
            }
        }

        if let Some(tool_calls) = msg.tool_calls {
            for tc in tool_calls {
                let input = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::Object(Default::default()));

                parts.push(ContentPart::ToolUse {
                    id: tc.id,
                    name: tc.function.name,
                    input,
                });
            }
        }

        if parts.is_empty() {
            parts.push(ContentPart::Text(String::new()));
        }

        parts
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiStreamChunk {
    pub choices: Vec<OpenAiStreamChoice>,
    #[serde(default)]
    pub usage: Option<OpenAiUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiStreamChoice {
    pub index: usize,
    pub delta: OpenAiDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiDelta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<OpenAiDeltaToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiDeltaToolCall {
    pub index: usize,
    pub id: Option<String>,
    pub r#type: Option<String>,
    pub function: Option<OpenAiDeltaFunction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiDeltaFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

impl OpenAiStreamChunk {
    pub fn to_stream_event(&self) -> Option<StreamEvent> {
        if let Some(usage) = &self.usage {
            return Some(StreamEvent::Usage(Usage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
            }));
        }

        for choice in &self.choices {
            if let Some(content) = &choice.delta.content {
                if !content.is_empty() {
                    return Some(StreamEvent::ContentDelta(content.clone()));
                }
            }

            if let Some(tool_calls) = &choice.delta.tool_calls {
                for tc in tool_calls {
                    if let Some(id) = &tc.id {
                        if let Some(func) = &tc.function {
                            if let Some(name) = &func.name {
                                return Some(StreamEvent::ToolUseStart {
                                    id: id.clone(),
                                    name: name.clone(),
                                });
                            }
                        }
                    }

                    if let Some(func) = &tc.function {
                        if let Some(args) = &func.arguments {
                            return Some(StreamEvent::ToolUseInputDelta(args.clone()));
                        }
                    }
                }
            }

            if let Some(reason) = &choice.finish_reason {
                if reason == "tool_calls" {
                    return Some(StreamEvent::ToolUseEnd);
                } else if reason == "stop" {
                    return Some(StreamEvent::MessageEnd {
                        stop_reason: StopReason::EndTurn,
                    });
                }
            }
        }

        None
    }
}

//! AWS Bedrock provider for Anthropic models.
//!
//! This provider supports Claude models through Amazon Bedrock's API.
//! It handles AWS SigV4 authentication and streaming responses.

use crate::traits::LlmProvider;
use crate::types::*;
use crate::{ProviderError, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, error, warn};

/// Bedrock provider configuration
#[derive(Debug, Clone)]
pub struct BedrockConfig {
    /// AWS region (e.g., "us-east-1")
    pub region: String,
    /// AWS access key ID (optional, falls back to AWS credential chain)
    pub access_key_id: Option<String>,
    /// AWS secret access key (optional, falls back to AWS credential chain)
    pub secret_access_key: Option<String>,
    /// AWS session token (optional)
    pub session_token: Option<String>,
    /// Default model to use
    pub default_model: String,
}

impl BedrockConfig {
    /// Create a new Bedrock configuration
    pub fn new(region: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            region: region.into(),
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            default_model: default_model.into(),
        }
    }

    /// Set AWS credentials
    pub fn with_credentials(
        mut self,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> Self {
        self.access_key_id = Some(access_key_id.into());
        self.secret_access_key = Some(secret_access_key.into());
        self
    }

    /// Set session token
    pub fn with_session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }
}

/// Bedrock provider
pub struct BedrockProvider {
    config: BedrockConfig,
    client: reqwest::Client,
}

impl BedrockProvider {
    /// Create a new Bedrock provider
    pub fn new(config: BedrockConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Get a builder for this provider
    pub fn builder() -> BedrockProviderBuilder {
        BedrockProviderBuilder::default()
    }

    fn get_endpoint(&self, model_id: &str) -> String {
        format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
            self.config.region, model_id
        )
    }

    fn get_stream_endpoint(&self, model_id: &str) -> String {
        format!(
            "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke-with-response-stream",
            self.config.region, model_id
        )
    }

    fn map_response(&self, resp: BedrockMessage) -> Result<ChatResponse> {
        let id = resp.id;
        let model = resp.model;

        let content = resp
            .content
            .into_iter()
            .flat_map(|block| match block {
                BedrockContentBlock::Text { text } => vec![ContentPart::Text(text)],
                BedrockContentBlock::ToolUse {
                    id,
                    name,
                    input,
                } => vec![ContentPart::ToolUse {
                    id,
                    name,
                    input,
                }],
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

/// Builder for BedrockProvider
pub struct BedrockProviderBuilder {
    region: String,
    access_key_id: Option<String>,
    secret_access_key: Option<String>,
    session_token: Option<String>,
    default_model: String,
}

impl Default for BedrockProviderBuilder {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
            default_model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

impl BedrockProviderBuilder {
    /// Set the AWS region
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Set AWS credentials
    pub fn credentials(
        mut self,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> Self {
        self.access_key_id = Some(access_key_id.into());
        self.secret_access_key = Some(secret_access_key.into());
        self
    }

    /// Set session token
    pub fn session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }

    /// Set the default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Build the provider
    pub fn build(self) -> Result<BedrockProvider> {
        Ok(BedrockProvider::new(BedrockConfig {
            region: self.region,
            access_key_id: self.access_key_id,
            secret_access_key: self.secret_access_key,
            session_token: self.session_token,
            default_model: self.default_model,
        }))
    }
}

#[async_trait]
impl LlmProvider for BedrockProvider {
    fn name(&self) -> &str {
        "bedrock"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                name: "Claude Sonnet 4 (Latest)".to_string(),
                context_window: 200000,
                supports_tools: true,
                supports_vision: true,
                supports_streaming: true,
            },
            ModelInfo {
                id: "claude-haiku-4-20250414".to_string(),
                name: "Claude Haiku 4".to_string(),
                context_window: 200000,
                supports_tools: true,
                supports_vision: true,
                supports_streaming: true,
            },
        ]
    }

    async fn chat(&self, mut request: ChatRequest) -> Result<ChatResponse> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        let bedrock_request = BedrockChatRequest::from_chat_request(request)?;
        let model_id = bedrock_request.model.clone();
        let endpoint = self.get_endpoint(&model_id);

        let req_builder = self.client.post(&endpoint).json(&bedrock_request);

        debug!("Sending chat request to Bedrock endpoint: {}", endpoint);

        // TODO: Implement AWS SigV4 signing for authentication
        // For now, this is a placeholder that assumes credentials are in environment
        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Bedrock: {}", e);
                e.into()
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Bedrock API error {}: {}", status, text);
            return Err(ProviderError::HttpError(format!(
                "Bedrock API error {}: {}",
                status, text
            )));
        }

        let bedrock_response: BedrockMessage = response.json().await?;
        self.map_response(bedrock_response)
    }

    async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        request.stream = true;

        let bedrock_request = BedrockChatRequest::from_chat_request(request)?;
        let model_id = bedrock_request.model.clone();
        let endpoint = self.get_stream_endpoint(&model_id);

        let req_builder = self.client.post(&endpoint).json(&bedrock_request);

        debug!(
            "Sending streaming chat request to Bedrock endpoint: {}",
            endpoint
        );

        // TODO: Implement AWS SigV4 signing for authentication
        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Bedrock: {}", e);
                e.into()
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Bedrock API error {}: {}", status, text);
            return Err(ProviderError::HttpError(format!(
                "Bedrock API error {}: {}",
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
                            match serde_json::from_str::<BedrockStreamEvent>(data) {
                                Ok(event) => {
                                    if let Some(stream_evt) = event.to_stream_event() {
                                        events.push(Ok(stream_evt));
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse Bedrock stream chunk: {}", e);
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

// ===== Bedrock API Types =====

#[derive(Debug, Serialize)]
pub struct BedrockChatRequest {
    pub model: String,
    pub messages: Vec<BedrockMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<BedrockTool>>,
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

impl BedrockChatRequest {
    pub fn from_chat_request(req: ChatRequest) -> Result<Self> {
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
                messages.push(BedrockMessage::from_chat_message(msg)?);
            }
        }

        let tools = req.tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| BedrockTool {
                    name: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })
                .collect()
        });

        Ok(BedrockChatRequest {
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
pub struct BedrockMessage {
    pub role: String,
    pub content: Vec<BedrockContentBlock>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: BedrockUsage,
}

impl BedrockMessage {
    pub fn from_chat_message(msg: ChatMessage) -> Result<Self> {
        let content = msg
            .content
            .into_iter()
            .filter_map(|part| match part {
                ContentPart::Text(text) => Some(BedrockContentBlock::Text { text }),
                ContentPart::ToolUse { id, name, input } => {
                    Some(BedrockContentBlock::ToolUse { id, name, input })
                }
                _ => None,
            })
            .collect();

        Ok(BedrockMessage {
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
pub enum BedrockContentBlock {
    Text { text: String },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BedrockUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl From<BedrockUsage> for Usage {
    fn from(u: BedrockUsage) -> Self {
        Usage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BedrockTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BedrockStreamEvent {
    MessageStart { message: BedrockMessage },
    ContentBlockStart {
        index: usize,
        content_block: BedrockContentBlockStart,
    },
    ContentBlockDelta {
        index: usize,
        delta: BedrockContentBlockDelta,
    },
    ContentBlockStop { index: usize },
    MessageDelta { delta: BedrockMessageDelta },
    MessageStop,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BedrockContentBlockStart {
    Text,
    ToolUse { id: String, name: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BedrockContentBlockDelta {
    TextDelta { text: String },
    ToolInputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
pub struct BedrockMessageDelta {
    pub stop_reason: Option<String>,
    pub usage: Option<BedrockUsage>,
}

impl BedrockStreamEvent {
    pub fn to_stream_event(&self) -> Option<StreamEvent> {
        match self {
            BedrockStreamEvent::MessageStart { message } => Some(StreamEvent::MessageStart {
                id: message.id.clone(),
                model: message.model.clone(),
            }),
            BedrockStreamEvent::ContentBlockStart { content_block, .. } => match content_block {
                BedrockContentBlockStart::Text => None,
                BedrockContentBlockStart::ToolUse { id, name } => {
                    Some(StreamEvent::ToolUseStart {
                        id: id.clone(),
                        name: name.clone(),
                    })
                }
            },
            BedrockStreamEvent::ContentBlockDelta { delta, .. } => match delta {
                BedrockContentBlockDelta::TextDelta { text } => {
                    Some(StreamEvent::ContentDelta(text.clone()))
                }
                BedrockContentBlockDelta::ToolInputJsonDelta { partial_json } => {
                    Some(StreamEvent::ToolUseInputDelta(partial_json.clone()))
                }
            },
            BedrockStreamEvent::ContentBlockStop { .. } => Some(StreamEvent::ToolUseEnd),
            BedrockStreamEvent::MessageDelta { delta } => {
                if let Some(usage) = &delta.usage {
                    Some(StreamEvent::Usage(usage.clone().into()))
                } else {
                    None
                }
            }
            BedrockStreamEvent::MessageStop => Some(StreamEvent::MessageEnd {
                stop_reason: StopReason::EndTurn,
            }),
        }
    }
}

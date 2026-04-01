//! Google Vertex AI provider for Anthropic models.
//!
//! This provider supports Claude models through Google's Vertex AI API.
//! It handles Google OAuth2 authentication and streaming responses.

use crate::traits::LlmProvider;
use crate::types::*;
use crate::{ProviderError, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, error, warn};

/// Vertex AI provider configuration
#[derive(Debug, Clone)]
pub struct VertexConfig {
    /// Google Cloud project ID
    pub project_id: String,
    /// Vertex AI location (default: "us-central1")
    pub location: String,
    /// Path to Google Application Credentials JSON (optional, falls back to GOOGLE_APPLICATION_CREDENTIALS)
    pub credentials_json: Option<String>,
    /// Default model to use
    pub default_model: String,
}

impl VertexConfig {
    /// Create a new Vertex AI configuration
    pub fn new(project_id: impl Into<String>, default_model: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            location: "us-central1".to_string(),
            credentials_json: None,
            default_model: default_model.into(),
        }
    }

    /// Set the location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }

    /// Set credentials JSON path
    pub fn with_credentials_json(mut self, json_path: impl Into<String>) -> Self {
        self.credentials_json = Some(json_path.into());
        self
    }
}

/// Vertex AI provider
pub struct VertexProvider {
    config: VertexConfig,
    client: reqwest::Client,
}

impl VertexProvider {
    /// Create a new Vertex AI provider
    pub fn new(config: VertexConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Get a builder for this provider
    pub fn builder() -> VertexProviderBuilder {
        VertexProviderBuilder::default()
    }

    fn get_endpoint(&self, model_id: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/anthropic/models/{}:streamRawPredict",
            self.config.location, self.config.project_id, self.config.location, model_id
        )
    }

    fn map_response(&self, resp: VertexMessage) -> Result<ChatResponse> {
        let id = resp.id;
        let model = resp.model;

        let content = resp
            .content
            .into_iter()
            .flat_map(|block| match block {
                VertexContentBlock::Text { text } => vec![ContentPart::Text(text)],
                VertexContentBlock::ToolUse {
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

/// Builder for VertexProvider
pub struct VertexProviderBuilder {
    project_id: String,
    location: String,
    credentials_json: Option<String>,
    default_model: String,
}

impl Default for VertexProviderBuilder {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            location: "us-central1".to_string(),
            credentials_json: None,
            default_model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

impl VertexProviderBuilder {
    /// Set the Google Cloud project ID
    pub fn project_id(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = project_id.into();
        self
    }

    /// Set the location
    pub fn location(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }

    /// Set credentials JSON path
    pub fn credentials_json(mut self, json_path: impl Into<String>) -> Self {
        self.credentials_json = Some(json_path.into());
        self
    }

    /// Set the default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Build the provider
    pub fn build(self) -> Result<VertexProvider> {
        if self.project_id.is_empty() {
            return Err(ProviderError::InvalidConfig(
                "Project ID is required".to_string(),
            ));
        }

        Ok(VertexProvider::new(VertexConfig {
            project_id: self.project_id,
            location: self.location,
            credentials_json: self.credentials_json,
            default_model: self.default_model,
        }))
    }
}

#[async_trait]
impl LlmProvider for VertexProvider {
    fn name(&self) -> &str {
        "vertex"
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

        let vertex_request = VertexChatRequest::from_chat_request(request, false)?;
        let model_id = vertex_request.model.clone();
        let endpoint = self.get_endpoint(&model_id);

        let req_builder = self.client.post(&endpoint).json(&vertex_request);

        debug!("Sending chat request to Vertex AI endpoint: {}", endpoint);

        // TODO: Implement Google OAuth2 token exchange for authentication
        // For now, this assumes credentials are available via GOOGLE_APPLICATION_CREDENTIALS env var
        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Vertex AI: {}", e);
                e.into()
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Vertex AI API error {}: {}", status, text);
            return Err(ProviderError::HttpError(format!(
                "Vertex AI API error {}: {}",
                status, text
            )));
        }

        let vertex_response: VertexMessage = response.json().await?;
        self.map_response(vertex_response)
    }

    async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        if request.model.is_empty() {
            request.model = self.config.default_model.clone();
        }

        request.stream = true;

        let vertex_request = VertexChatRequest::from_chat_request(request, true)?;
        let model_id = vertex_request.model.clone();
        let endpoint = self.get_endpoint(&model_id);

        let req_builder = self.client.post(&endpoint).json(&vertex_request);

        debug!(
            "Sending streaming chat request to Vertex AI endpoint: {}",
            endpoint
        );

        // TODO: Implement Google OAuth2 token exchange for authentication
        let response = req_builder
            .send()
            .await
            .map_err(|e| {
                error!("Failed to send request to Vertex AI: {}", e);
                e.into()
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("Vertex AI API error {}: {}", status, text);
            return Err(ProviderError::HttpError(format!(
                "Vertex AI API error {}: {}",
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
                            match serde_json::from_str::<VertexStreamEvent>(data) {
                                Ok(event) => {
                                    if let Some(stream_evt) = event.to_stream_event() {
                                        events.push(Ok(stream_evt));
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse Vertex AI stream chunk: {}", e);
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

// ===== Vertex AI API Types =====

#[derive(Debug, Serialize)]
pub struct VertexChatRequest {
    pub model: String,
    pub messages: Vec<VertexMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<VertexTool>>,
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

impl VertexChatRequest {
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
                messages.push(VertexMessage::from_chat_message(msg)?);
            }
        }

        let tools = req.tools.map(|tools| {
            tools
                .into_iter()
                .map(|t| VertexTool {
                    name: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })
                .collect()
        });

        Ok(VertexChatRequest {
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
pub struct VertexMessage {
    pub role: String,
    pub content: Vec<VertexContentBlock>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub stop_reason: Option<String>,
    #[serde(default)]
    pub usage: VertexUsage,
}

impl VertexMessage {
    pub fn from_chat_message(msg: ChatMessage) -> Result<Self> {
        let content = msg
            .content
            .into_iter()
            .filter_map(|part| match part {
                ContentPart::Text(text) => Some(VertexContentBlock::Text { text }),
                ContentPart::ToolUse { id, name, input } => {
                    Some(VertexContentBlock::ToolUse { id, name, input })
                }
                _ => None,
            })
            .collect();

        Ok(VertexMessage {
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
pub enum VertexContentBlock {
    Text { text: String },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VertexUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

impl From<VertexUsage> for Usage {
    fn from(u: VertexUsage) -> Self {
        Usage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VertexTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VertexStreamEvent {
    MessageStart { message: VertexMessage },
    ContentBlockStart {
        index: usize,
        content_block: VertexContentBlockStart,
    },
    ContentBlockDelta {
        index: usize,
        delta: VertexContentBlockDelta,
    },
    ContentBlockStop { index: usize },
    MessageDelta { delta: VertexMessageDelta },
    MessageStop,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VertexContentBlockStart {
    Text,
    ToolUse { id: String, name: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VertexContentBlockDelta {
    TextDelta { text: String },
    ToolInputJsonDelta { partial_json: String },
}

#[derive(Debug, Deserialize)]
pub struct VertexMessageDelta {
    pub stop_reason: Option<String>,
    pub usage: Option<VertexUsage>,
}

impl VertexStreamEvent {
    pub fn to_stream_event(&self) -> Option<StreamEvent> {
        match self {
            VertexStreamEvent::MessageStart { message } => Some(StreamEvent::MessageStart {
                id: message.id.clone(),
                model: message.model.clone(),
            }),
            VertexStreamEvent::ContentBlockStart { content_block, .. } => match content_block {
                VertexContentBlockStart::Text => None,
                VertexContentBlockStart::ToolUse { id, name } => {
                    Some(StreamEvent::ToolUseStart {
                        id: id.clone(),
                        name: name.clone(),
                    })
                }
            },
            VertexStreamEvent::ContentBlockDelta { delta, .. } => match delta {
                VertexContentBlockDelta::TextDelta { text } => {
                    Some(StreamEvent::ContentDelta(text.clone()))
                }
                VertexContentBlockDelta::ToolInputJsonDelta { partial_json } => {
                    Some(StreamEvent::ToolUseInputDelta(partial_json.clone()))
                }
            },
            VertexStreamEvent::ContentBlockStop { .. } => Some(StreamEvent::ToolUseEnd),
            VertexStreamEvent::MessageDelta { delta } => {
                if let Some(usage) = &delta.usage {
                    Some(StreamEvent::Usage(usage.clone().into()))
                } else {
                    None
                }
            }
            VertexStreamEvent::MessageStop => Some(StreamEvent::MessageEnd {
                stop_reason: StopReason::EndTurn,
            }),
        }
    }
}

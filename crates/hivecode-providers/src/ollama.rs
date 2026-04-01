//! Local Ollama provider.
//!
//! This provider wraps OpenAI-compatible endpoint support for local Ollama instances.

use crate::openai::{OpenAiConfig, OpenAiProvider};
use crate::traits::LlmProvider;
use crate::types::*;
use crate::{ProviderError, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use tracing::{debug, info};

/// Ollama provider configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Base URL for Ollama (e.g., "http://localhost:11434")
    pub base_url: String,
    /// Default model to use
    pub default_model: String,
}

impl OllamaConfig {
    /// Create a new Ollama configuration with default localhost
    pub fn new(default_model: impl Into<String>) -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            default_model: default_model.into(),
        }
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

/// Ollama provider (wraps OpenAI-compatible interface)
pub struct OllamaProvider {
    config: OllamaConfig,
    inner: OpenAiProvider,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(config: OllamaConfig) -> Self {
        let openai_config = OpenAiConfig::local(&config.base_url, &config.default_model);
        let inner = OpenAiProvider::new(openai_config);
        let client = reqwest::Client::new();

        Self {
            config,
            inner,
            client,
        }
    }

    /// Create a builder for Ollama provider
    pub fn builder() -> OllamaProviderBuilder {
        OllamaProviderBuilder::default()
    }

    /// List available models in the Ollama instance
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> {
        let url = format!("{}/api/tags", self.config.base_url);
        debug!("Fetching Ollama models from {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    ProviderError::ConnectionRefused(format!(
                        "Could not connect to Ollama at {}",
                        self.config.base_url
                    ))
                } else {
                    ProviderError::HttpError(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(ProviderError::HttpError(format!(
                "Failed to fetch models: {}",
                status
            )));
        }

        let data: OllamaTagsResponse = response.json().await?;
        info!(
            "Found {} models in Ollama instance",
            data.models.len()
        );
        Ok(data.models)
    }

    /// Check if Ollama is running
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.config.base_url);
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                if e.is_connect() {
                    Ok(false)
                } else {
                    Err(ProviderError::ConnectionRefused(e.to_string()))
                }
            }
        }
    }
}

/// Builder for OllamaProvider
pub struct OllamaProviderBuilder {
    base_url: String,
    default_model: String,
}

impl Default for OllamaProviderBuilder {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            default_model: "llama2".to_string(),
        }
    }
}

impl OllamaProviderBuilder {
    /// Set the base URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the default model
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Build the provider
    pub fn build(self) -> OllamaProvider {
        OllamaProvider::new(OllamaConfig {
            base_url: self.base_url,
            default_model: self.default_model,
        })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    fn models(&self) -> Vec<ModelInfo> {
        vec![ModelInfo {
            id: self.config.default_model.clone(),
            name: self.config.default_model.clone(),
            context_window: 4096,
            supports_tools: false,
            supports_vision: false,
            supports_streaming: true,
        }]
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        self.inner.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        self.inner.chat_stream(request).await
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            supports_streaming: true,
            supports_tools: false,
            supports_vision: false,
            supports_function_calling: false,
            max_context_window: 4096,
        }
    }
}

// ===== Ollama API Types =====

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub modified_at: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub details: Option<OllamaModelDetails>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModelDetails {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub family: String,
    #[serde(default)]
    pub families: Vec<String>,
    #[serde(default)]
    pub parameter_size: String,
    #[serde(default)]
    pub quantization_level: String,
}

#[derive(Debug, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaModel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_config() {
        let config = OllamaConfig::new("llama2");
        assert_eq!(config.default_model, "llama2");
        assert_eq!(config.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_builder() {
        let provider = OllamaProvider::builder()
            .base_url("http://custom:11434")
            .default_model("mistral")
            .build();

        assert_eq!(provider.config.base_url, "http://custom:11434");
        assert_eq!(provider.config.default_model, "mistral");
    }
}

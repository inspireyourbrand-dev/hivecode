//! Anthropic Foundry provider.
//!
//! This provider is a thin wrapper around the Anthropic provider with
//! Foundry-specific configuration (custom base URL for Foundry endpoints).
//! It reuses the Anthropic protocol and types, just with a different base URL.

use crate::anthropic::{AnthropicConfig, AnthropicProvider};
use crate::traits::LlmProvider;
use crate::types::*;
use crate::{ProviderError, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;
use tracing::debug;

/// Foundry provider configuration
#[derive(Debug, Clone)]
pub struct FoundryConfig {
    /// Foundry base URL (e.g., "https://api.anthropic.com/foundry")
    pub base_url: String,
    /// API key (required)
    pub api_key: String,
    /// Default model to use
    pub default_model: String,
}

impl FoundryConfig {
    /// Create a new Foundry configuration
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        default_model: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            default_model: default_model.into(),
        }
    }
}

/// Foundry provider (wrapper around Anthropic with custom base URL)
pub struct FoundryProvider {
    config: FoundryConfig,
    inner: AnthropicProvider,
}

impl FoundryProvider {
    /// Create a new Foundry provider
    pub fn new(config: FoundryConfig) -> Self {
        let anthropic_config = AnthropicConfig::new(&config.api_key, &config.default_model)
            .with_base_url(&config.base_url);

        let inner = AnthropicProvider::new(anthropic_config);

        Self { config, inner }
    }

    /// Get a builder for this provider
    pub fn builder() -> FoundryProviderBuilder {
        FoundryProviderBuilder::default()
    }
}

/// Builder for FoundryProvider
pub struct FoundryProviderBuilder {
    base_url: String,
    api_key: String,
    default_model: String,
}

impl Default for FoundryProviderBuilder {
    fn default() -> Self {
        Self {
            base_url: "https://api.anthropic.com".to_string(),
            api_key: String::new(),
            default_model: "claude-3-5-sonnet-20241022".to_string(),
        }
    }
}

impl FoundryProviderBuilder {
    /// Set the Foundry base URL
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

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

    /// Build the provider
    pub fn build(self) -> Result<FoundryProvider> {
        if self.api_key.is_empty() {
            return Err(ProviderError::InvalidConfig(
                "API key is required".to_string(),
            ));
        }

        Ok(FoundryProvider::new(FoundryConfig {
            base_url: self.base_url,
            api_key: self.api_key,
            default_model: self.default_model,
        }))
    }
}

#[async_trait]
impl LlmProvider for FoundryProvider {
    fn name(&self) -> &str {
        "foundry"
    }

    fn models(&self) -> Vec<ModelInfo> {
        self.inner.models()
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        debug!("Foundry provider: forwarding chat request to Anthropic");
        self.inner.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>> {
        debug!("Foundry provider: forwarding streaming chat request to Anthropic");
        self.inner.chat_stream(request).await
    }

    fn capabilities(&self) -> ProviderCapabilities {
        self.inner.capabilities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foundry_config() {
        let config = FoundryConfig::new(
            "https://api.anthropic.com/foundry",
            "test-key",
            "claude-3-5-sonnet-20241022",
        );
        assert_eq!(config.base_url, "https://api.anthropic.com/foundry");
        assert_eq!(config.api_key, "test-key");
    }

    #[test]
    fn test_foundry_builder() -> Result<()> {
        let provider = FoundryProvider::builder()
            .base_url("https://api.anthropic.com/foundry")
            .api_key("test-key")
            .default_model("claude-3-5-sonnet-20241022")
            .build()?;

        assert_eq!(provider.config.base_url, "https://api.anthropic.com/foundry");
        assert_eq!(provider.config.api_key, "test-key");
        Ok(())
    }
}

//! Core LLM provider trait and abstractions.

use crate::types::{ChatRequest, ChatResponse, ModelInfo, ProviderCapabilities, StreamEvent};
use crate::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

/// Core trait for LLM providers.
///
/// Implementations of this trait provide a uniform interface to different LLM services
/// (OpenAI, Anthropic, Ollama, etc.).
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider's display name
    fn name(&self) -> &str;

    /// Get available models for this provider
    fn models(&self) -> Vec<ModelInfo>;

    /// Send a chat request and get a response
    ///
    /// # Arguments
    ///
    /// * `request` - The chat request
    ///
    /// # Returns
    ///
    /// A complete chat response
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;

    /// Send a chat request and get a streaming response
    ///
    /// # Arguments
    ///
    /// * `request` - The chat request (should have `stream: true`)
    ///
    /// # Returns
    ///
    /// A stream of response events
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>>;

    /// Get provider capabilities
    fn capabilities(&self) -> ProviderCapabilities;
}

//! Application state management
//!
//! Provides thread-safe access to application state using Arc<RwLock>.
//! Manages sessions, conversations, configuration, and providers.

use crate::config::HiveConfig;
use crate::conversation::ConversationEngine;
use crate::error::{HiveCodeError, Result};
use crate::types::{AppSessionState, ProviderInfo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Thread-safe application state
pub struct AppState {
    /// Current session
    session: Arc<RwLock<AppSessionState>>,
    /// Current conversation
    conversation: Arc<RwLock<ConversationEngine>>,
    /// Application configuration
    config: Arc<RwLock<HiveConfig>>,
    /// Available providers
    providers: Arc<RwLock<Vec<ProviderInfo>>>,
}

impl AppState {
    /// Create a new application state
    pub async fn new(session: AppSessionState, conversation: ConversationEngine, config: HiveConfig) -> Self {
        Self {
            session: Arc::new(RwLock::new(session)),
            conversation: Arc::new(RwLock::new(conversation)),
            config: Arc::new(RwLock::new(config)),
            providers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new application state with defaults
    pub async fn default() -> Self {
        let session = AppSessionState::new("/");
        let conversation = ConversationEngine::new("claude");
        let config = HiveConfig::default();

        Self::new(session, conversation, config).await
    }

    /// Get the current session (read-only)
    pub async fn get_session(&self) -> Result<AppSessionState> {
        Ok(self.session.read().await.clone())
    }

    /// Update the session (mutable)
    pub async fn update_session<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppSessionState),
    {
        let mut session = self.session.write().await;
        f(&mut session);
        Ok(())
    }

    /// Get the conversation state
    pub async fn get_conversation_state(&self) -> Result<crate::types::ConversationState> {
        let conversation = self.conversation.read().await;
        Ok(conversation.state().clone())
    }

    /// Get all messages in the current conversation
    pub async fn get_messages(&self) -> Result<Vec<crate::types::Message>> {
        let conversation = self.conversation.read().await;
        Ok(conversation.state().messages())
    }

    /// Add a message to the conversation
    pub async fn add_message(&self, message: crate::types::Message) -> Result<()> {
        let mut conversation = self.conversation.write().await;
        conversation.add_message(message)?;
        Ok(())
    }

    /// Remove a message from the conversation
    pub async fn remove_message(&self, message_id: &str) -> Result<Option<crate::types::Message>> {
        let mut conversation = self.conversation.write().await;
        Ok(conversation.remove_message(message_id)?)
    }

    /// Clear the conversation
    pub async fn clear_conversation(&self) -> Result<()> {
        let mut conversation = self.conversation.write().await;
        conversation.clear();
        Ok(())
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> Result<HiveConfig> {
        Ok(self.config.read().await.clone())
    }

    /// Update the configuration
    pub async fn update_config<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut HiveConfig),
    {
        let mut config = self.config.write().await;
        f(&mut config);
        Ok(())
    }

    /// Reload configuration from disk
    pub async fn reload_config(&self) -> Result<()> {
        let new_config = HiveConfig::load()?;
        let mut config = self.config.write().await;
        *config = new_config;
        debug!("Configuration reloaded");
        Ok(())
    }

    /// Get all providers
    pub async fn get_providers(&self) -> Result<Vec<ProviderInfo>> {
        Ok(self.providers.read().await.clone())
    }

    /// Get a specific provider by ID
    pub async fn get_provider(&self, id: &str) -> Result<Option<ProviderInfo>> {
        let providers = self.providers.read().await;
        Ok(providers.iter().find(|p| p.id == id).cloned())
    }

    /// Register a provider
    pub async fn register_provider(&self, provider: ProviderInfo) -> Result<()> {
        let mut providers = self.providers.write().await;

        // Remove existing provider with same ID
        providers.retain(|p| p.id != provider.id);

        providers.push(provider);
        debug!("Provider registered");
        Ok(())
    }

    /// Update provider availability
    pub async fn set_provider_availability(&self, id: &str, available: bool) -> Result<()> {
        let mut providers = self.providers.write().await;

        if let Some(provider) = providers.iter_mut().find(|p| p.id == id) {
            provider.available = available;
            debug!("Provider {} availability set to {}", id, available);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Provider not found: {}", id)))
        }
    }

    /// Get conversation summary
    pub async fn get_conversation_summary(&self) -> Result<crate::conversation::ConversationSummary> {
        let conversation = self.conversation.read().await;
        Ok(conversation.get_summary())
    }

    /// Get available tokens in the current conversation
    pub async fn get_available_tokens(&self) -> Result<u64> {
        let conversation = self.conversation.read().await;
        Ok(conversation.available_tokens())
    }

    /// Update token counts
    pub async fn update_tokens(&self, input: u64, output: u64) -> Result<()> {
        let mut conversation = self.conversation.write().await;
        conversation.update_tokens(input, output);
        Ok(())
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            session: self.session.clone(),
            conversation: self.conversation.clone(),
            config: self.config.clone(),
            providers: self.providers.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Message;
    use crate::types::MessageRole;

    #[tokio::test]
    async fn test_new_app_state() {
        let state = AppState::default().await;
        let session = state.get_session().await.unwrap();
        assert!(!session.session_id.is_empty());
    }

    #[tokio::test]
    async fn test_session_operations() {
        let state = AppState::default().await;

        // Update session
        state.update_session(|s| {
            s.working_directory = "/home/user".to_string();
        }).await.unwrap();

        // Verify update
        let session = state.get_session().await.unwrap();
        assert_eq!(session.working_directory, "/home/user");
    }

    #[tokio::test]
    async fn test_add_message() {
        let state = AppState::default().await;
        let message = Message::text(MessageRole::User, "Hello");

        state.add_message(message).await.unwrap();

        let messages = state.get_messages().await.unwrap();
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_remove_message() {
        let state = AppState::default().await;
        let message = Message::text(MessageRole::User, "Hello");
        let msg_id = message.id.clone();

        state.add_message(message).await.unwrap();
        let messages = state.get_messages().await.unwrap();
        assert_eq!(messages.len(), 1);

        state.remove_message(&msg_id).await.unwrap();
        let messages = state.get_messages().await.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_clear_conversation() {
        let state = AppState::default().await;

        state.add_message(Message::text(MessageRole::User, "Test")).await.unwrap();
        assert_eq!(state.get_messages().await.unwrap().len(), 1);

        state.clear_conversation().await.unwrap();
        assert_eq!(state.get_messages().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_config_operations() {
        let state = AppState::default().await;

        let config = state.get_config().await.unwrap();
        assert_eq!(config.app.name, "HiveCode");

        state.update_config(|c| {
            c.app.name = "CustomName".to_string();
        }).await.unwrap();

        let config = state.get_config().await.unwrap();
        assert_eq!(config.app.name, "CustomName");
    }

    #[tokio::test]
    async fn test_provider_registration() {
        let state = AppState::default().await;

        let provider = ProviderInfo {
            id: "test".to_string(),
            name: "Test Provider".to_string(),
            available: true,
            models: vec!["model-1".to_string()],
            config: None,
        };

        state.register_provider(provider).await.unwrap();

        let providers = state.get_providers().await.unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "test");
    }

    #[tokio::test]
    async fn test_provider_availability() {
        let state = AppState::default().await;

        let provider = ProviderInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            available: true,
            models: vec![],
            config: None,
        };

        state.register_provider(provider).await.unwrap();
        state.set_provider_availability("test", false).await.unwrap();

        let provider = state.get_provider("test").await.unwrap();
        assert!(!provider.unwrap().available);
    }

    #[tokio::test]
    async fn test_conversation_summary() {
        let state = AppState::default().await;

        state.add_message(Message::text(MessageRole::User, "Hello")).await.unwrap();
        state.add_message(Message::text(MessageRole::Assistant, "Hi")).await.unwrap();

        let summary = state.get_conversation_summary().await.unwrap();
        assert_eq!(summary.total_messages, 2);
        assert_eq!(summary.user_messages, 1);
        assert_eq!(summary.assistant_messages, 1);
    }

    #[tokio::test]
    async fn test_token_updates() {
        let state = AppState::default().await;

        state.update_tokens(100, 50).await.unwrap();
        let summary = state.get_conversation_summary().await.unwrap();
        assert_eq!(summary.total_tokens, 150);
    }

    #[tokio::test]
    async fn test_state_clone() {
        let state = AppState::default().await;
        let cloned = state.clone();

        state.add_message(Message::text(MessageRole::User, "Hello")).await.unwrap();

        let messages = cloned.get_messages().await.unwrap();
        assert_eq!(messages.len(), 1);
    }
}

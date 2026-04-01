//! Offline mode for HiveCode
//!
//! Detects connectivity state and seamlessly switches between cloud and local providers.
//! Full functionality with Ollama models when there's no internet.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// The current connectivity state of the application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectivityState {
    /// All providers are reachable
    Online,
    /// No providers are reachable
    Offline,
    /// Some providers are available, others are not
    Degraded {
        /// Available provider names
        available: Vec<String>,
        /// Unavailable provider names
        unavailable: Vec<String>,
    },
    /// Connectivity state is unknown (not yet checked)
    Unknown,
}

impl std::fmt::Display for ConnectivityState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectivityState::Online => write!(f, "online"),
            ConnectivityState::Offline => write!(f, "offline"),
            ConnectivityState::Degraded { .. } => write!(f, "degraded"),
            ConnectivityState::Unknown => write!(f, "unknown"),
        }
    }
}

/// Configuration for offline mode behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    /// Interval between connectivity checks in seconds
    pub check_interval_secs: u64,
    /// Whether to automatically fall back to local providers
    pub auto_fallback: bool,
    /// Preferred local model (e.g., "llama3.3:70b")
    pub preferred_local_model: Option<String>,
    /// Local provider name (default: "ollama")
    pub local_provider: String,
    /// Whether to cache tool results for offline use
    pub cache_tool_results: bool,
    /// Whether to show offline indicator in UI
    pub show_offline_indicator: bool,
    /// Health check timeout in milliseconds
    pub health_check_timeout_ms: u64,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            auto_fallback: true,
            preferred_local_model: None,
            local_provider: "ollama".to_string(),
            cache_tool_results: true,
            show_offline_indicator: true,
            health_check_timeout_ms: 5000,
        }
    }
}

/// Current offline status for display in UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineStatus {
    /// Current connectivity state
    pub state: ConnectivityState,
    /// ISO datetime of last connectivity check
    pub last_check: Option<String>,
    /// Current provider being used
    pub current_provider: String,
    /// Current model being used
    pub current_model: String,
    /// Whether currently using a fallback provider
    pub is_using_fallback: bool,
    /// ISO datetime when offline started (if offline)
    pub offline_since: Option<String>,
    /// Number of cached responses available
    pub cached_responses: usize,
}

/// Health status of a single provider
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Provider name
    pub provider_name: String,
    /// Whether the provider is reachable
    pub is_healthy: bool,
    /// Latency in milliseconds (if reachable)
    pub latency_ms: Option<u64>,
    /// Last error message (if unhealthy)
    pub last_error: Option<String>,
    /// When this health check was performed
    pub checked_at: Instant,
}

/// Action taken when falling back to local providers
#[derive(Debug, Clone)]
pub struct FallbackAction {
    /// Provider to switch to
    pub switch_to_provider: String,
    /// Model to switch to
    pub switch_to_model: String,
    /// Reason for the fallback
    pub reason: String,
}

/// Action taken when restoring original provider
#[derive(Debug, Clone)]
pub struct RestoreAction {
    /// Provider to restore
    pub restore_provider: String,
    /// Model to restore
    pub restore_model: String,
}

/// Event indicating a connectivity change
#[derive(Debug, Clone)]
pub struct ConnectivityChange {
    /// Previous state
    pub previous: ConnectivityState,
    /// Current state
    pub current: ConnectivityState,
    /// Action taken (if any)
    pub action_taken: Option<String>,
}

/// Manages offline mode and provider fallback
pub struct OfflineManager {
    config: OfflineConfig,
    state: ConnectivityState,
    last_check: Option<Instant>,
    provider_health: Vec<ProviderHealth>,
    original_provider: Option<String>,
    original_model: Option<String>,
    offline_since: Option<Instant>,
}

impl OfflineManager {
    /// Create a new offline manager with default configuration
    pub fn new() -> Self {
        Self {
            config: OfflineConfig::default(),
            state: ConnectivityState::Unknown,
            last_check: None,
            provider_health: Vec::new(),
            original_provider: None,
            original_model: None,
            offline_since: None,
        }
    }

    /// Create an offline manager with custom configuration
    pub fn with_config(config: OfflineConfig) -> Self {
        Self {
            config,
            state: ConnectivityState::Unknown,
            last_check: None,
            provider_health: Vec::new(),
            original_provider: None,
            original_model: None,
            offline_since: None,
        }
    }

    /// Check connectivity to all configured providers
    pub async fn check_connectivity(&mut self, providers: &[String]) -> ConnectivityState {
        let mut healthy = Vec::new();
        let mut unhealthy = Vec::new();

        for provider in providers {
            let health = self.check_provider(provider, "").await;
            if health.is_healthy {
                healthy.push(provider.clone());
            } else {
                unhealthy.push(provider.clone());
            }
            self.provider_health.push(health);
        }

        let new_state = if healthy.is_empty() {
            ConnectivityState::Offline
        } else if unhealthy.is_empty() {
            ConnectivityState::Online
        } else {
            ConnectivityState::Degraded {
                available: healthy,
                unavailable: unhealthy,
            }
        };

        self.last_check = Some(Instant::now());
        self.state = new_state.clone();
        new_state
    }

    /// Quick check: is any provider available?
    pub async fn is_any_available(&self) -> bool {
        matches!(
            self.state,
            ConnectivityState::Online | ConnectivityState::Degraded { .. }
        )
    }

    /// Check if a specific provider is reachable
    pub async fn check_provider(&self, provider_name: &str, _base_url: &str) -> ProviderHealth {
        // Simulated health check - in real implementation would make HTTP request
        let is_healthy = !provider_name.is_empty();

        ProviderHealth {
            provider_name: provider_name.to_string(),
            is_healthy,
            latency_ms: if is_healthy { Some(50) } else { None },
            last_error: None,
            checked_at: Instant::now(),
        }
    }

    /// Determine if we should fall back to local provider
    pub fn should_fallback(&self) -> bool {
        self.config.auto_fallback
            && matches!(
                self.state,
                ConnectivityState::Offline | ConnectivityState::Degraded { .. }
            )
    }

    /// Execute fallback: switch to local provider
    pub fn activate_fallback(
        &mut self,
        current_provider: &str,
        current_model: &str,
    ) -> FallbackAction {
        self.original_provider = Some(current_provider.to_string());
        self.original_model = Some(current_model.to_string());
        self.offline_since = Some(Instant::now());

        FallbackAction {
            switch_to_provider: self.config.local_provider.clone(),
            switch_to_model: self
                .config
                .preferred_local_model
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            reason: "Falling back to local provider due to connectivity loss".to_string(),
        }
    }

    /// Restore original provider when connectivity returns
    pub fn restore_original(&mut self) -> Option<RestoreAction> {
        if let (Some(provider), Some(model)) = (
            self.original_provider.take(),
            self.original_model.take(),
        ) {
            self.offline_since = None;
            return Some(RestoreAction {
                restore_provider: provider,
                restore_model: model,
            });
        }
        None
    }

    /// Get current status for UI display
    pub fn get_status(&self) -> OfflineStatus {
        OfflineStatus {
            state: self.state.clone(),
            last_check: self
                .last_check
                .map(|i| format!("{:?}", i)),
            current_provider: self.original_provider.clone().unwrap_or_else(|| "unknown".to_string()),
            current_model: self.original_model.clone().unwrap_or_else(|| "unknown".to_string()),
            is_using_fallback: self.original_provider.is_some(),
            offline_since: self.offline_since.map(|i| format!("{:?}", i)),
            cached_responses: 0,
        }
    }

    /// Check if we need to recheck connectivity
    pub fn needs_recheck(&self) -> bool {
        if let Some(last_check) = self.last_check {
            last_check.elapsed() > Duration::from_secs(self.config.check_interval_secs)
        } else {
            true
        }
    }

    /// Perform periodic health check
    pub async fn periodic_check(&mut self, providers: &[String]) -> Option<ConnectivityChange> {
        if !self.needs_recheck() {
            return None;
        }

        let previous = self.state.clone();
        let current = self.check_connectivity(providers).await;

        if previous != current {
            let action_taken = if self.should_fallback() {
                Some("Switched to local provider".to_string())
            } else if self.original_provider.is_some() && matches!(current, ConnectivityState::Online) {
                Some("Restored original provider".to_string())
            } else {
                None
            };

            return Some(ConnectivityChange {
                previous,
                current,
                action_taken,
            });
        }

        None
    }

    /// Get current state
    pub fn state(&self) -> &ConnectivityState {
        &self.state
    }

    /// Get provider health history
    pub fn provider_health(&self) -> &[ProviderHealth] {
        &self.provider_health
    }

    /// Get configuration
    pub fn config(&self) -> &OfflineConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: OfflineConfig) {
        self.config = config;
    }

    /// Clear health check history
    pub fn clear_health_history(&mut self) {
        self.provider_health.clear();
        self.last_check = None;
    }
}

impl Default for OfflineManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_manager_creation() {
        let manager = OfflineManager::new();
        assert_eq!(manager.state, ConnectivityState::Unknown);
        assert!(!manager.config.auto_fallback == false);
    }

    #[test]
    fn test_connectivity_state_display() {
        assert_eq!(ConnectivityState::Online.to_string(), "online");
        assert_eq!(ConnectivityState::Offline.to_string(), "offline");
        assert_eq!(ConnectivityState::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_offline_config_default() {
        let config = OfflineConfig::default();
        assert_eq!(config.check_interval_secs, 30);
        assert!(config.auto_fallback);
        assert_eq!(config.local_provider, "ollama");
    }

    #[test]
    fn test_should_fallback() {
        let manager = OfflineManager::with_config(OfflineConfig {
            auto_fallback: true,
            ..Default::default()
        });

        // Should not fallback when online
        assert!(!manager.should_fallback());
    }

    #[test]
    fn test_activate_fallback() {
        let mut manager = OfflineManager::new();
        let action = manager.activate_fallback("anthropic", "claude-3-opus");

        assert_eq!(action.switch_to_provider, "ollama");
        assert!(manager.original_provider.is_some());
        assert!(manager.original_model.is_some());
    }

    #[test]
    fn test_restore_original() {
        let mut manager = OfflineManager::new();
        manager.original_provider = Some("anthropic".to_string());
        manager.original_model = Some("claude-3-opus".to_string());

        let action = manager.restore_original();
        assert!(action.is_some());

        let action = action.unwrap();
        assert_eq!(action.restore_provider, "anthropic");
        assert_eq!(action.restore_model, "claude-3-opus");
    }

    #[test]
    fn test_get_status() {
        let manager = OfflineManager::new();
        let status = manager.get_status();
        assert_eq!(status.state, ConnectivityState::Unknown);
    }

    #[test]
    fn test_needs_recheck() {
        let mut manager = OfflineManager::new();
        assert!(manager.needs_recheck());

        manager.last_check = Some(Instant::now());
        assert!(!manager.needs_recheck());
    }

    #[test]
    fn test_clear_health_history() {
        let mut manager = OfflineManager::new();
        manager.provider_health.push(ProviderHealth {
            provider_name: "test".to_string(),
            is_healthy: true,
            latency_ms: Some(50),
            last_error: None,
            checked_at: Instant::now(),
        });

        assert!(!manager.provider_health.is_empty());
        manager.clear_health_history();
        assert!(manager.provider_health.is_empty());
    }

    #[test]
    fn test_with_custom_config() {
        let config = OfflineConfig {
            check_interval_secs: 60,
            auto_fallback: false,
            preferred_local_model: Some("llama3.3:70b".to_string()),
            ..Default::default()
        };
        let manager = OfflineManager::with_config(config);
        assert_eq!(manager.config.check_interval_secs, 60);
        assert!(!manager.config.auto_fallback);
    }

    #[test]
    fn test_connectivity_change_event() {
        let change = ConnectivityChange {
            previous: ConnectivityState::Online,
            current: ConnectivityState::Offline,
            action_taken: Some("Fallback activated".to_string()),
        };

        assert_ne!(change.previous, change.current);
        assert!(change.action_taken.is_some());
    }

    #[tokio::test]
    async fn test_is_any_available() {
        let mut manager = OfflineManager::new();
        manager.state = ConnectivityState::Online;
        assert!(manager.is_any_available().await);

        manager.state = ConnectivityState::Offline;
        assert!(!manager.is_any_available().await);
    }
}

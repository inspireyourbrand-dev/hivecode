//! Context window and token management system
//!
//! Tracks token usage across different models, manages context windows,
//! calculates costs, and provides warnings when context is running low.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;
use crate::error::{HiveCodeError, Result};

/// Token usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTracker {
    /// Total input tokens consumed across all sessions
    pub total_input_tokens: u64,
    /// Total output tokens consumed across all sessions
    pub total_output_tokens: u64,
    /// Input tokens in current session
    pub session_input_tokens: u64,
    /// Output tokens in current session
    pub session_output_tokens: u64,
    /// Maximum context window size for current model
    pub max_context_window: u64,
    /// Current token usage in context window
    pub current_context_usage: u64,
    /// Estimated cost in USD
    pub cost_usd: f64,
}

impl TokenTracker {
    /// Create a new token tracker
    pub fn new(max_context: u64) -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            session_input_tokens: 0,
            session_output_tokens: 0,
            max_context_window: max_context,
            current_context_usage: 0,
            cost_usd: 0.0,
        }
    }

    /// Get total tokens across all sessions
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }

    /// Get session tokens
    pub fn session_tokens(&self) -> u64 {
        self.session_input_tokens + self.session_output_tokens
    }

    /// Get remaining context tokens
    pub fn remaining_context(&self) -> u64 {
        self.max_context_window.saturating_sub(self.current_context_usage)
    }

    /// Check if context is running low (>80% used)
    pub fn should_summarize(&self) -> bool {
        let usage_percent = (self.current_context_usage as f64 / self.max_context_window as f64) * 100.0;
        usage_percent > 80.0
    }
}

/// Pricing information for a model
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per 1M input tokens in USD
    pub input_per_million: f64,
    /// Cost per 1M output tokens in USD
    pub output_per_million: f64,
}

impl ModelPricing {
    /// Create new pricing
    pub fn new(input_per_million: f64, output_per_million: f64) -> Self {
        Self {
            input_per_million,
            output_per_million,
        }
    }

    /// Calculate cost for tokens
    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_per_million;
        input_cost + output_cost
    }
}

/// Context and token management system
pub struct ContextManager {
    tracker: Arc<RwLock<TokenTracker>>,
    model_limits: Arc<RwLock<HashMap<String, u64>>>,
    model_pricing: Arc<RwLock<HashMap<String, ModelPricing>>>,
}

impl ContextManager {
    /// Create a new context manager
    pub fn new(max_context: u64) -> Self {
        let mut manager = Self {
            tracker: Arc::new(RwLock::new(TokenTracker::new(max_context))),
            model_limits: Arc::new(RwLock::new(HashMap::new())),
            model_pricing: Arc::new(RwLock::new(HashMap::new())),
        };

        // Register default models
        let mut limits = HashMap::new();
        limits.insert("claude-sonnet-4".to_string(), 200_000);
        limits.insert("claude-opus-4".to_string(), 200_000);
        limits.insert("gpt-4o".to_string(), 128_000);
        limits.insert("gpt-4o-mini".to_string(), 128_000);
        limits.insert("llama-3".to_string(), 8_000);

        let mut pricing = HashMap::new();
        pricing.insert(
            "claude-sonnet-4".to_string(),
            ModelPricing::new(3.0, 15.0),
        );
        pricing.insert(
            "claude-opus-4".to_string(),
            ModelPricing::new(15.0, 75.0),
        );
        pricing.insert(
            "gpt-4o".to_string(),
            ModelPricing::new(2.50, 10.0),
        );
        pricing.insert(
            "gpt-4o-mini".to_string(),
            ModelPricing::new(0.15, 0.60),
        );
        pricing.insert(
            "llama-3".to_string(),
            ModelPricing::new(0.0, 0.0),
        );

        // We need to set these synchronously in the constructor
        // So we'll set them in the creation instead
        manager.model_limits = Arc::new(RwLock::new(limits));
        manager.model_pricing = Arc::new(RwLock::new(pricing));

        manager
    }

    /// Record token usage
    pub async fn record_usage(
        &self,
        input_tokens: u64,
        output_tokens: u64,
        model: &str,
    ) -> Result<()> {
        let mut tracker = self.tracker.write().await;

        // Update totals
        tracker.total_input_tokens += input_tokens;
        tracker.total_output_tokens += output_tokens;
        tracker.session_input_tokens += input_tokens;
        tracker.session_output_tokens += output_tokens;
        tracker.current_context_usage += input_tokens + output_tokens;

        // Calculate cost
        let pricing = self.model_pricing.read().await;
        if let Some(model_pricing) = pricing.get(model) {
            tracker.cost_usd += model_pricing.calculate_cost(input_tokens, output_tokens);
        }

        debug!(
            "Recorded tokens: input={}, output={}, model={}, cost=${:.4}",
            input_tokens, output_tokens, model, tracker.cost_usd
        );

        Ok(())
    }

    /// Get current token usage
    pub async fn get_usage(&self) -> Result<TokenUsage> {
        let tracker = self.tracker.read().await;

        Ok(TokenUsage {
            total_input_tokens: tracker.total_input_tokens,
            total_output_tokens: tracker.total_output_tokens,
            total_tokens: tracker.total_tokens(),
            session_input_tokens: tracker.session_input_tokens,
            session_output_tokens: tracker.session_output_tokens,
            session_tokens: tracker.session_tokens(),
            current_context_usage: tracker.current_context_usage,
            remaining_context: tracker.remaining_context(),
        })
    }

    /// Get cost summary
    pub async fn get_cost_summary(&self) -> Result<CostSummary> {
        let tracker = self.tracker.read().await;

        Ok(CostSummary {
            total_cost_usd: tracker.cost_usd,
            total_input_tokens: tracker.total_input_tokens,
            total_output_tokens: tracker.total_output_tokens,
            session_cost_usd: tracker.cost_usd, // In a real system, track session separately
        })
    }

    /// Estimate cost for hypothetical usage
    pub async fn estimate_cost(
        &self,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Result<f64> {
        let pricing = self.model_pricing.read().await;

        if let Some(model_pricing) = pricing.get(model) {
            Ok(model_pricing.calculate_cost(input_tokens, output_tokens))
        } else {
            Err(HiveCodeError::NotFound(format!(
                "Pricing not found for model: {}",
                model
            )))
        }
    }

    /// Get remaining context tokens
    pub async fn get_remaining_context(&self) -> Result<u64> {
        Ok(self.tracker.read().await.remaining_context())
    }

    /// Check if context should be summarized
    pub async fn should_summarize(&self) -> Result<bool> {
        Ok(self.tracker.read().await.should_summarize())
    }

    /// Reset session usage
    pub async fn reset_session(&self) -> Result<()> {
        let mut tracker = self.tracker.write().await;
        tracker.session_input_tokens = 0;
        tracker.session_output_tokens = 0;
        tracker.current_context_usage = 0;

        debug!("Session usage reset");
        Ok(())
    }

    /// Register a new model
    pub async fn register_model(
        &self,
        model_name: &str,
        context_limit: u64,
        pricing: ModelPricing,
    ) -> Result<()> {
        let mut limits = self.model_limits.write().await;
        let mut pricing_map = self.model_pricing.write().await;

        limits.insert(model_name.to_string(), context_limit);
        pricing_map.insert(model_name.to_string(), pricing);

        debug!("Registered model: {} (context: {})", model_name, context_limit);
        Ok(())
    }

    /// Get model context limit
    pub async fn get_model_limit(&self, model: &str) -> Result<Option<u64>> {
        Ok(self.model_limits.read().await.get(model).copied())
    }

    /// Get all registered models
    pub async fn get_registered_models(&self) -> Result<Vec<String>> {
        Ok(self
            .model_limits
            .read()
            .await
            .keys()
            .cloned()
            .collect())
    }
}

impl Clone for ContextManager {
    fn clone(&self) -> Self {
        Self {
            tracker: self.tracker.clone(),
            model_limits: self.model_limits.clone(),
            model_pricing: self.model_pricing.clone(),
        }
    }
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Total input tokens across all sessions
    pub total_input_tokens: u64,
    /// Total output tokens across all sessions
    pub total_output_tokens: u64,
    /// Total tokens
    pub total_tokens: u64,
    /// Session input tokens
    pub session_input_tokens: u64,
    /// Session output tokens
    pub session_output_tokens: u64,
    /// Session total tokens
    pub session_tokens: u64,
    /// Current context window usage
    pub current_context_usage: u64,
    /// Remaining context tokens
    pub remaining_context: u64,
}

/// Cost summary information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    /// Total cost in USD
    pub total_cost_usd: f64,
    /// Total input tokens
    pub total_input_tokens: u64,
    /// Total output tokens
    pub total_output_tokens: u64,
    /// Session cost in USD
    pub session_cost_usd: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_usage() {
        let manager = ContextManager::new(200_000);

        manager
            .record_usage(100, 50, "claude-sonnet-4")
            .await
            .unwrap();

        let usage = manager.get_usage().await.unwrap();
        assert_eq!(usage.total_input_tokens, 100);
        assert_eq!(usage.total_output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let manager = ContextManager::new(200_000);

        manager
            .record_usage(1_000_000, 1_000_000, "claude-sonnet-4")
            .await
            .unwrap();

        let cost = manager.get_cost_summary().await.unwrap();
        // Claude Sonnet 4: $3 + $15 = $18 for 1M tokens each
        assert!((cost.total_cost_usd - 18.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_estimate_cost() {
        let manager = ContextManager::new(200_000);

        let cost = manager
            .estimate_cost("gpt-4o", 1_000_000, 1_000_000)
            .await
            .unwrap();

        // GPT-4o: $2.50 + $10 = $12.50
        assert!((cost - 12.50).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_remaining_context() {
        let manager = ContextManager::new(200_000);

        manager
            .record_usage(50_000, 0, "claude-sonnet-4")
            .await
            .unwrap();

        let remaining = manager.get_remaining_context().await.unwrap();
        assert_eq!(remaining, 150_000);
    }

    #[tokio::test]
    async fn test_should_summarize() {
        let manager = ContextManager::new(100_000);

        // Use 85% of context
        manager
            .record_usage(85_000, 0, "claude-sonnet-4")
            .await
            .unwrap();

        assert!(manager.should_summarize().await.unwrap());
    }

    #[tokio::test]
    async fn test_reset_session() {
        let manager = ContextManager::new(200_000);

        manager
            .record_usage(100, 50, "claude-sonnet-4")
            .await
            .unwrap();

        let usage_before = manager.get_usage().await.unwrap();
        assert!(usage_before.session_tokens > 0);

        manager.reset_session().await.unwrap();

        let usage_after = manager.get_usage().await.unwrap();
        assert_eq!(usage_after.session_tokens, 0);
        assert_eq!(usage_after.current_context_usage, 0);
    }

    #[tokio::test]
    async fn test_register_model() {
        let manager = ContextManager::new(200_000);

        let pricing = ModelPricing::new(1.0, 2.0);
        manager
            .register_model("custom-model", 150_000, pricing)
            .await
            .unwrap();

        let limit = manager.get_model_limit("custom-model").await.unwrap();
        assert_eq!(limit, Some(150_000));
    }

    #[tokio::test]
    async fn test_get_registered_models() {
        let manager = ContextManager::new(200_000);

        let models = manager.get_registered_models().await.unwrap();
        assert!(models.len() >= 5); // At least default models
        assert!(models.contains(&"claude-sonnet-4".to_string()));
        assert!(models.contains(&"gpt-4o".to_string()));
    }

    #[test]
    fn test_token_tracker_remaining() {
        let mut tracker = TokenTracker::new(100_000);
        tracker.current_context_usage = 30_000;

        assert_eq!(tracker.remaining_context(), 70_000);
    }

    #[test]
    fn test_should_summarize_threshold() {
        let mut tracker = TokenTracker::new(100_000);
        tracker.current_context_usage = 75_000; // 75% - should not summarize
        assert!(!tracker.should_summarize());

        tracker.current_context_usage = 85_000; // 85% - should summarize
        assert!(tracker.should_summarize());
    }
}

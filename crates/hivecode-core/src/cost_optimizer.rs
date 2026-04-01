//! Cost optimizer for HiveCode
//!
//! Analyzes spending patterns and recommends ways to reduce costs.
//! Shows what you could have saved with model routing, prompt caching, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete cost analysis with recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    /// Actual cost of the session/conversation
    pub actual_cost: f64,
    /// Estimated cost with optimizations applied
    pub optimized_cost: f64,
    /// Potential savings in dollars
    pub potential_savings: f64,
    /// Percentage of cost that could be saved
    pub savings_percentage: f64,
    /// Recommended optimizations
    pub recommendations: Vec<CostRecommendation>,
    /// Detailed breakdown by model and token type
    pub breakdown: CostBreakdown,
}

/// A single cost optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecommendation {
    /// Human-readable title
    pub title: String,
    /// Detailed description of the recommendation
    pub description: String,
    /// Estimated savings in dollars
    pub estimated_savings: f64,
    /// Percentage savings
    pub savings_percentage: f64,
    /// How difficult this optimization is
    pub difficulty: Difficulty,
    /// Type of optimization
    pub recommendation_type: RecommendationType,
}

/// Type of cost optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationType {
    /// Use cheaper models for simple tasks
    ModelRouting,
    /// Enable prompt caching for repeated requests
    PromptCaching,
    /// Compress tool outputs to reduce tokens
    OutputCompression,
    /// Compact conversation history earlier
    ConversationCompact,
    /// Switch to local models for some tasks
    LocalModel,
    /// Batch tool calls to reduce overhead
    BatchToolCalls,
    /// Use smaller context window
    SmallerContext,
    /// Reduce extended thinking budget
    ReduceThinking,
}

impl std::fmt::Display for RecommendationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendationType::ModelRouting => write!(f, "model_routing"),
            RecommendationType::PromptCaching => write!(f, "prompt_caching"),
            RecommendationType::OutputCompression => write!(f, "output_compression"),
            RecommendationType::ConversationCompact => write!(f, "conversation_compact"),
            RecommendationType::LocalModel => write!(f, "local_model"),
            RecommendationType::BatchToolCalls => write!(f, "batch_tool_calls"),
            RecommendationType::SmallerContext => write!(f, "smaller_context"),
            RecommendationType::ReduceThinking => write!(f, "reduce_thinking"),
        }
    }
}

/// Difficulty of implementing an optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Difficulty {
    /// Just toggle a setting
    Easy,
    /// Requires some configuration
    Medium,
    /// Requires setup (e.g., Ollama install)
    Advanced,
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "easy"),
            Difficulty::Medium => write!(f, "medium"),
            Difficulty::Advanced => write!(f, "advanced"),
        }
    }
}

/// Detailed cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Costs by model
    pub by_model: HashMap<String, ModelCost>,
    /// Costs attributed to tool-heavy requests
    pub by_tool: HashMap<String, f64>,
    /// Cost of input tokens
    pub input_tokens_cost: f64,
    /// Cost of output tokens
    pub output_tokens_cost: f64,
    /// Cost of cache reads
    pub cache_read_cost: f64,
    /// Cost of cache writes
    pub cache_write_cost: f64,
    /// Cost of thinking tokens
    pub thinking_cost: f64,
}

impl Default for CostBreakdown {
    fn default() -> Self {
        Self {
            by_model: HashMap::new(),
            by_tool: HashMap::new(),
            input_tokens_cost: 0.0,
            output_tokens_cost: 0.0,
            cache_read_cost: 0.0,
            cache_write_cost: 0.0,
            thinking_cost: 0.0,
        }
    }
}

/// Cost for a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
    /// Model name
    pub model_name: String,
    /// Total input tokens
    pub input_tokens: u64,
    /// Total output tokens
    pub output_tokens: u64,
    /// Total cost for this model
    pub total_cost: f64,
    /// Number of requests
    pub request_count: usize,
    /// Average cost per request
    pub avg_cost_per_request: f64,
}

/// A single usage record for analysis
#[derive(Debug, Clone)]
pub struct UsageRecord {
    /// Model used
    pub model: String,
    /// Input tokens consumed
    pub input_tokens: u64,
    /// Output tokens generated
    pub output_tokens: u64,
    /// Cost of this request
    pub cost: f64,
    /// Tools called (if any)
    pub tool_calls: Vec<String>,
    /// Whether this could have used a cheaper model
    pub was_simple_task: bool,
    /// Whether this had cacheable content
    pub could_have_cached: bool,
    /// Thinking tokens used
    pub thinking_tokens: u64,
}

/// Analyzes costs and recommends optimizations
pub struct CostOptimizer {
    records: Vec<UsageRecord>,
    model_pricing: HashMap<String, (f64, f64)>, // model -> (input_per_million, output_per_million)
}

impl CostOptimizer {
    /// Create a new cost optimizer
    pub fn new() -> Self {
        let mut optimizer = Self {
            records: Vec::new(),
            model_pricing: HashMap::new(),
        };

        // Register default pricing (per million tokens)
        optimizer.register_pricing("claude-3-opus", 15.0, 75.0);
        optimizer.register_pricing("claude-3-sonnet", 3.0, 15.0);
        optimizer.register_pricing("claude-3-haiku", 0.25, 1.25);
        optimizer.register_pricing("gpt-4o", 2.50, 10.0);
        optimizer.register_pricing("gpt-4o-mini", 0.15, 0.60);
        optimizer.register_pricing("local-ollama", 0.0, 0.0);

        optimizer
    }

    /// Register pricing for a model
    pub fn register_pricing(
        &mut self,
        model: &str,
        input_per_million: f64,
        output_per_million: f64,
    ) {
        self.model_pricing
            .insert(model.to_string(), (input_per_million, output_per_million));
    }

    /// Record a usage event
    pub fn record(&mut self, record: UsageRecord) {
        self.records.push(record);
    }

    /// Calculate cost for a request
    pub fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        if let Some((input_price, output_price)) = self.model_pricing.get(model) {
            let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
            let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;
            input_cost + output_cost
        } else {
            0.0
        }
    }

    /// Analyze costs and generate recommendations
    pub fn analyze(&self) -> CostAnalysis {
        let actual_cost: f64 = self.records.iter().map(|r| r.cost).sum();
        let breakdown = self.get_breakdown();

        let routing_savings = self.estimate_routing_savings();
        let caching_savings = self.estimate_caching_savings();
        let compression_savings = self.estimate_compression_savings();
        let local_savings = self.estimate_local_savings();

        let mut recommendations = Vec::new();

        if routing_savings > 0.01 {
            recommendations.push(CostRecommendation {
                title: "Enable Model Routing".to_string(),
                description: "Use smaller, cheaper models for simple tasks like summarization and analysis."
                    .to_string(),
                estimated_savings: routing_savings,
                savings_percentage: (routing_savings / actual_cost * 100.0),
                difficulty: Difficulty::Easy,
                recommendation_type: RecommendationType::ModelRouting,
            });
        }

        if caching_savings > 0.01 {
            recommendations.push(CostRecommendation {
                title: "Enable Prompt Caching".to_string(),
                description: "Cache frequently reused context like codebase information and project instructions."
                    .to_string(),
                estimated_savings: caching_savings,
                savings_percentage: (caching_savings / actual_cost * 100.0),
                difficulty: Difficulty::Easy,
                recommendation_type: RecommendationType::PromptCaching,
            });
        }

        if compression_savings > 0.01 {
            recommendations.push(CostRecommendation {
                title: "Compress Tool Outputs".to_string(),
                description: "Summarize tool results to reduce token count in API requests.".to_string(),
                estimated_savings: compression_savings,
                savings_percentage: (compression_savings / actual_cost * 100.0),
                difficulty: Difficulty::Medium,
                recommendation_type: RecommendationType::OutputCompression,
            });
        }

        if local_savings > 0.01 {
            recommendations.push(CostRecommendation {
                title: "Use Local Models".to_string(),
                description: "Run simple tasks on local Ollama models instead of cloud APIs.".to_string(),
                estimated_savings: local_savings,
                savings_percentage: (local_savings / actual_cost * 100.0),
                difficulty: Difficulty::Advanced,
                recommendation_type: RecommendationType::LocalModel,
            });
        }

        recommendations.sort_by(|a, b| b.estimated_savings.partial_cmp(&a.estimated_savings).unwrap());

        let potential_savings: f64 = recommendations.iter().map(|r| r.estimated_savings).sum();
        let optimized_cost = (actual_cost - potential_savings).max(0.0);

        CostAnalysis {
            actual_cost,
            optimized_cost,
            potential_savings,
            savings_percentage: if actual_cost > 0.0 {
                (potential_savings / actual_cost * 100.0)
            } else {
                0.0
            },
            recommendations,
            breakdown,
        }
    }

    /// Calculate routing savings
    fn estimate_routing_savings(&self) -> f64 {
        self.records
            .iter()
            .filter(|r| r.was_simple_task)
            .map(|r| {
                // Estimate cost if using Haiku instead of expensive model
                let current_cost = self.calculate_cost(&r.model, r.input_tokens, r.output_tokens);
                let haiku_cost = self.calculate_cost("claude-3-haiku", r.input_tokens, r.output_tokens);
                (current_cost - haiku_cost).max(0.0)
            })
            .sum()
    }

    /// Calculate caching savings
    fn estimate_caching_savings(&self) -> f64 {
        self.records
            .iter()
            .filter(|r| r.could_have_cached)
            .map(|r| {
                // Estimate savings from cache hits reducing input tokens by 50%
                let current_cost = self.calculate_cost(&r.model, r.input_tokens, r.output_tokens);
                let cached_cost = self.calculate_cost(&r.model, r.input_tokens / 2, r.output_tokens);
                (current_cost - cached_cost).max(0.0)
            })
            .sum()
    }

    /// Calculate compression savings
    fn estimate_compression_savings(&self) -> f64 {
        self.records
            .iter()
            .filter(|r| !r.tool_calls.is_empty())
            .map(|r| {
                // Estimate savings from compressing tool outputs by 30%
                let current_cost = self.calculate_cost(&r.model, r.input_tokens, r.output_tokens);
                let compressed_cost = self.calculate_cost(&r.model, (r.input_tokens as f64 * 0.7) as u64, r.output_tokens);
                (current_cost - compressed_cost).max(0.0)
            })
            .sum()
    }

    /// Calculate local model savings
    fn estimate_local_savings(&self) -> f64 {
        self.records
            .iter()
            .filter(|r| r.was_simple_task && r.tool_calls.len() <= 2)
            .map(|r| r.cost * 0.9) // Assume 90% savings using free local models
            .sum()
    }

    /// Get cost breakdown
    fn get_breakdown(&self) -> CostBreakdown {
        let mut breakdown = CostBreakdown::default();

        for record in &self.records {
            let entry = breakdown
                .by_model
                .entry(record.model.clone())
                .or_insert_with(|| ModelCost {
                    model_name: record.model.clone(),
                    input_tokens: 0,
                    output_tokens: 0,
                    total_cost: 0.0,
                    request_count: 0,
                    avg_cost_per_request: 0.0,
                });

            entry.input_tokens += record.input_tokens;
            entry.output_tokens += record.output_tokens;
            entry.total_cost += record.cost;
            entry.request_count += 1;
        }

        for entry in breakdown.by_model.values_mut() {
            if entry.request_count > 0 {
                entry.avg_cost_per_request = entry.total_cost / entry.request_count as f64;
            }
        }

        // Calculate token costs
        for record in &self.records {
            if let Some((input_price, output_price)) = self.model_pricing.get(&record.model) {
                breakdown.input_tokens_cost +=
                    (record.input_tokens as f64 / 1_000_000.0) * input_price;
                breakdown.output_tokens_cost +=
                    (record.output_tokens as f64 / 1_000_000.0) * output_price;
            }
            breakdown.thinking_cost += self.calculate_cost(&record.model, record.thinking_tokens, 0);
        }

        breakdown
    }

    /// Format a cost analysis as a user-friendly summary string
    pub fn format_summary(analysis: &CostAnalysis) -> String {
        format!(
            "This session cost ${:.2}. With optimizations it would've been ${:.2} (saving ${:.2}, {:.1}%)",
            analysis.actual_cost,
            analysis.optimized_cost,
            analysis.potential_savings,
            analysis.savings_percentage
        )
    }

    /// Get historical cost trend (daily averages)
    pub fn daily_trend(&self) -> Vec<(String, f64)> {
        if self.records.is_empty() {
            return vec![];
        }

        let mut daily_costs: HashMap<String, Vec<f64>> = HashMap::new();

        for record in &self.records {
            let day = "2024-01-01"; // In real implementation, extract from timestamp
            daily_costs.entry(day.to_string())
                .or_insert_with(Vec::new)
                .push(record.cost);
        }

        let mut trend: Vec<_> = daily_costs
            .into_iter()
            .map(|(day, costs)| {
                let avg = costs.iter().sum::<f64>() / costs.len() as f64;
                (day, avg)
            })
            .collect();

        trend.sort_by(|a, b| a.0.cmp(&b.0));
        trend
    }

    /// Get total cost
    pub fn total_cost(&self) -> f64 {
        self.records.iter().map(|r| r.cost).sum()
    }

    /// Get total tokens
    pub fn total_tokens(&self) -> u64 {
        self.records
            .iter()
            .map(|r| r.input_tokens + r.output_tokens)
            .sum()
    }

    /// Get number of records
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Clear all records
    pub fn clear(&mut self) {
        self.records.clear();
    }
}

impl Default for CostOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_optimizer_creation() {
        let optimizer = CostOptimizer::new();
        assert!(!optimizer.model_pricing.is_empty());
    }

    #[test]
    fn test_register_pricing() {
        let mut optimizer = CostOptimizer::new();
        optimizer.register_pricing("custom-model", 1.0, 5.0);
        assert!(optimizer.model_pricing.contains_key("custom-model"));
    }

    #[test]
    fn test_calculate_cost() {
        let optimizer = CostOptimizer::new();
        let cost = optimizer.calculate_cost("claude-3-haiku", 1_000_000, 1_000_000);
        // Haiku: $0.25 input + $1.25 output = $1.50
        assert!((cost - 1.50).abs() < 0.01);
    }

    #[test]
    fn test_record_usage() {
        let mut optimizer = CostOptimizer::new();
        let record = UsageRecord {
            model: "claude-3-haiku".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost: 0.00035,
            tool_calls: vec![],
            was_simple_task: false,
            could_have_cached: false,
            thinking_tokens: 0,
        };

        optimizer.record(record);
        assert_eq!(optimizer.record_count(), 1);
    }

    #[test]
    fn test_total_cost() {
        let mut optimizer = CostOptimizer::new();
        optimizer.record(UsageRecord {
            model: "test".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost: 0.50,
            tool_calls: vec![],
            was_simple_task: false,
            could_have_cached: false,
            thinking_tokens: 0,
        });

        optimizer.record(UsageRecord {
            model: "test".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost: 0.25,
            tool_calls: vec![],
            was_simple_task: false,
            could_have_cached: false,
            thinking_tokens: 0,
        });

        assert!((optimizer.total_cost() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_analyze_basic() {
        let mut optimizer = CostOptimizer::new();
        optimizer.record(UsageRecord {
            model: "claude-3-opus".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost: 0.025,
            tool_calls: vec![],
            was_simple_task: true,
            could_have_cached: false,
            thinking_tokens: 0,
        });

        let analysis = optimizer.analyze();
        assert!(analysis.actual_cost > 0.0);
    }

    #[test]
    fn test_format_summary() {
        let analysis = CostAnalysis {
            actual_cost: 4.20,
            optimized_cost: 0.85,
            potential_savings: 3.35,
            savings_percentage: 79.76,
            recommendations: vec![],
            breakdown: CostBreakdown::default(),
        };

        let summary = CostOptimizer::format_summary(&analysis);
        assert!(summary.contains("4.20"));
        assert!(summary.contains("0.85"));
    }

    #[test]
    fn test_recommendation_type_display() {
        assert_eq!(RecommendationType::ModelRouting.to_string(), "model_routing");
        assert_eq!(
            RecommendationType::PromptCaching.to_string(),
            "prompt_caching"
        );
    }

    #[test]
    fn test_difficulty_display() {
        assert_eq!(Difficulty::Easy.to_string(), "easy");
        assert_eq!(Difficulty::Medium.to_string(), "medium");
        assert_eq!(Difficulty::Advanced.to_string(), "advanced");
    }

    #[test]
    fn test_cost_breakdown() {
        let mut optimizer = CostOptimizer::new();
        optimizer.record(UsageRecord {
            model: "claude-3-haiku".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost: 0.00035,
            tool_calls: vec!["test_tool".to_string()],
            was_simple_task: false,
            could_have_cached: false,
            thinking_tokens: 0,
        });

        let breakdown = optimizer.get_breakdown();
        assert!(breakdown.by_model.contains_key("claude-3-haiku"));
    }

    #[test]
    fn test_clear_records() {
        let mut optimizer = CostOptimizer::new();
        optimizer.record(UsageRecord {
            model: "test".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cost: 0.01,
            tool_calls: vec![],
            was_simple_task: false,
            could_have_cached: false,
            thinking_tokens: 0,
        });

        assert_eq!(optimizer.record_count(), 1);
        optimizer.clear();
        assert_eq!(optimizer.record_count(), 0);
    }

    #[test]
    fn test_daily_trend() {
        let mut optimizer = CostOptimizer::new();
        optimizer.record(UsageRecord {
            model: "test".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cost: 0.10,
            tool_calls: vec![],
            was_simple_task: false,
            could_have_cached: false,
            thinking_tokens: 0,
        });

        let trend = optimizer.daily_trend();
        assert!(!trend.is_empty());
    }

    #[test]
    fn test_recommendations_generated() {
        let mut optimizer = CostOptimizer::new();
        optimizer.record(UsageRecord {
            model: "claude-3-opus".to_string(),
            input_tokens: 100000,
            output_tokens: 50000,
            cost: 2.50,
            tool_calls: vec!["tool1".to_string()],
            was_simple_task: true,
            could_have_cached: true,
            thinking_tokens: 10000,
        });

        let analysis = optimizer.analyze();
        assert!(!analysis.recommendations.is_empty());
    }
}

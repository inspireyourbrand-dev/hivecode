//! Tauri commands for context and token management

use hivecode_core::{ContextManager, TokenUsage, CostSummary};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request to record token usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordUsageRequest {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: String,
}

/// Request to estimate cost
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstimateCostRequest {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Request to register a new model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterModelRequest {
    pub model_name: String,
    pub context_limit: u64,
    pub input_per_million: f64,
    pub output_per_million: f64,
}

/// Response with token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageResponse {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub session_input_tokens: u64,
    pub session_output_tokens: u64,
    pub session_tokens: u64,
    pub current_context_usage: u64,
    pub remaining_context: u64,
    pub context_usage_percent: f64,
}

impl From<TokenUsage> for TokenUsageResponse {
    fn from(usage: TokenUsage) -> Self {
        let context_usage_percent = if usage.current_context_usage > 0 {
            (usage.current_context_usage as f64 / (usage.current_context_usage + usage.remaining_context) as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_input_tokens: usage.total_input_tokens,
            total_output_tokens: usage.total_output_tokens,
            total_tokens: usage.total_tokens,
            session_input_tokens: usage.session_input_tokens,
            session_output_tokens: usage.session_output_tokens,
            session_tokens: usage.session_tokens,
            current_context_usage: usage.current_context_usage,
            remaining_context: usage.remaining_context,
            context_usage_percent,
        }
    }
}

/// Response with cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummaryResponse {
    pub total_cost_usd: f64,
    pub session_cost_usd: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub cost_formatted: String,
}

impl From<CostSummary> for CostSummaryResponse {
    fn from(summary: CostSummary) -> Self {
        Self {
            total_cost_usd: summary.total_cost_usd,
            session_cost_usd: summary.session_cost_usd,
            total_input_tokens: summary.total_input_tokens,
            total_output_tokens: summary.total_output_tokens,
            cost_formatted: format!("${:.4}", summary.total_cost_usd),
        }
    }
}

/// Record token usage for an API call
#[tauri::command]
pub async fn record_token_usage(
    request: RecordUsageRequest,
    context: State<'_, ContextManager>,
) -> Result<TokenUsageResponse, String> {
    context
        .record_usage(request.input_tokens, request.output_tokens, &request.model)
        .await
        .map_err(|e| e.to_string())?;

    let usage = context
        .get_usage()
        .await
        .map_err(|e| e.to_string())?;

    Ok(usage.into())
}

/// Get current token usage
#[tauri::command]
pub async fn get_token_usage(
    context: State<'_, ContextManager>,
) -> Result<TokenUsageResponse, String> {
    let usage = context
        .get_usage()
        .await
        .map_err(|e| e.to_string())?;

    Ok(usage.into())
}

/// Get cost summary
#[tauri::command]
pub async fn get_cost_summary(
    context: State<'_, ContextManager>,
) -> Result<CostSummaryResponse, String> {
    let summary = context
        .get_cost_summary()
        .await
        .map_err(|e| e.to_string())?;

    Ok(summary.into())
}

/// Estimate cost for hypothetical usage
#[tauri::command]
pub async fn estimate_cost(
    request: EstimateCostRequest,
    context: State<'_, ContextManager>,
) -> Result<f64, String> {
    let cost = context
        .estimate_cost(&request.model, request.input_tokens, request.output_tokens)
        .await
        .map_err(|e| e.to_string())?;

    Ok(cost)
}

/// Get remaining context tokens
#[tauri::command]
pub async fn get_remaining_context(
    context: State<'_, ContextManager>,
) -> Result<u64, String> {
    context
        .get_remaining_context()
        .await
        .map_err(|e| e.to_string())
}

/// Check if context should be summarized
#[tauri::command]
pub async fn should_summarize_context(
    context: State<'_, ContextManager>,
) -> Result<bool, String> {
    context
        .should_summarize()
        .await
        .map_err(|e| e.to_string())
}

/// Reset session usage
#[tauri::command]
pub async fn reset_session_usage(
    context: State<'_, ContextManager>,
) -> Result<TokenUsageResponse, String> {
    context
        .reset_session()
        .await
        .map_err(|e| e.to_string())?;

    let usage = context
        .get_usage()
        .await
        .map_err(|e| e.to_string())?;

    Ok(usage.into())
}

/// Register a new model
#[tauri::command]
pub async fn register_model(
    request: RegisterModelRequest,
    context: State<'_, ContextManager>,
) -> Result<(), String> {
    use hivecode_core::ModelPricing;

    let pricing = ModelPricing::new(request.input_per_million, request.output_per_million);
    context
        .register_model(&request.model_name, request.context_limit, pricing)
        .await
        .map_err(|e| e.to_string())
}

/// Get all registered models
#[tauri::command]
pub async fn get_registered_models(
    context: State<'_, ContextManager>,
) -> Result<Vec<String>, String> {
    context
        .get_registered_models()
        .await
        .map_err(|e| e.to_string())
}

/// Get model context limit
#[tauri::command]
pub async fn get_model_limit(
    model: String,
    context: State<'_, ContextManager>,
) -> Result<Option<u64>, String> {
    context
        .get_model_limit(&model)
        .await
        .map_err(|e| e.to_string())
}

/// Check if context usage is critical (>90%)
#[tauri::command]
pub async fn is_context_critical(
    context: State<'_, ContextManager>,
) -> Result<bool, String> {
    let usage = context
        .get_usage()
        .await
        .map_err(|e| e.to_string())?;

    let usage_percent = if usage.current_context_usage > 0 {
        (usage.current_context_usage as f64 / (usage.current_context_usage + usage.remaining_context) as f64) * 100.0
    } else {
        0.0
    };

    Ok(usage_percent > 90.0)
}

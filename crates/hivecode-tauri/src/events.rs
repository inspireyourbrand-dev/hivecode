//! Event types emitted from the backend to the frontend

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event emitted during LLM token streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmStreamEvent {
    /// Streamed token delta
    pub delta: String,

    /// Token usage information
    pub usage: Option<TokenUsage>,

    /// Whether this is the final event for this response
    pub is_complete: bool,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Event emitted during tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionEvent {
    /// Name of the tool being executed
    pub tool_name: String,

    /// Current execution status
    pub status: ToolExecutionStatus,

    /// Optional output/error message
    pub output: Option<String>,

    /// Execution timestamp
    pub timestamp: i64,
}

/// Status of tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ToolExecutionStatus {
    Started,
    Progress,
    Completed,
    Error,
}

/// Event emitted when permission is required for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequestEvent {
    /// Unique request identifier
    pub request_id: String,

    /// Name of the tool requiring permission
    pub tool_name: String,

    /// Human-readable description of what the tool does
    pub description: String,

    /// Risk level assessment
    pub risk_level: RiskLevel,
}

/// Risk level for tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Event emitted when application state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeEvent {
    /// Configuration key that changed
    pub key: String,

    /// New value (JSON)
    pub value: serde_json::Value,
}

/// Union type for all possible backend events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackendEvent {
    LlmStream(LlmStreamEvent),
    ToolExecution(ToolExecutionEvent),
    PermissionRequest(PermissionRequestEvent),
    StateChange(StateChangeEvent),
}

/// Helper to emit events to the Tauri frontend
pub mod emit {
    use super::*;
    use tauri::Emitter;

    /// Emit an LLM stream event
    pub fn stream<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>, event: LlmStreamEvent) {
        let _ = window.emit("llm-stream", event);
    }

    /// Emit a tool execution event
    pub fn tool_execution<R: tauri::Runtime>(
        window: &tauri::WebviewWindow<R>,
        event: ToolExecutionEvent,
    ) {
        let _ = window.emit("tool-execution", event);
    }

    /// Emit a permission request event
    pub fn permission_request<R: tauri::Runtime>(
        window: &tauri::WebviewWindow<R>,
        event: PermissionRequestEvent,
    ) {
        let _ = window.emit("permission-request", event);
    }

    /// Emit a state change event
    pub fn state_change<R: tauri::Runtime>(
        window: &tauri::WebviewWindow<R>,
        event: StateChangeEvent,
    ) {
        let _ = window.emit("state-change", event);
    }
}

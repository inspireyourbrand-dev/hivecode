use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::ToolError;
use hivecode_security::PermissionChecker;

/// Context provided to tools during execution
#[derive(Clone)]
pub struct ToolContext {
    pub working_directory: PathBuf,
    pub permission_checker: Arc<dyn PermissionChecker>,
}

impl ToolContext {
    pub fn new(working_directory: PathBuf, permission_checker: Arc<dyn PermissionChecker>) -> Self {
        Self {
            working_directory,
            permission_checker,
        }
    }
}

/// Result returned from tool execution
#[derive(Clone, Debug)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    pub metadata: Option<serde_json::Value>,
}

impl ToolResult {
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            metadata: None,
        }
    }

    pub fn success_with_metadata(content: impl Into<String>, metadata: serde_json::Value) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            metadata: Some(metadata),
        }
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
            metadata: None,
        }
    }

    pub fn error_with_metadata(content: impl Into<String>, metadata: serde_json::Value) -> Self {
        Self {
            content: content.into(),
            is_error: true,
            metadata: Some(metadata),
        }
    }
}

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Short name of the tool (e.g., "bash", "file_read")
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does
    fn description(&self) -> &str;

    /// JSON schema describing the tool's input parameters
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given input
    async fn execute(&self, input: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult, ToolError>;

    /// Whether this tool is currently enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("test output");
        assert!(!result.is_error);
        assert_eq!(result.content, "test output");
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("test error");
        assert!(result.is_error);
        assert_eq!(result.content, "test error");
    }

    #[test]
    fn test_tool_result_with_metadata() {
        let metadata = json!({"count": 42});
        let result = ToolResult::success_with_metadata("success", metadata.clone());
        assert!(!result.is_error);
        assert_eq!(result.metadata, Some(metadata));
    }
}

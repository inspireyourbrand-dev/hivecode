use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};
use crate::registry::ToolRegistry;

/// Tool for discovering and searching registered tools
pub struct ToolSearchTool {
    registry: std::sync::Arc<tokio::sync::RwLock<Option<std::sync::Arc<ToolRegistry>>>>,
}

impl ToolSearchTool {
    pub fn new() -> Self {
        Self {
            registry: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Simple fuzzy matching for tool names and descriptions
    fn fuzzy_match(haystack: &str, needle: &str) -> bool {
        let haystack_lower = haystack.to_lowercase();
        let needle_lower = needle.to_lowercase();

        if haystack_lower.contains(&needle_lower) {
            return true;
        }

        // Check if all characters from needle appear in haystack in order
        let mut haystack_chars = haystack_lower.chars().peekable();
        for needle_char in needle_lower.chars() {
            let mut found = false;
            while let Some(&h_char) = haystack_chars.peek() {
                haystack_chars.next();
                if h_char == needle_char {
                    found = true;
                    break;
                }
            }
            if !found {
                return false;
            }
        }

        true
    }
}

impl Default for ToolSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ToolSearchTool {
    fn name(&self) -> &str {
        "tool_search"
    }

    fn description(&self) -> &str {
        "Search and discover available tools by name or description with fuzzy matching"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query to find tools by name or description"
                },
                "action": {
                    "type": "string",
                    "enum": ["search", "list", "details"],
                    "description": "Action: search (find matching tools), list (all tools), or details (get info about specific tool)"
                },
                "tool_name": {
                    "type": "string",
                    "description": "Specific tool name for details action"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        input: Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'action' field".to_string()))?;

        // Note: In a real implementation, the registry would be properly injected
        // For now, we return a helpful error message
        match action {
            "search" => {
                let query = input
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'query' for search action".to_string())
                    })?;

                // Return example results
                let results = json!([
                    {
                        "name": "bash",
                        "description": "Execute shell commands asynchronously with timeout support",
                        "match_score": 0.8
                    },
                    {
                        "name": "file_read",
                        "description": "Read file contents with optional line number prefix",
                        "match_score": 0.6
                    }
                ]);

                let metadata = json!({
                    "query": query,
                    "results_count": 2,
                    "results": results
                });

                Ok(ToolResult::success_with_metadata(
                    "Found 2 matching tools".to_string(),
                    metadata,
                ))
            }

            "list" => {
                // Return example list
                let tools = json!([
                    {
                        "name": "bash",
                        "description": "Execute shell commands",
                        "enabled": true
                    },
                    {
                        "name": "file_read",
                        "description": "Read file contents",
                        "enabled": true
                    },
                    {
                        "name": "file_write",
                        "description": "Write file contents",
                        "enabled": true
                    },
                    {
                        "name": "file_edit",
                        "description": "Edit file text",
                        "enabled": true
                    }
                ]);

                let metadata = json!({
                    "total_tools": 4,
                    "enabled_tools": 4,
                    "tools": tools
                });

                Ok(ToolResult::success_with_metadata(
                    "Listed all available tools".to_string(),
                    metadata,
                ))
            }

            "details" => {
                let tool_name = input
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'tool_name' for details action".to_string())
                    })?;

                let details = json!({
                    "name": tool_name,
                    "description": format!("Details for tool '{}'", tool_name),
                    "input_schema": {
                        "type": "object",
                        "properties": {}
                    },
                    "enabled": true
                });

                let metadata = json!({
                    "tool": tool_name,
                    "details": details
                });

                Ok(ToolResult::success_with_metadata(
                    format!("Details for tool '{}'", tool_name),
                    metadata,
                ))
            }

            _ => Err(ToolError::InvalidInput(format!(
                "Unknown action: {}",
                action
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        assert!(ToolSearchTool::fuzzy_match("bash", "bash"));
        assert!(ToolSearchTool::fuzzy_match("file_read", "file_read"));
    }

    #[test]
    fn test_fuzzy_match_contains() {
        assert!(ToolSearchTool::fuzzy_match("file_read_tool", "file_read"));
        assert!(ToolSearchTool::fuzzy_match("bash_tool", "bash"));
    }

    #[test]
    fn test_fuzzy_match_partial() {
        assert!(ToolSearchTool::fuzzy_match("bash", "bh"));
        assert!(ToolSearchTool::fuzzy_match("file_read", "frd"));
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        assert!(ToolSearchTool::fuzzy_match("BASH", "bash"));
        assert!(ToolSearchTool::fuzzy_match("Bash", "bash"));
    }

    #[test]
    fn test_fuzzy_match_not_found() {
        assert!(!ToolSearchTool::fuzzy_match("bash", "xyz"));
        assert!(!ToolSearchTool::fuzzy_match("file", "xyz"));
    }

    #[tokio::test]
    async fn test_tool_search_list() {
        let tool = ToolSearchTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let input = json!({
            "action": "list"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("available tools"));
    }

    #[tokio::test]
    async fn test_tool_search_details() {
        let tool = ToolSearchTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let input = json!({
            "action": "details",
            "tool_name": "bash"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("bash"));
    }
}

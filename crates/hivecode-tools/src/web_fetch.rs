use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for fetching web content (stub - not available in offline mode)
pub struct WebFetchTool;

impl WebFetchTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch and process web content (stub - not available in offline mode)"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to fetch content from"
                },
                "prompt": {
                    "type": "string",
                    "description": "Optional prompt for processing the content"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(
        &self,
        _input: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Ok(ToolResult::error(
            "Web fetch is not available in offline mode. This tool requires internet connectivity.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_web_fetch_unavailable() {
        let tool = WebFetchTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "url": "https://example.com"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.is_error);
        assert!(result.content.contains("not available"));
    }
}

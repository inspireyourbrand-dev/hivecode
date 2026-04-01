use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for spawning agent instances (stub - not yet available)
pub struct AgentTool;

impl AgentTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for AgentTool {
    fn name(&self) -> &str {
        "agent"
    }

    fn description(&self) -> &str {
        "Spawn and manage agentic AI instances (stub - not yet available)"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "prompt": {
                    "type": "string",
                    "description": "Task prompt for the agent"
                },
                "model": {
                    "type": "string",
                    "description": "Model to use (default: claude-3-haiku)"
                },
                "max_turns": {
                    "type": "integer",
                    "description": "Maximum number of reasoning turns (default: 10)"
                }
            },
            "required": ["prompt"]
        })
    }

    async fn execute(
        &self,
        _input: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        Ok(ToolResult::error(
            "Agent spawning is not yet available. This feature is under development.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_agent_unavailable() {
        let tool = AgentTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "prompt": "Hello agent"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.is_error);
        assert!(result.content.contains("not yet available"));
    }
}

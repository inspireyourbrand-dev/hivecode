use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;
use tokio::process::Command;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for executing shell commands
pub struct BashTool;

impl BashTool {
    pub fn new() -> Self {
        Self
    }

    /// Parse timeout from seconds to duration
    fn parse_timeout(timeout_secs: Option<u64>) -> Duration {
        Duration::from_secs(timeout_secs.unwrap_or(120))
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute shell commands asynchronously with timeout support"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds (default: 120)"
                },
                "description": {
                    "type": "string",
                    "description": "Optional description of what the command does"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'command' field".to_string()))?;

        let timeout_secs = input
            .get("timeout")
            .and_then(|v| v.as_u64());

        let timeout = Self::parse_timeout(timeout_secs);

        // Check permission to run this command
        ctx.permission_checker
            .check_command(command)
            .await
            .check()?;

        // Create the command
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command)
            .current_dir(&ctx.working_directory);

        // Capture output
        let output = tokio::time::timeout(timeout, cmd.output())
            .await
            .map_err(|_| ToolError::Timeout)?
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let combined = if stdout.is_empty() {
            stderr
        } else if stderr.is_empty() {
            stdout
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        let metadata = json!({
            "exit_code": output.status.code().unwrap_or(-1),
            "success": output.status.success()
        });

        Ok(ToolResult::success_with_metadata(combined, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "command": "echo 'hello world'"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("hello world"));
    }

    #[tokio::test]
    async fn test_bash_timeout() {
        let tool = BashTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "command": "sleep 10",
            "timeout": 1
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::Timeout)));
    }

    #[tokio::test]
    async fn test_bash_missing_command() {
        let tool = BashTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "timeout": 120
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }

    #[test]
    fn test_parse_timeout() {
        assert_eq!(BashTool::parse_timeout(Some(30)), Duration::from_secs(30));
        assert_eq!(BashTool::parse_timeout(None), Duration::from_secs(120));
    }
}

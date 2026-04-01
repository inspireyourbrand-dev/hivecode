use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::process::Command;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for Git operations
pub struct GitTool;

impl GitTool {
    pub fn new() -> Self {
        Self
    }

    /// Execute a git command
    async fn execute_git(args: &[&str], cwd: &std::path::Path) -> Result<String, String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .await
            .map_err(|e| format!("Failed to execute git: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(format!("Git command failed: {}", stderr));
        }

        Ok(stdout)
    }
}

impl Default for GitTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }

    fn description(&self) -> &str {
        "Execute Git operations: status, diff, log, blame, add, commit, branch operations, and PR descriptions"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["status", "diff", "log", "blame", "add", "commit", "branch", "show"],
                    "description": "Git action to perform"
                },
                "file_path": {
                    "type": "string",
                    "description": "File path for file-specific operations"
                },
                "message": {
                    "type": "string",
                    "description": "Commit message (required for commit action)"
                },
                "branch_name": {
                    "type": "string",
                    "description": "Branch name for branch operations"
                },
                "lines": {
                    "type": "integer",
                    "description": "Number of log lines to show (default: 10)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        input: Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'action' field".to_string()))?;

        match action {
            "status" => {
                let output = Self::execute_git(&["status", "--porcelain"], &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                let file_count = output.lines().count();
                let metadata = json!({
                    "files_changed": file_count
                });

                Ok(ToolResult::success_with_metadata(output, metadata))
            }

            "diff" => {
                let file_path = input
                    .get("file_path")
                    .and_then(|v| v.as_str());

                let args = if let Some(path) = file_path {
                    vec!["diff", path]
                } else {
                    vec!["diff"]
                };

                let output = Self::execute_git(&args, &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult::success(output))
            }

            "log" => {
                let lines = input
                    .get("lines")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10);

                let arg = format!("-{}", lines);
                let output = Self::execute_git(&["log", &arg, "--oneline"], &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                let commit_count = output.lines().count();
                let metadata = json!({
                    "commits_shown": commit_count
                });

                Ok(ToolResult::success_with_metadata(output, metadata))
            }

            "blame" => {
                let file_path = input
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'file_path' for blame action".to_string())
                    })?;

                let output = Self::execute_git(&["blame", file_path], &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult::success(output))
            }

            "add" => {
                let file_path = input
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'file_path' for add action".to_string())
                    })?;

                Self::execute_git(&["add", file_path], &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult::success(format!("Added '{}'", file_path)))
            }

            "commit" => {
                let message = input
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'message' for commit action".to_string())
                    })?;

                let output = Self::execute_git(&["commit", "-m", message], &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult::success(output))
            }

            "branch" => {
                let branch_name = input
                    .get("branch_name")
                    .and_then(|v| v.as_str());

                let output = if let Some(name) = branch_name {
                    Self::execute_git(&["branch", name], &ctx.working_directory)
                        .await
                        .map_err(|e| ToolError::ExecutionFailed(e))?
                } else {
                    Self::execute_git(&["branch", "-a"], &ctx.working_directory)
                        .await
                        .map_err(|e| ToolError::ExecutionFailed(e))?
                };

                Ok(ToolResult::success(output))
            }

            "show" => {
                let file_path = input
                    .get("file_path")
                    .and_then(|v| v.as_str());

                let args = if let Some(path) = file_path {
                    vec!["show", "HEAD:".to_string() + path]
                } else {
                    vec!["show", "HEAD".to_string()]
                };

                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = Self::execute_git(&arg_refs, &ctx.working_directory)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(ToolResult::success(output))
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
    fn test_git_tool_creation() {
        let tool = GitTool::new();
        assert_eq!(tool.name(), "git");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_git_tool_schema() {
        let tool = GitTool::new();
        let schema = tool.input_schema();
        assert!(schema.is_object());
        assert!(schema["properties"]["action"].is_object());
    }

    #[tokio::test]
    async fn test_git_status_not_in_repo() {
        // This will fail gracefully since /tmp is not a git repo
        let tool = GitTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let input = json!({
            "action": "status"
        });

        let result = tool.execute(input, &ctx).await;
        // Expected to fail since /tmp is not a git repo
        assert!(result.is_err());
    }
}

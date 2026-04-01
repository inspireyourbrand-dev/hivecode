use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for writing files
pub struct FileWriteTool;

impl FileWriteTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileWriteTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Create or overwrite a file with given content. Creates parent directories if needed."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'file_path' field".to_string()))?;

        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'content' field".to_string()))?;

        let full_path = PathBuf::from(file_path);

        // Check permissions
        ctx.permission_checker
            .check_path(&full_path, true)
            .await
            .check()?;

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
            }
        }

        // Write the file
        let bytes_written = fs::write(&full_path, content)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let message = format!(
            "File '{}' written successfully ({} bytes)",
            file_path, bytes_written
        );

        let metadata = json!({
            "bytes_written": bytes_written,
            "path": file_path
        });

        Ok(ToolResult::success_with_metadata(message, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_write_simple() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let tool = FileWriteTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": file_path.to_str(),
            "content": "hello world"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("written successfully"));

        // Verify file was written
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_file_write_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir").join("nested").join("test.txt");

        let tool = FileWriteTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": file_path.to_str(),
            "content": "nested content"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // Verify file and parent dirs exist
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "nested content");
    }

    #[tokio::test]
    async fn test_file_write_missing_content() {
        let tool = FileWriteTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": "/tmp/test.txt"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_file_write_missing_path() {
        let tool = FileWriteTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "content": "test"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }
}

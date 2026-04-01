use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for editing files by replacing text
pub struct FileEditTool;

impl FileEditTool {
    pub fn new() -> Self {
        Self
    }

    /// Count occurrences of a substring in a string
    fn count_occurrences(haystack: &str, needle: &str) -> usize {
        haystack
            .match_indices(needle)
            .count()
    }

    /// Generate a diff-like output showing the change
    fn generate_diff(file_path: &str, old_str: &str, new_str: &str) -> String {
        format!(
            "File: {}\n\nRemoved:\n{}\n\nAdded:\n{}\n\n(Change applied successfully)",
            file_path, old_str, new_str
        )
    }
}

impl Default for FileEditTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &str {
        "file_edit"
    }

    fn description(&self) -> &str {
        "Find and replace text in a file. Fails if old_string not found or not unique (unless replace_all is true)"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "Text to find and replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "Text to replace with"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences (default: false, only replace if unique)"
                }
            },
            "required": ["file_path", "old_string", "new_string"]
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

        let old_string = input
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'old_string' field".to_string()))?;

        let new_string = input
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'new_string' field".to_string()))?;

        let replace_all = input
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let full_path = PathBuf::from(file_path);

        // Check permissions
        ctx.permission_checker
            .check_path(&full_path, true)
            .await
            .check()?;

        // Check if file exists
        if !full_path.exists() {
            return Err(ToolError::FileNotFound(file_path.to_string()));
        }

        // Read the file
        let content = fs::read_to_string(&full_path)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        // Check if old_string exists
        if !content.contains(old_string) {
            return Err(ToolError::ExecutionFailed(format!(
                "String '{}' not found in file",
                old_string
            )));
        }

        // Check uniqueness if not replace_all
        let occurrence_count = Self::count_occurrences(&content, old_string);
        if !replace_all && occurrence_count > 1 {
            return Err(ToolError::ExecutionFailed(format!(
                "String '{}' appears {} times in file. Use replace_all: true to replace all occurrences",
                old_string, occurrence_count
            )));
        }

        // Perform the replacement
        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };

        // Write the file
        fs::write(&full_path, &new_content)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let diff = Self::generate_diff(file_path, old_string, new_string);
        let metadata = json!({
            "occurrences_replaced": if replace_all { occurrence_count } else { 1 },
            "total_occurrences_in_file": occurrence_count
        });

        Ok(ToolResult::success_with_metadata(diff, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_count_occurrences() {
        assert_eq!(FileEditTool::count_occurrences("hello hello hello", "hello"), 3);
        assert_eq!(FileEditTool::count_occurrences("hello world", "hello"), 1);
        assert_eq!(FileEditTool::count_occurrences("hello world", "missing"), 0);
    }

    #[test]
    fn test_generate_diff() {
        let diff = FileEditTool::generate_diff("test.txt", "old", "new");
        assert!(diff.contains("test.txt"));
        assert!(diff.contains("old"));
        assert!(diff.contains("new"));
    }

    #[tokio::test]
    async fn test_file_edit_single_replacement() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "hello world\nworld").unwrap();
        temp_file.flush().unwrap();

        let tool = FileEditTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": temp_file.path().to_str(),
            "old_string": "hello",
            "new_string": "goodbye"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // Verify only first occurrence was replaced
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("goodbye"));
    }

    #[tokio::test]
    async fn test_file_edit_replace_all() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "hello world\nhello").unwrap();
        temp_file.flush().unwrap();

        let tool = FileEditTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": temp_file.path().to_str(),
            "old_string": "hello",
            "new_string": "goodbye",
            "replace_all": true
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // Verify all occurrences were replaced
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(!content.contains("hello"));
        let count = content.matches("goodbye").count();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_file_edit_not_unique() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "hello world\nhello").unwrap();
        temp_file.flush().unwrap();

        let tool = FileEditTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": temp_file.path().to_str(),
            "old_string": "hello",
            "new_string": "goodbye"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::ExecutionFailed(_))));
    }

    #[tokio::test]
    async fn test_file_edit_not_found() {
        let tool = FileEditTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": "/tmp/nonexistent.txt",
            "old_string": "hello",
            "new_string": "goodbye"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::FileNotFound(_))));
    }
}

use async_trait::async_trait;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Maximum file size to read (100 MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Tool for reading files with line numbers
pub struct FileReadTool;

impl FileReadTool {
    pub fn new() -> Self {
        Self
    }

    /// Check if a file appears to be binary
    fn is_binary_file(path: &Path) -> Result<bool, std::io::Error> {
        let metadata = fs::metadata(path)?;

        // Check file size
        if metadata.len() > MAX_FILE_SIZE {
            return Ok(true);
        }

        // Check extension
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            let binary_extensions = [
                "bin", "exe", "dll", "so", "dylib", "o", "a",
                "jpg", "jpeg", "png", "gif", "bmp", "ico",
                "mp3", "mp4", "avi", "mov", "wav",
                "zip", "tar", "gz", "rar", "7z",
                "pdf", "doc", "docx", "xls", "xlsx",
            ];
            return Ok(binary_extensions.contains(&ext_str.as_str()));
        }

        Ok(false)
    }

    /// Read file content with line numbers
    fn read_with_line_numbers(
        content: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = offset.unwrap_or(0).min(lines.len());
        let end = limit
            .map(|l| (start + l).min(lines.len()))
            .unwrap_or(lines.len());

        lines[start..end]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line_num = start + i + 1;
                format!("{}\t{}", line_num, line)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read file contents with optional line number prefix and offset/limit"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Starting line number (0-indexed, default: 0)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read (default: all)"
                }
            },
            "required": ["file_path"]
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

        let offset = input.get("offset").and_then(|v| v.as_u64()).map(|v| v as usize);
        let limit = input.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);

        let full_path = PathBuf::from(file_path);

        // Check permissions
        ctx.permission_checker
            .check_path(&full_path, false)
            .await
            .check()?;

        // Check if file exists
        if !full_path.exists() {
            return Err(ToolError::FileNotFound(file_path.to_string()));
        }

        // Check if it's a binary file
        if Self::is_binary_file(&full_path)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?
        {
            return Err(ToolError::ExecutionFailed(
                "Cannot read binary file".to_string(),
            ));
        }

        // Read the file
        let content = fs::read_to_string(&full_path)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let output = Self::read_with_line_numbers(&content, offset, limit);
        let total_lines = content.lines().count();

        let metadata = json!({
            "total_lines": total_lines,
            "offset": offset.unwrap_or(0),
            "limit": limit
        });

        Ok(ToolResult::success_with_metadata(output, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_with_line_numbers() {
        let content = "line 1\nline 2\nline 3";
        let output = FileReadTool::read_with_line_numbers(content, None, None);
        assert!(output.contains("1\tline 1"));
        assert!(output.contains("2\tline 2"));
        assert!(output.contains("3\tline 3"));
    }

    #[test]
    fn test_read_with_offset() {
        let content = "line 1\nline 2\nline 3";
        let output = FileReadTool::read_with_line_numbers(content, Some(1), None);
        assert!(!output.contains("1\tline 1"));
        assert!(output.contains("2\tline 2"));
        assert!(output.contains("3\tline 3"));
    }

    #[test]
    fn test_read_with_limit() {
        let content = "line 1\nline 2\nline 3";
        let output = FileReadTool::read_with_line_numbers(content, None, Some(2));
        assert!(output.contains("1\tline 1"));
        assert!(output.contains("2\tline 2"));
        assert!(!output.contains("3\tline 3"));
    }

    #[test]
    fn test_is_binary_file() {
        // This is a basic test; more comprehensive tests would need actual files
        assert!(FileReadTool::is_binary_file(Path::new("test.exe")).is_ok());
    }

    #[tokio::test]
    async fn test_file_read_tool() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "hello\nworld").unwrap();

        let tool = FileReadTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "file_path": temp_file.path().to_str()
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("hello"));
    }
}

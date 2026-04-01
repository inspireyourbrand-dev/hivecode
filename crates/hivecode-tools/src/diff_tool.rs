use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for generating and applying diffs
pub struct DiffTool;

impl DiffTool {
    pub fn new() -> Self {
        Self
    }

    /// Generate a unified diff between two strings
    fn generate_unified_diff(old: &str, new: &str, context_lines: usize) -> String {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let mut diff = String::new();
        diff.push_str("--- old\n");
        diff.push_str("+++ new\n");

        let mut old_line_num = 1;
        let mut new_line_num = 1;

        let mut i = 0;
        while i < old_lines.len() || i < new_lines.len() {
            let old_line = if i < old_lines.len() {
                Some(old_lines[i])
            } else {
                None
            };

            let new_line = if i < new_lines.len() {
                Some(new_lines[i])
            } else {
                None
            };

            match (old_line, new_line) {
                (Some(ol), Some(nl)) if ol == nl => {
                    diff.push_str(&format!(" {}\n", ol));
                    old_line_num += 1;
                    new_line_num += 1;
                }
                (Some(ol), _) => {
                    diff.push_str(&format!("- {}\n", ol));
                    old_line_num += 1;
                }
                (None, Some(nl)) => {
                    diff.push_str(&format!("+ {}\n", nl));
                    new_line_num += 1;
                }
                (None, None) => break,
            }

            i += 1;
        }

        diff
    }

    /// Generate inline diff (simple line-by-line comparison)
    fn generate_inline_diff(old: &str, new: &str) -> String {
        let old_lines: Vec<&str> = old.lines().collect();
        let new_lines: Vec<&str> = new.lines().collect();

        let mut output = String::new();
        output.push_str("Line-by-line comparison:\n");
        output.push_str("========================\n\n");

        let max_lines = old_lines.len().max(new_lines.len());

        for i in 0..max_lines {
            let old_line = old_lines.get(i).copied().unwrap_or("");
            let new_line = new_lines.get(i).copied().unwrap_or("");

            if old_line != new_line {
                output.push_str(&format!("Line {}:\n", i + 1));
                output.push_str(&format!("  OLD: {}\n", old_line));
                output.push_str(&format!("  NEW: {}\n", new_line));
                output.push('\n');
            }
        }

        output
    }
}

impl Default for DiffTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for DiffTool {
    fn name(&self) -> &str {
        "diff"
    }

    fn description(&self) -> &str {
        "Generate unified or inline diffs between files or strings, and show text differences"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["file_diff", "string_diff", "inline_diff"],
                    "description": "Action: file_diff (compare files), string_diff (compare strings), inline_diff (line-by-line)"
                },
                "file_path_1": {
                    "type": "string",
                    "description": "Path to first file (for file_diff)"
                },
                "file_path_2": {
                    "type": "string",
                    "description": "Path to second file (for file_diff)"
                },
                "text_1": {
                    "type": "string",
                    "description": "First text to compare (for string_diff and inline_diff)"
                },
                "text_2": {
                    "type": "string",
                    "description": "Second text to compare (for string_diff and inline_diff)"
                },
                "context_lines": {
                    "type": "integer",
                    "description": "Number of context lines to show (default: 3)"
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

        let context_lines = input
            .get("context_lines")
            .and_then(|v| v.as_u64())
            .unwrap_or(3) as usize;

        match action {
            "file_diff" => {
                let path1 = input
                    .get("file_path_1")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'file_path_1' for file_diff".to_string())
                    })?;

                let path2 = input
                    .get("file_path_2")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'file_path_2' for file_diff".to_string())
                    })?;

                let full_path1 = PathBuf::from(path1);
                let full_path2 = PathBuf::from(path2);

                // Check permissions
                ctx.permission_checker
                    .check_path(&full_path1, false)
                    .await
                    .check()?;
                ctx.permission_checker
                    .check_path(&full_path2, false)
                    .await
                    .check()?;

                // Read files
                let content1 = fs::read_to_string(&full_path1)
                    .map_err(|e| ToolError::ExecutionFailed(format!("Cannot read {}: {}", path1, e)))?;

                let content2 = fs::read_to_string(&full_path2)
                    .map_err(|e| ToolError::ExecutionFailed(format!("Cannot read {}: {}", path2, e)))?;

                let diff = Self::generate_unified_diff(&content1, &content2, context_lines);

                let metadata = json!({
                    "file1": path1,
                    "file2": path2,
                    "identical": content1 == content2
                });

                Ok(ToolResult::success_with_metadata(diff, metadata))
            }

            "string_diff" => {
                let text1 = input
                    .get("text_1")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'text_1' for string_diff".to_string())
                    })?;

                let text2 = input
                    .get("text_2")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'text_2' for string_diff".to_string())
                    })?;

                let diff = Self::generate_unified_diff(text1, text2, context_lines);

                let metadata = json!({
                    "identical": text1 == text2,
                    "text1_lines": text1.lines().count(),
                    "text2_lines": text2.lines().count()
                });

                Ok(ToolResult::success_with_metadata(diff, metadata))
            }

            "inline_diff" => {
                let text1 = input
                    .get("text_1")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'text_1' for inline_diff".to_string())
                    })?;

                let text2 = input
                    .get("text_2")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'text_2' for inline_diff".to_string())
                    })?;

                let diff = Self::generate_inline_diff(text1, text2);

                let metadata = json!({
                    "identical": text1 == text2
                });

                Ok(ToolResult::success_with_metadata(diff, metadata))
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
    fn test_generate_unified_diff() {
        let old = "line 1\nline 2\nline 3";
        let new = "line 1\nmodified\nline 3";

        let diff = DiffTool::generate_unified_diff(old, new, 3);
        assert!(diff.contains("--- old"));
        assert!(diff.contains("+++ new"));
        assert!(diff.contains("modified"));
    }

    #[test]
    fn test_generate_inline_diff() {
        let old = "line 1\nline 2";
        let new = "line 1\nmodified";

        let diff = DiffTool::generate_inline_diff(old, new);
        assert!(diff.contains("Line 2"));
        assert!(diff.contains("OLD: line 2"));
        assert!(diff.contains("NEW: modified"));
    }

    #[test]
    fn test_generate_inline_diff_identical() {
        let text = "line 1\nline 2";
        let diff = DiffTool::generate_inline_diff(text, text);
        // Should not contain any differences
        assert!(!diff.contains("OLD:"));
    }

    #[tokio::test]
    async fn test_string_diff() {
        let tool = DiffTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let input = json!({
            "action": "string_diff",
            "text_1": "hello\nworld",
            "text_2": "hello\nthere"
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("world"));
        assert!(result.content.contains("there"));
    }
}

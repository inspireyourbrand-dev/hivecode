use async_trait::async_trait;
use glob::glob;
use regex::Regex;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Maximum number of results to return
const MAX_RESULTS: usize = 1000;

/// Tool for text search in files using regex
pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self {
        Self
    }

    /// Search for a pattern in a file
    fn search_file(
        path: &PathBuf,
        regex: &Regex,
        context: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if regex.is_match(line) {
                let start = line_num.saturating_sub(context);
                let end = std::cmp::min(line_num + context + 1, lines.len());

                for i in start..end {
                    let prefix = if i == line_num {
                        format!("{}:", line_num + 1)
                    } else {
                        format!("{}-", i + 1)
                    };
                    matches.push(format!(
                        "{}:{}:{}",
                        path.display(),
                        prefix,
                        lines[i]
                    ));
                }
            }
        }

        Ok(matches)
    }
}

impl Default for GrepTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for text patterns in files using regex. Supports context lines and glob patterns."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "File path or directory to search in"
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern for files to search (e.g., '**/*.rs')"
                },
                "context": {
                    "type": "integer",
                    "description": "Number of context lines to show (-C/-B/-A equivalent, default: 0)"
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'pattern' field".to_string()))?;

        let context = input
            .get("context")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Compile regex
        let regex = Regex::new(pattern)
            .map_err(|e| ToolError::InvalidInput(format!("Invalid regex: {}", e)))?;

        let mut all_matches = Vec::new();

        // Determine what files to search
        let files_to_search = if let Some(glob_pattern) = input.get("glob").and_then(|v| v.as_str()) {
            // Use glob pattern
            let glob_result = glob(glob_pattern)
                .map_err(|e| ToolError::ExecutionFailed(format!("Invalid glob pattern: {}", e)))?;

            let mut paths = Vec::new();
            for entry in glob_result {
                if let Ok(path) = entry {
                    if path.is_file() {
                        paths.push(path);
                    }
                }
            }
            paths
        } else if let Some(path_str) = input.get("path").and_then(|v| v.as_str()) {
            // Search in specific file or directory
            let path = PathBuf::from(path_str);
            if path.is_file() {
                vec![path]
            } else if path.is_dir() {
                // Recursively find all files in directory
                let mut paths = Vec::new();
                Self::collect_files(&path, &mut paths)?;
                paths
            } else {
                return Err(ToolError::FileNotFound(path_str.to_string()));
            }
        } else {
            // Default to current working directory
            let mut paths = Vec::new();
            Self::collect_files(&ctx.working_directory, &mut paths)?;
            paths
        };

        // Search files
        for file_path in files_to_search {
            if all_matches.len() >= MAX_RESULTS {
                break;
            }

            // Check permission
            if let Err(_) = ctx.permission_checker.check_path(&file_path, false).await.check() {
                continue; // Skip files we don't have permission to read
            }

            match Self::search_file(&file_path, &regex, context) {
                Ok(matches) => {
                    all_matches.extend(matches);
                }
                Err(_) => {
                    // Skip files that can't be read (e.g., binary files)
                    continue;
                }
            }
        }

        let count = all_matches.len();
        let output = if all_matches.is_empty() {
            format!("No matches found for pattern '{}'", pattern)
        } else {
            all_matches.join("\n")
        };

        let metadata = json!({
            "count": count,
            "pattern": pattern,
            "context_lines": context
        });

        Ok(ToolResult::success_with_metadata(output, metadata))
    }
}

impl GrepTool {
    fn collect_files(dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<(), ToolError> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path);
                    } else if path.is_dir() {
                        // Limit recursion depth
                        let depth = path.components().count();
                        if depth < 50 {
                            let _ = Self::collect_files(&path, files);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_search_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(
            &file_path,
            "line 1\nmatch line\nline 3\nanother match\nline 5",
        )
        .unwrap();

        let regex = Regex::new("match").unwrap();
        let matches = GrepTool::search_file(&file_path, &regex, 0).unwrap();

        assert_eq!(matches.len(), 2);
        assert!(matches.iter().any(|m| m.contains("match line")));
    }

    #[test]
    fn test_search_file_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(
            &file_path,
            "line 1\nline 2\nmatch\nline 4\nline 5",
        )
        .unwrap();

        let regex = Regex::new("match").unwrap();
        let matches = GrepTool::search_file(&file_path, &regex, 1).unwrap();

        // With context=1, should show line before and after
        assert!(matches.len() > 1);
    }

    #[tokio::test]
    async fn test_grep_tool_simple() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello\nworld\nhello world").unwrap();

        let tool = GrepTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from(temp_dir.path()), Arc::new(checker));

        let input = json!({
            "pattern": "hello",
            "path": file_path.to_str()
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("hello"));
    }

    #[tokio::test]
    async fn test_grep_tool_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello\nworld").unwrap();

        let tool = GrepTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from(temp_dir.path()), Arc::new(checker));

        let input = json!({
            "pattern": "xyz",
            "path": file_path.to_str()
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("No matches"));
    }

    #[tokio::test]
    async fn test_grep_invalid_regex() {
        let tool = GrepTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from("/tmp"), Arc::new(checker));

        let input = json!({
            "pattern": "[invalid(regex"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }
}

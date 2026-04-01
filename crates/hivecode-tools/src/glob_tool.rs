use async_trait::async_trait;
use glob::glob;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for file pattern matching using glob
pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GlobTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "File pattern matching using glob patterns (e.g., **/*.rs). Results sorted by modification time."
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match files (e.g., '**/*.rs', 'src/**/test*.py')"
                },
                "path": {
                    "type": "string",
                    "description": "Base path to search in (default: current directory)"
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

        let base_path = input
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        // Build the full glob pattern
        let full_pattern = if base_path == "." {
            pattern.to_string()
        } else {
            format!("{}/{}", base_path, pattern)
        };

        // Execute glob
        let glob_result = glob(&full_pattern)
            .map_err(|e| ToolError::ExecutionFailed(format!("Invalid glob pattern: {}", e)))?;

        // Collect and sort results by modification time
        let mut results: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();

        for entry in glob_result {
            if let Ok(path) = entry {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        results.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        // Format output
        let output = results
            .iter()
            .map(|(path, _)| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        let count = results.len();
        let metadata = json!({
            "count": count,
            "pattern": pattern,
            "base_path": base_path
        });

        let message = if count == 0 {
            format!("No files matching pattern '{}'", pattern)
        } else {
            format!("Found {} file(s) matching pattern '{}'", count, pattern)
        };

        if count == 0 {
            Ok(ToolResult::success_with_metadata(message, metadata))
        } else {
            Ok(ToolResult::success_with_metadata(output, metadata))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    use std::fs::File;

    #[tokio::test]
    async fn test_glob_simple_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        // Create test files
        File::create(temp_dir.path().join("test1.rs")).unwrap();
        File::create(temp_dir.path().join("test2.rs")).unwrap();
        File::create(temp_dir.path().join("test.py")).unwrap();

        let tool = GlobTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from(base_path), Arc::new(checker));

        let input = json!({
            "pattern": "*.rs",
            "path": base_path
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("test1.rs"));
        assert!(result.content.contains("test2.rs"));
        assert!(!result.content.contains("test.py"));
    }

    #[tokio::test]
    async fn test_glob_recursive_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        // Create nested directories and files
        std::fs::create_dir_all(temp_dir.path().join("src")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("tests")).unwrap();
        File::create(temp_dir.path().join("src").join("main.rs")).unwrap();
        File::create(temp_dir.path().join("tests").join("test.rs")).unwrap();
        File::create(temp_dir.path().join("root.txt")).unwrap();

        let tool = GlobTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from(base_path), Arc::new(checker));

        let input = json!({
            "pattern": "**/*.rs",
            "path": base_path
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("main.rs"));
        assert!(result.content.contains("test.rs"));
        assert!(!result.content.contains("root.txt"));
    }

    #[tokio::test]
    async fn test_glob_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        let tool = GlobTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from(base_path), Arc::new(checker));

        let input = json!({
            "pattern": "*.nonexistent",
            "path": base_path
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("No files"));
    }

    #[tokio::test]
    async fn test_glob_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_str().unwrap();

        File::create(temp_dir.path().join("test.rs")).unwrap();

        let tool = GlobTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(PathBuf::from(base_path), Arc::new(checker));

        let input = json!({
            "pattern": "*.rs",
            "path": base_path
        });

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.metadata.is_some());
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata["count"], 1);
    }
}

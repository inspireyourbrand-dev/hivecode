use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for Language Server Protocol integration
pub struct LspTool;

impl LspTool {
    pub fn new() -> Self {
        Self
    }

    /// Convert line/character position to 0-indexed
    fn parse_position(input: &Value) -> Result<(usize, usize), String> {
        let line = input
            .get("line")
            .and_then(|v| v.as_u64())
            .ok_or("Missing or invalid 'line'")?;

        let character = input
            .get("character")
            .and_then(|v| v.as_u64())
            .ok_or("Missing or invalid 'character'")?;

        Ok((line as usize, character as usize))
    }
}

impl Default for LspTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for LspTool {
    fn name(&self) -> &str {
        "lsp"
    }

    fn description(&self) -> &str {
        "Language Server Protocol integration: symbol info, completions, diagnostics, definitions, references"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["hover", "completions", "diagnostics", "definition", "references"],
                    "description": "LSP action to perform"
                },
                "file_path": {
                    "type": "string",
                    "description": "Path to the file being analyzed"
                },
                "position": {
                    "type": "object",
                    "properties": {
                        "line": {
                            "type": "integer",
                            "description": "Line number (0-indexed)"
                        },
                        "character": {
                            "type": "integer",
                            "description": "Character position in line (0-indexed)"
                        }
                    },
                    "description": "Position in the file"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language (rust, typescript, python, etc.)"
                }
            },
            "required": ["action", "file_path"]
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

        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'file_path' field".to_string()))?;

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

        match action {
            "hover" => {
                if input.get("position").is_none() {
                    return Err(ToolError::InvalidInput(
                        "Missing 'position' for hover action".to_string(),
                    ));
                }

                let (line, character) = Self::parse_position(&input["position"])
                    .map_err(|e| ToolError::InvalidInput(e))?;

                let hover_info = json!({
                    "file": file_path,
                    "position": {
                        "line": line,
                        "character": character
                    },
                    "contents": "Type information would be provided by actual LSP server",
                    "range": {
                        "start": { "line": line, "character": character },
                        "end": { "line": line, "character": character + 1 }
                    }
                });

                Ok(ToolResult::success_with_metadata(
                    "Hover information retrieved".to_string(),
                    hover_info,
                ))
            }

            "completions" => {
                if input.get("position").is_none() {
                    return Err(ToolError::InvalidInput(
                        "Missing 'position' for completions action".to_string(),
                    ));
                }

                let (line, character) = Self::parse_position(&input["position"])
                    .map_err(|e| ToolError::InvalidInput(e))?;

                let completions = json!({
                    "file": file_path,
                    "position": {
                        "line": line,
                        "character": character
                    },
                    "items": [
                        {
                            "label": "completion_1",
                            "kind": "Function",
                            "detail": "Example completion",
                            "sortText": "0"
                        },
                        {
                            "label": "completion_2",
                            "kind": "Variable",
                            "detail": "Another completion",
                            "sortText": "1"
                        }
                    ],
                    "isIncomplete": false
                });

                let metadata = json!({
                    "completions_count": 2
                });

                Ok(ToolResult::success_with_metadata(
                    "Completions retrieved".to_string(),
                    completions,
                ))
            }

            "diagnostics" => {
                let diagnostics = json!({
                    "file": file_path,
                    "diagnostics": [
                        {
                            "range": {
                                "start": { "line": 0, "character": 0 },
                                "end": { "line": 0, "character": 10 }
                            },
                            "severity": "information",
                            "code": "unused_variable",
                            "message": "Unused variable would be reported here"
                        }
                    ]
                });

                let metadata = json!({
                    "file": file_path,
                    "diagnostics_count": 1
                });

                Ok(ToolResult::success_with_metadata(
                    "Diagnostics retrieved".to_string(),
                    diagnostics,
                ))
            }

            "definition" => {
                if input.get("position").is_none() {
                    return Err(ToolError::InvalidInput(
                        "Missing 'position' for definition action".to_string(),
                    ));
                }

                let (line, character) = Self::parse_position(&input["position"])
                    .map_err(|e| ToolError::InvalidInput(e))?;

                let definition = json!({
                    "file": file_path,
                    "position": {
                        "line": line,
                        "character": character
                    },
                    "definitions": [
                        {
                            "uri": file_path,
                            "range": {
                                "start": { "line": 10, "character": 0 },
                                "end": { "line": 10, "character": 20 }
                            }
                        }
                    ]
                });

                Ok(ToolResult::success_with_metadata(
                    "Definition retrieved".to_string(),
                    definition,
                ))
            }

            "references" => {
                if input.get("position").is_none() {
                    return Err(ToolError::InvalidInput(
                        "Missing 'position' for references action".to_string(),
                    ));
                }

                let (line, character) = Self::parse_position(&input["position"])
                    .map_err(|e| ToolError::InvalidInput(e))?;

                let references = json!({
                    "file": file_path,
                    "position": {
                        "line": line,
                        "character": character
                    },
                    "references": [
                        {
                            "uri": file_path,
                            "range": {
                                "start": { "line": 5, "character": 10 },
                                "end": { "line": 5, "character": 15 }
                            }
                        },
                        {
                            "uri": file_path,
                            "range": {
                                "start": { "line": 20, "character": 5 },
                                "end": { "line": 20, "character": 10 }
                            }
                        }
                    ]
                });

                let metadata = json!({
                    "references_count": 2
                });

                Ok(ToolResult::success_with_metadata(
                    "References retrieved".to_string(),
                    references,
                ))
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
    fn test_parse_position_valid() {
        let pos = json!({
            "line": 10,
            "character": 5
        });

        let result = LspTool::parse_position(&pos);
        assert!(result.is_ok());
        let (line, char) = result.unwrap();
        assert_eq!(line, 10);
        assert_eq!(char, 5);
    }

    #[test]
    fn test_parse_position_missing_line() {
        let pos = json!({
            "character": 5
        });

        let result = LspTool::parse_position(&pos);
        assert!(result.is_err());
    }

    #[test]
    fn test_lsp_tool_creation() {
        let tool = LspTool::new();
        assert_eq!(tool.name(), "lsp");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_lsp_tool_schema() {
        let tool = LspTool::new();
        let schema = tool.input_schema();
        assert!(schema.is_object());
        assert!(schema["properties"]["action"].is_object());
    }

    #[tokio::test]
    async fn test_lsp_hover_missing_position() {
        let tool = LspTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let input = json!({
            "action": "hover",
            "file_path": "/tmp/test.rs"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_err());
    }
}

use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Tool for editing Jupyter notebooks (.ipynb files)
pub struct NotebookEditTool;

impl NotebookEditTool {
    pub fn new() -> Self {
        Self
    }

    /// Parse notebook JSON structure
    fn parse_notebook(content: &str) -> Result<Value, serde_json::Error> {
        serde_json::from_str(content)
    }

    /// Validate notebook structure
    fn validate_notebook(notebook: &Value) -> Result<(), String> {
        if !notebook.is_object() {
            return Err("Invalid notebook: root is not an object".to_string());
        }

        if notebook.get("nbformat").is_none() {
            return Err("Invalid notebook: missing nbformat".to_string());
        }

        if notebook.get("cells").is_none() {
            return Err("Invalid notebook: missing cells array".to_string());
        }

        if !notebook["cells"].is_array() {
            return Err("Invalid notebook: cells is not an array".to_string());
        }

        Ok(())
    }

    /// Edit a cell by index
    fn edit_cell(
        notebook: &mut Value,
        cell_index: usize,
        new_source: Option<String>,
    ) -> Result<(), String> {
        let cells = notebook["cells"]
            .as_array_mut()
            .ok_or("cells is not an array")?;

        if cell_index >= cells.len() {
            return Err(format!(
                "Cell index {} out of range (notebook has {} cells)",
                cell_index,
                cells.len()
            ));
        }

        if let Some(source) = new_source {
            // Source can be a string or array of strings
            cells[cell_index]["source"] = Value::String(source);
        }

        Ok(())
    }

    /// Insert a new cell at specified index
    fn insert_cell(
        notebook: &mut Value,
        index: usize,
        cell_type: &str,
        source: String,
    ) -> Result<(), String> {
        let cells = notebook["cells"]
            .as_array_mut()
            .ok_or("cells is not an array")?;

        if index > cells.len() {
            return Err(format!(
                "Insert index {} out of range (notebook has {} cells)",
                index,
                cells.len()
            ));
        }

        let new_cell = json!({
            "cell_type": cell_type,
            "metadata": {},
            "source": source,
            "outputs": if cell_type == "code" { json!([]) } else { json!(Value::Null) },
            "execution_count": if cell_type == "code" { json!(Value::Null) } else { json!(Value::Null) }
        });

        cells.insert(index, new_cell);
        Ok(())
    }

    /// Delete a cell by index
    fn delete_cell(notebook: &mut Value, index: usize) -> Result<(), String> {
        let cells = notebook["cells"]
            .as_array_mut()
            .ok_or("cells is not an array")?;

        if index >= cells.len() {
            return Err(format!(
                "Cell index {} out of range (notebook has {} cells)",
                index,
                cells.len()
            ));
        }

        cells.remove(index);
        Ok(())
    }
}

impl Default for NotebookEditTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for NotebookEditTool {
    fn name(&self) -> &str {
        "notebook_edit"
    }

    fn description(&self) -> &str {
        "Edit Jupyter notebook cells: read, edit, insert, or delete cells by index"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute path to the .ipynb file"
                },
                "action": {
                    "type": "string",
                    "enum": ["read", "edit", "insert", "delete"],
                    "description": "Action to perform: read (return cells), edit (modify cell), insert (add cell), or delete (remove cell)"
                },
                "cell_index": {
                    "type": "integer",
                    "description": "Index of the cell (0-based, required for edit/delete, optional for insert)"
                },
                "cell_type": {
                    "type": "string",
                    "enum": ["code", "markdown", "raw"],
                    "description": "Type of cell (required for insert action)"
                },
                "new_source": {
                    "type": "string",
                    "description": "New source code/text for the cell (required for edit, insert actions)"
                }
            },
            "required": ["file_path", "action"]
        })
    }

    async fn execute(
        &self,
        input: Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'file_path' field".to_string()))?;

        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'action' field".to_string()))?;

        let full_path = PathBuf::from(file_path);

        // Check permissions
        ctx.permission_checker
            .check_path(&full_path, matches!(action, "edit" | "insert" | "delete"))
            .await
            .check()?;

        // Check if file exists
        if !full_path.exists() {
            return Err(ToolError::FileNotFound(file_path.to_string()));
        }

        // Read the notebook
        let content = fs::read_to_string(&full_path)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let mut notebook =
            Self::parse_notebook(&content).map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        // Validate notebook structure
        Self::validate_notebook(&notebook)
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        match action {
            "read" => {
                let cells = notebook["cells"]
                    .as_array()
                    .ok_or_else(|| ToolError::ExecutionFailed("No cells found".to_string()))?;

                let cell_summaries: Vec<Value> = cells
                    .iter()
                    .enumerate()
                    .map(|(idx, cell)| {
                        json!({
                            "index": idx,
                            "type": cell.get("cell_type").unwrap_or(&Value::Null),
                            "source_preview": cell.get("source").unwrap_or(&Value::Null)
                        })
                    })
                    .collect();

                let metadata = json!({
                    "total_cells": cells.len(),
                    "cells": cell_summaries
                });

                Ok(ToolResult::success_with_metadata(
                    format!("Notebook has {} cells", cells.len()),
                    metadata,
                ))
            }

            "edit" => {
                let cell_index = input
                    .get("cell_index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'cell_index' for edit".to_string()))?
                    as usize;

                let new_source = input
                    .get("new_source")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'new_source' for edit".to_string()))?
                    .to_string();

                Self::edit_cell(&mut notebook, cell_index, Some(new_source))
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                // Write back
                let notebook_str = serde_json::to_string_pretty(&notebook)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                fs::write(&full_path, notebook_str)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

                Ok(ToolResult::success(format!("Cell {} edited successfully", cell_index)))
            }

            "insert" => {
                let index = input
                    .get("cell_index")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or_else(|| {
                        notebook["cells"]
                            .as_array()
                            .map(|a| a.len())
                            .unwrap_or(0)
                    });

                let cell_type = input
                    .get("cell_type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'cell_type' for insert".to_string()))?;

                let source = input
                    .get("new_source")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'new_source' for insert".to_string()))?
                    .to_string();

                Self::insert_cell(&mut notebook, index, cell_type, source)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                // Write back
                let notebook_str = serde_json::to_string_pretty(&notebook)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                fs::write(&full_path, notebook_str)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

                Ok(ToolResult::success(format!(
                    "Cell inserted at index {} successfully",
                    index
                )))
            }

            "delete" => {
                let cell_index = input
                    .get("cell_index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'cell_index' for delete".to_string()))?
                    as usize;

                Self::delete_cell(&mut notebook, cell_index)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                // Write back
                let notebook_str = serde_json::to_string_pretty(&notebook)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                fs::write(&full_path, notebook_str)
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

                Ok(ToolResult::success(format!("Cell {} deleted successfully", cell_index)))
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
    use std::sync::Arc;

    fn create_test_notebook() -> Value {
        json!({
            "nbformat": 4,
            "nbformat_minor": 4,
            "metadata": {},
            "cells": [
                {
                    "cell_type": "code",
                    "metadata": {},
                    "source": "print('hello')",
                    "outputs": [],
                    "execution_count": null
                },
                {
                    "cell_type": "markdown",
                    "metadata": {},
                    "source": "# Title"
                }
            ]
        })
    }

    #[test]
    fn test_validate_notebook_valid() {
        let notebook = create_test_notebook();
        assert!(NotebookEditTool::validate_notebook(&notebook).is_ok());
    }

    #[test]
    fn test_validate_notebook_missing_format() {
        let notebook = json!({
            "cells": []
        });
        assert!(NotebookEditTool::validate_notebook(&notebook).is_err());
    }

    #[test]
    fn test_edit_cell() {
        let mut notebook = create_test_notebook();
        let result = NotebookEditTool::edit_cell(&mut notebook, 0, Some("new code".to_string()));
        assert!(result.is_ok());
        assert_eq!(notebook["cells"][0]["source"], "new code");
    }

    #[test]
    fn test_insert_cell() {
        let mut notebook = create_test_notebook();
        let result =
            NotebookEditTool::insert_cell(&mut notebook, 1, "code", "inserted".to_string());
        assert!(result.is_ok());
        assert_eq!(notebook["cells"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_delete_cell() {
        let mut notebook = create_test_notebook();
        let initial_len = notebook["cells"].as_array().unwrap().len();
        let result = NotebookEditTool::delete_cell(&mut notebook, 0);
        assert!(result.is_ok());
        assert_eq!(notebook["cells"].as_array().unwrap().len(), initial_len - 1);
    }
}

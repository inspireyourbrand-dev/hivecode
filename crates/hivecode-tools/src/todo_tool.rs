use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Utc;

use crate::error::ToolError;
use crate::traits::{Tool, ToolContext, ToolResult};

/// Global todo storage (in real app, would be session-based)
static TODO_STORAGE: std::sync::OnceLock<Arc<Mutex<HashMap<String, Value>>>> =
    std::sync::OnceLock::new();

fn get_todo_store() -> Arc<Mutex<HashMap<String, Value>>> {
    TODO_STORAGE
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

/// Tool for managing todo lists and tasks
pub struct TodoTool;

impl TodoTool {
    pub fn new() -> Self {
        Self
    }

    /// Create a new todo item
    fn create_todo(id: &str, content: &str, status: &str) -> Value {
        json!({
            "id": id,
            "content": content,
            "status": status,
            "created_at": Utc::now().to_rfc3339()
        })
    }

    /// Format todos for display
    fn format_todos(todos: &[Value]) -> String {
        if todos.is_empty() {
            return "No todos found.".to_string();
        }

        let mut output = String::new();
        for (idx, todo) in todos.iter().enumerate() {
            let id = todo["id"].as_str().unwrap_or("unknown");
            let status = todo["status"].as_str().unwrap_or("unknown");
            let content = todo["content"].as_str().unwrap_or("unknown");

            output.push_str(&format!("{}. [{}] {} (id: {})\n", idx + 1, status, content, id));
        }
        output
    }
}

impl Default for TodoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for TodoTool {
    fn name(&self) -> &str {
        "todo"
    }

    fn description(&self) -> &str {
        "Manage todo lists and task tracking: create, update, delete, list todos with status tracking"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["create", "update", "delete", "list", "clear"],
                    "description": "Action to perform on todos"
                },
                "id": {
                    "type": "string",
                    "description": "Unique identifier for the todo (required for update/delete)"
                },
                "content": {
                    "type": "string",
                    "description": "Content/description of the todo (required for create/update)"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed"],
                    "description": "Status of the todo (default: pending)"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(
        &self,
        input: Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let action = input
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("Missing 'action' field".to_string()))?;

        let store = get_todo_store();
        let mut todos = store.lock().unwrap();

        match action {
            "create" => {
                let id = input
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'id' for create".to_string()))?;

                let content = input
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ToolError::InvalidInput("Missing 'content' for create".to_string())
                    })?;

                let status = input
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("pending");

                if todos.contains_key(id) {
                    return Err(ToolError::ExecutionFailed(format!(
                        "Todo with id '{}' already exists",
                        id
                    )));
                }

                let todo = Self::create_todo(id, content, status);
                todos.insert(id.to_string(), todo);

                Ok(ToolResult::success(format!("Todo '{}' created", id)))
            }

            "update" => {
                let id = input
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'id' for update".to_string()))?;

                let todo = todos.get_mut(id).ok_or_else(|| {
                    ToolError::ExecutionFailed(format!("Todo '{}' not found", id))
                })?;

                if let Some(content) = input.get("content").and_then(|v| v.as_str()) {
                    todo["content"] = json!(content);
                }

                if let Some(status) = input.get("status").and_then(|v| v.as_str()) {
                    if !["pending", "in_progress", "completed"].contains(&status) {
                        return Err(ToolError::InvalidInput(format!(
                            "Invalid status: {}",
                            status
                        )));
                    }
                    todo["status"] = json!(status);
                }

                Ok(ToolResult::success(format!("Todo '{}' updated", id)))
            }

            "delete" => {
                let id = input
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidInput("Missing 'id' for delete".to_string()))?;

                if todos.remove(id).is_none() {
                    return Err(ToolError::ExecutionFailed(format!(
                        "Todo '{}' not found",
                        id
                    )));
                }

                Ok(ToolResult::success(format!("Todo '{}' deleted", id)))
            }

            "list" => {
                let todo_list: Vec<Value> = todos.values().cloned().collect();
                let formatted = Self::format_todos(&todo_list);
                let metadata = json!({
                    "total": todo_list.len(),
                    "todos": todo_list
                });

                Ok(ToolResult::success_with_metadata(formatted, metadata))
            }

            "clear" => {
                let count = todos.len();
                todos.clear();
                Ok(ToolResult::success(format!(
                    "Cleared {} todos",
                    count
                )))
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
    fn test_create_todo() {
        let todo = TodoTool::create_todo("task1", "Do something", "pending");
        assert_eq!(todo["id"], "task1");
        assert_eq!(todo["content"], "Do something");
        assert_eq!(todo["status"], "pending");
    }

    #[test]
    fn test_format_todos_empty() {
        let todos = vec![];
        let formatted = TodoTool::format_todos(&todos);
        assert_eq!(formatted, "No todos found.");
    }

    #[test]
    fn test_format_todos_with_items() {
        let todos = vec![
            TodoTool::create_todo("t1", "Task 1", "pending"),
            TodoTool::create_todo("t2", "Task 2", "completed"),
        ];
        let formatted = TodoTool::format_todos(&todos);
        assert!(formatted.contains("Task 1"));
        assert!(formatted.contains("Task 2"));
        assert!(formatted.contains("pending"));
        assert!(formatted.contains("completed"));
    }

    #[tokio::test]
    async fn test_todo_create_and_list() {
        let tool = TodoTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        // Clear any existing todos
        let clear_input = json!({"action": "clear"});
        let _ = tool.execute(clear_input, &ctx).await;

        // Create a todo
        let create_input = json!({
            "action": "create",
            "id": "test-task",
            "content": "Test task content",
            "status": "pending"
        });

        let result = tool.execute(create_input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // List todos
        let list_input = json!({"action": "list"});
        let result = tool.execute(list_input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Test task content"));
    }

    #[tokio::test]
    async fn test_todo_update() {
        let tool = TodoTool::new();
        let checker = hivecode_security::create_default_permission_checker();
        let ctx = ToolContext::new(std::path::PathBuf::from("/tmp"), std::sync::Arc::new(checker));

        let clear_input = json!({"action": "clear"});
        let _ = tool.execute(clear_input, &ctx).await;

        // Create
        let create_input = json!({
            "action": "create",
            "id": "test-task",
            "content": "Original",
            "status": "pending"
        });
        tool.execute(create_input, &ctx).await.unwrap();

        // Update
        let update_input = json!({
            "action": "update",
            "id": "test-task",
            "status": "completed"
        });

        let result = tool.execute(update_input, &ctx).await.unwrap();
        assert!(!result.is_error);

        // Verify
        let list_input = json!({"action": "list"});
        let result = tool.execute(list_input, &ctx).await.unwrap();
        assert!(result.content.contains("completed"));
    }
}

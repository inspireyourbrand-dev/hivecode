pub mod traits;
pub mod registry;
pub mod bash;
pub mod file_read;
pub mod file_write;
pub mod file_edit;
pub mod glob_tool;
pub mod grep;
pub mod web_fetch;
pub mod agent;
pub mod error;
pub mod notebook_edit;
pub mod todo_tool;
pub mod tool_search;
pub mod config_tool;
pub mod diff_tool;
pub mod git_tool;
pub mod lsp_tool;
pub mod parallel;

pub use traits::{Tool, ToolContext, ToolResult};
pub use registry::ToolRegistry;
pub use error::ToolError;
pub use parallel::{
    ParallelExecutor, ToolCall, ToolCallResult, ParallelExecutionReport, ExecutionMode,
};

use std::sync::Arc;
use hivecode_security::PermissionChecker;

/// Create a default tool registry with all standard tools
pub fn create_default_registry(permission_checker: Arc<dyn PermissionChecker>) -> ToolRegistry {
    let mut registry = ToolRegistry::new(permission_checker.clone());

    registry.register(Arc::new(bash::BashTool::new()));
    registry.register(Arc::new(file_read::FileReadTool::new()));
    registry.register(Arc::new(file_write::FileWriteTool::new()));
    registry.register(Arc::new(file_edit::FileEditTool::new()));
    registry.register(Arc::new(glob_tool::GlobTool::new()));
    registry.register(Arc::new(grep::GrepTool::new()));
    registry.register(Arc::new(web_fetch::WebFetchTool::new()));
    registry.register(Arc::new(agent::AgentTool::new()));
    registry.register(Arc::new(notebook_edit::NotebookEditTool::new()));
    registry.register(Arc::new(todo_tool::TodoTool::new()));
    registry.register(Arc::new(tool_search::ToolSearchTool::new()));
    registry.register(Arc::new(config_tool::ConfigTool::new()));
    registry.register(Arc::new(diff_tool::DiffTool::new()));
    registry.register(Arc::new(git_tool::GitTool::new()));
    registry.register(Arc::new(lsp_tool::LspTool::new()));

    registry
}

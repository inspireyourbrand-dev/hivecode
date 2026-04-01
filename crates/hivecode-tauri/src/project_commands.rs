//! Tauri IPC commands for project instructions management
//!
//! These commands handle loading, saving, and validating HIVECODE.md files
//! that define project context and AI constraints.

use std::sync::Arc;
use tauri::State;
use tracing::{debug, info, warn};

use crate::state::TauriAppState;

/// Load project instructions from disk
///
/// Reads the HIVECODE.md file from the specified project path.
#[tauri::command]
pub async fn load_project_instructions(
    state: State<'_, TauriAppState>,
    project_path: String,
) -> Result<String, String> {
    debug!("load_project_instructions command received: project_path={}", project_path);

    if project_path.is_empty() {
        return Err("project_path cannot be empty".to_string());
    }

    // Placeholder: In production, would read HIVECODE.md from filesystem
    // This would need proper error handling for file not found, permission errors, etc.

    let content = String::new();

    info!("Loaded project instructions from: {}", project_path);
    Ok(content)
}

/// Save project instructions to disk
///
/// Writes the HIVECODE.md file to the specified project path.
#[tauri::command]
pub async fn save_project_instructions(
    state: State<'_, TauriAppState>,
    project_path: String,
    content: String,
) -> Result<(), String> {
    debug!("save_project_instructions command received: project_path={}", project_path);

    if project_path.is_empty() {
        return Err("project_path cannot be empty".to_string());
    }

    if content.trim().is_empty() {
        return Err("Instructions content cannot be empty".to_string());
    }

    // Placeholder: In production, would write HIVECODE.md to filesystem
    // with proper validation and error handling

    info!("Saved project instructions to: {}", project_path);
    Ok(())
}

/// Get a template for new HIVECODE.md files
///
/// Returns a starter template with common sections and best practices.
#[tauri::command]
pub async fn get_project_instructions_template() -> Result<String, String> {
    debug!("get_project_instructions_template command received");

    let template = r#"# HiveCode Instructions

## Instructions

Describe your project context, goals, and any special requirements for the AI assistant:

- **Project Type**: [e.g., Web App, CLI Tool, Library]
- **Tech Stack**: [e.g., React, Node.js, Rust]
- **Key Constraints**: [e.g., performance requirements, security concerns]
- **Coding Standards**: [e.g., naming conventions, style guides]

## Tools

List the tools and capabilities the AI should use:

- **bash**: Execute shell commands, run scripts
- **file_operations**: Read and write files, manage directories
- **code_generation**: Create or modify code files
- **testing**: Run test suites and provide feedback

## Files

Specify which files the AI should prioritize and which to avoid:

- **Include**: `src/**`, `tests/**`, `*.md`
- **Exclude**: `node_modules/**`, `.git/**`, `dist/**`, `*.lock`
- **Read-only**: `package.json`, `tsconfig.json`

## Model Preferences

Specify preferred AI models and behaviors:

- **Primary Model**: Claude 3.5 Sonnet (recommended for coding)
- **Fast Model**: Claude Haiku (for quick tasks)
- **Extended Thinking**: Enable for complex architectural decisions
- **Context Strategy**: Keep recent history, summarize older messages

## Additional Notes

- Assume the AI has no prior knowledge of proprietary systems
- Explain complex decisions and ask for clarification when needed
- Prioritize code quality and maintainability over quick solutions
"#;

    info!("Generated project instructions template");
    Ok(template.to_string())
}

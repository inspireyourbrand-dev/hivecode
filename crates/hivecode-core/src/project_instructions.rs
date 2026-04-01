//! HIVECODE.md project instructions system
//!
//! Loads per-project configuration files (HIVECODE.md) that customize behavior,
//! persona, allowed/denied tools, file restrictions, and standing instructions.
//! Similar to CLAUDE.md, this is a markdown file that projects can define to
//! configure HiveCode's behavior for their specific needs.

use crate::error::{HiveCodeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Complete project instructions loaded from HIVECODE.md
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectInstructions {
    /// Project name
    pub project_name: Option<String>,
    /// Brief description of the project
    pub description: Option<String>,
    /// Standing instructions for the LLM to follow
    pub instructions: Vec<String>,
    /// Custom persona/role for the assistant in this project
    pub persona: Option<String>,
    /// Whitelist of allowed tools (None means all allowed)
    pub allowed_tools: Option<Vec<String>>,
    /// Blacklist of denied tools (takes precedence)
    pub denied_tools: Option<Vec<String>>,
    /// File access restrictions
    pub file_restrictions: FileRestrictions,
    /// Project-specific hooks
    pub hooks: Vec<ProjectHook>,
    /// Files to always include in context
    pub context_files: Vec<String>,
    /// Project environment variables
    pub environment: HashMap<String, String>,
    /// Model preferences for this project
    pub model_preferences: ModelPreferences,
    /// The raw markdown content that was parsed
    pub raw_content: String,
    /// Path to the source HIVECODE.md file
    pub source_path: PathBuf,
    /// When these instructions were loaded
    pub loaded_at: DateTime<Utc>,
}

impl ProjectInstructions {
    /// Build a human-readable summary of these instructions
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if let Some(name) = &self.project_name {
            parts.push(format!("Project: {}", name));
        }

        if !self.instructions.is_empty() {
            parts.push(format!("Instructions: {} items", self.instructions.len()));
        }

        if let Some(tools) = &self.allowed_tools {
            parts.push(format!("Allowed tools: {}", tools.len()));
        }

        if let Some(tools) = &self.denied_tools {
            parts.push(format!("Denied tools: {}", tools.len()));
        }

        parts.join(" | ")
    }
}

/// File access restrictions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileRestrictions {
    /// Glob patterns for read-only files (can be read, not modified)
    pub read_only: Vec<String>,
    /// Glob patterns for files that cannot be accessed at all
    pub no_access: Vec<String>,
    /// Glob patterns for files that should always be included in context
    pub auto_include: Vec<String>,
}

/// A hook defined in the project instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHook {
    /// Tool name or pattern that triggers this hook
    pub trigger: String,
    /// Command or action to execute
    pub action: String,
    /// When to execute: "before" or "after"
    pub when: String,
}

/// Model preferences for a project
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelPreferences {
    /// Preferred model to use for this project
    pub preferred_model: Option<String>,
    /// Fallback model if preferred is unavailable
    pub fallback_model: Option<String>,
    /// Maximum tokens per request
    pub max_tokens_per_request: Option<u32>,
    /// Temperature for response generation
    pub temperature: Option<f32>,
}

/// Path access check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathAccessResult {
    /// Path is accessible and can be read/written
    Allowed,
    /// Path is accessible but read-only
    ReadOnly,
    /// Path cannot be accessed
    Denied(String),
}

/// Loads and manages HIVECODE.md project instruction files
pub struct ProjectInstructionsLoader;

impl ProjectInstructionsLoader {
    /// Search for HIVECODE.md in the project directory and parent directories
    pub fn find_instructions_file(project_dir: &Path) -> Option<PathBuf> {
        let mut current = project_dir.to_path_buf();

        loop {
            let candidate = current.join("HIVECODE.md");
            if candidate.exists() {
                debug!("Found HIVECODE.md at: {}", candidate.display());
                return Some(candidate);
            }

            if !current.pop() {
                // Reached filesystem root without finding HIVECODE.md
                break;
            }
        }

        None
    }

    /// Load and parse a HIVECODE.md file
    pub async fn load(path: &Path) -> Result<ProjectInstructions> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read HIVECODE.md: {}", e)))?;

        info!("Loaded HIVECODE.md from: {}", path.display());

        let mut instructions = Self::parse_markdown(&content);
        instructions.source_path = path.to_path_buf();
        instructions.raw_content = content;
        instructions.loaded_at = Utc::now();

        Ok(instructions)
    }

    /// Load from a project directory (auto-discovers the file)
    pub async fn load_from_project(project_dir: &Path) -> Result<Option<ProjectInstructions>> {
        match Self::find_instructions_file(project_dir) {
            Some(path) => {
                let instructions = Self::load(&path).await?;
                Ok(Some(instructions))
            }
            None => {
                debug!("No HIVECODE.md found in project");
                Ok(None)
            }
        }
    }

    /// Parse markdown content into structured instructions
    fn parse_markdown(content: &str) -> ProjectInstructions {
        let mut instructions = ProjectInstructions::default();

        let mut current_section = String::new();
        let mut current_items = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Section headers
            if let Some(header) = trimmed.strip_prefix("# ") {
                instructions.project_name = Some(header.to_string());
            } else if let Some(_desc) = trimmed.strip_prefix("## Description") {
                current_section = "description".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Instructions") {
                current_section = "instructions".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Persona") {
                current_section = "persona".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Allowed Tools") {
                current_section = "allowed_tools".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Denied Tools") {
                current_section = "denied_tools".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## File Restrictions") {
                current_section = "file_restrictions".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Hooks") {
                current_section = "hooks".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Context Files") {
                current_section = "context_files".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Model Preferences") {
                current_section = "model_preferences".to_string();
            } else if let Some(_) = trimmed.strip_prefix("## Environment") {
                current_section = "environment".to_string();
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                // List items
                let item = trimmed.strip_prefix("- ").or(trimmed.strip_prefix("* ")).unwrap_or(trimmed);
                current_items.push(item.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with("#") {
                // Regular text content
                match current_section.as_str() {
                    "description" => {
                        if instructions.description.is_none() {
                            instructions.description = Some(trimmed.to_string());
                        } else if let Some(ref mut desc) = instructions.description {
                            desc.push(' ');
                            desc.push_str(trimmed);
                        }
                    }
                    "persona" => {
                        if instructions.persona.is_none() {
                            instructions.persona = Some(trimmed.to_string());
                        } else if let Some(ref mut persona) = instructions.persona {
                            persona.push(' ');
                            persona.push_str(trimmed);
                        }
                    }
                    _ => {}
                }
            } else if trimmed.is_empty() && !current_items.is_empty() {
                // Process accumulated items when we hit a blank line
                Self::process_section_items(&mut instructions, &current_section, &current_items);
                current_items.clear();
            }
        }

        // Process any remaining items
        if !current_items.is_empty() {
            Self::process_section_items(&mut instructions, &current_section, &current_items);
        }

        instructions
    }

    fn process_section_items(
        instructions: &mut ProjectInstructions,
        section: &str,
        items: &[String],
    ) {
        match section {
            "instructions" => {
                instructions.instructions.extend(items.iter().cloned());
            }
            "allowed_tools" => {
                instructions.allowed_tools = Some(items.iter().cloned().collect());
            }
            "denied_tools" => {
                instructions.denied_tools = Some(items.iter().cloned().collect());
            }
            "context_files" => {
                instructions.context_files.extend(items.iter().cloned());
            }
            "file_restrictions" => {
                for item in items {
                    if let Some(pattern) = item.strip_prefix("read-only: ") {
                        instructions.file_restrictions.read_only.push(pattern.to_string());
                    } else if let Some(pattern) = item.strip_prefix("no-access: ") {
                        instructions.file_restrictions.no_access.push(pattern.to_string());
                    } else if let Some(pattern) = item.strip_prefix("auto-include: ") {
                        instructions.file_restrictions.auto_include.push(pattern.to_string());
                    }
                }
            }
            "hooks" => {
                for item in items {
                    if let Some(hook_def) = item.strip_prefix("on ") {
                        // Parse "on bash: before -> lint"
                        if let Some((trigger_part, action_part)) = hook_def.split_once("->") {
                            if let Some((trigger, when)) = trigger_part.split_once(':') {
                                instructions.hooks.push(ProjectHook {
                                    trigger: trigger.trim().to_string(),
                                    action: action_part.trim().to_string(),
                                    when: when.trim().to_string(),
                                });
                            }
                        }
                    }
                }
            }
            "model_preferences" => {
                for item in items {
                    if let Some(model) = item.strip_prefix("preferred: ") {
                        instructions.model_preferences.preferred_model = Some(model.to_string());
                    } else if let Some(model) = item.strip_prefix("fallback: ") {
                        instructions.model_preferences.fallback_model = Some(model.to_string());
                    } else if let Some(tokens) = item.strip_prefix("max_tokens: ") {
                        if let Ok(n) = tokens.parse::<u32>() {
                            instructions.model_preferences.max_tokens_per_request = Some(n);
                        }
                    } else if let Some(temp) = item.strip_prefix("temperature: ") {
                        if let Ok(t) = temp.parse::<f32>() {
                            instructions.model_preferences.temperature = Some(t);
                        }
                    }
                }
            }
            "environment" => {
                for item in items {
                    if let Some((key, value)) = item.split_once('=') {
                        instructions
                            .environment
                            .insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    /// Build a system prompt addition from project instructions
    pub fn build_system_context(instructions: &ProjectInstructions) -> String {
        let mut context = String::new();

        if !instructions.instructions.is_empty() {
            context.push_str("## Project Instructions\n");
            for instruction in &instructions.instructions {
                context.push_str("- ");
                context.push_str(instruction);
                context.push('\n');
            }
            context.push('\n');
        }

        if let Some(persona) = &instructions.persona {
            context.push_str("## Role\n");
            context.push_str(persona);
            context.push_str("\n\n");
        }

        if let Some(tools) = &instructions.allowed_tools {
            if !tools.is_empty() {
                context.push_str("## Allowed Tools\n");
                context.push_str("You may only use these tools: ");
                context.push_str(&tools.join(", "));
                context.push_str("\n\n");
            }
        }

        if let Some(tools) = &instructions.denied_tools {
            if !tools.is_empty() {
                context.push_str("## Denied Tools\n");
                context.push_str("You must NOT use these tools: ");
                context.push_str(&tools.join(", "));
                context.push_str("\n\n");
            }
        }

        context
    }

    /// Check if a tool is allowed by project instructions
    pub fn is_tool_allowed(instructions: &ProjectInstructions, tool_name: &str) -> bool {
        // If tool is explicitly denied, it's not allowed
        if let Some(denied) = &instructions.denied_tools {
            if denied.iter().any(|d| d == tool_name) {
                return false;
            }
        }

        // If there's an allow list, tool must be in it
        if let Some(allowed) = &instructions.allowed_tools {
            return allowed.iter().any(|a| a == tool_name);
        }

        // Otherwise, tool is allowed
        true
    }

    /// Check if a file path is accessible
    pub fn is_path_accessible(instructions: &ProjectInstructions, path: &Path) -> PathAccessResult {
        let path_str = path.to_string_lossy();

        // Check no-access patterns first
        for pattern in &instructions.file_restrictions.no_access {
            if Self::glob_matches(pattern, &path_str) {
                return PathAccessResult::Denied(format!("Path matches no-access pattern: {}", pattern));
            }
        }

        // Check read-only patterns
        for pattern in &instructions.file_restrictions.read_only {
            if Self::glob_matches(pattern, &path_str) {
                return PathAccessResult::ReadOnly;
            }
        }

        PathAccessResult::Allowed
    }

    /// Check if a file should be auto-included in context
    pub fn should_auto_include(instructions: &ProjectInstructions, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        instructions
            .file_restrictions
            .auto_include
            .iter()
            .any(|pattern| Self::glob_matches(pattern, &path_str))
    }

    // Helper: simple glob matching (simplified version)
    fn glob_matches(pattern: &str, path: &str) -> bool {
        // Very basic implementation - in production would use glob crate
        if pattern == "*" {
            return true;
        }
        if pattern.contains("*") {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            let mut pos = 0;
            for (i, part) in pattern_parts.iter().enumerate() {
                if i == 0 && !part.is_empty() {
                    if !path.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else if i == pattern_parts.len() - 1 && !part.is_empty() {
                    if !path.ends_with(part) {
                        return false;
                    }
                } else if !part.is_empty() {
                    if let Some(new_pos) = path[pos..].find(part) {
                        pos += new_pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            path == pattern || path.ends_with(&format!("/{}", pattern))
        }
    }

    /// Watch for changes to the instructions file and reload when modified
    pub async fn watch_for_changes(
        path: &Path,
    ) -> Result<tokio::sync::watch::Receiver<ProjectInstructions>> {
        let (tx, rx) = tokio::sync::watch::channel(Self::load(path).await?);
        let path = path.to_path_buf();

        tokio::spawn(async move {
            let mut last_modified = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if Some(modified) != last_modified {
                            last_modified = Some(modified);
                            if let Ok(updated) = Self::load(&path).await {
                                let _ = tx.send(updated);
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_project_instructions_default() {
        let instr = ProjectInstructions::default();
        assert!(instr.project_name.is_none());
        assert!(instr.instructions.is_empty());
    }

    #[test]
    fn test_project_instructions_summary() {
        let mut instr = ProjectInstructions::default();
        instr.project_name = Some("Test Project".to_string());
        instr.instructions = vec!["Instruction 1".to_string(), "Instruction 2".to_string()];

        let summary = instr.summary();
        assert!(summary.contains("Test Project"));
        assert!(summary.contains("Instructions: 2 items"));
    }

    #[test]
    fn test_parse_markdown_simple() {
        let content = r#"# My Project
## Description
A test project

## Instructions
- Follow best practices
- Write tests
"#;

        let instr = ProjectInstructionsLoader::parse_markdown(content);
        assert_eq!(instr.project_name, Some("My Project".to_string()));
        assert_eq!(instr.instructions.len(), 2);
    }

    #[test]
    fn test_parse_markdown_tools() {
        let content = r#"# Project
## Allowed Tools
- bash
- file_edit

## Denied Tools
- delete_file
"#;

        let instr = ProjectInstructionsLoader::parse_markdown(content);
        assert_eq!(instr.allowed_tools, Some(vec!["bash".to_string(), "file_edit".to_string()]));
        assert_eq!(instr.denied_tools, Some(vec!["delete_file".to_string()]));
    }

    #[test]
    fn test_is_tool_allowed_with_allowlist() {
        let mut instr = ProjectInstructions::default();
        instr.allowed_tools = Some(vec!["bash".to_string(), "file_edit".to_string()]);

        assert!(ProjectInstructionsLoader::is_tool_allowed(&instr, "bash"));
        assert!(!ProjectInstructionsLoader::is_tool_allowed(&instr, "delete_file"));
    }

    #[test]
    fn test_is_tool_allowed_with_denylist() {
        let mut instr = ProjectInstructions::default();
        instr.denied_tools = Some(vec!["delete_file".to_string()]);

        assert!(ProjectInstructionsLoader::is_tool_allowed(&instr, "bash"));
        assert!(!ProjectInstructionsLoader::is_tool_allowed(&instr, "delete_file"));
    }

    #[test]
    fn test_is_tool_allowed_both_lists() {
        let mut instr = ProjectInstructions::default();
        instr.allowed_tools = Some(vec!["bash".to_string(), "file_edit".to_string()]);
        instr.denied_tools = Some(vec!["bash".to_string()]);

        // Deny takes precedence
        assert!(!ProjectInstructionsLoader::is_tool_allowed(&instr, "bash"));
        assert!(ProjectInstructionsLoader::is_tool_allowed(&instr, "file_edit"));
    }

    #[test]
    fn test_path_access_no_access() {
        let mut instr = ProjectInstructions::default();
        instr.file_restrictions.no_access = vec!["*.secret".to_string()];

        let path = Path::new("config.secret");
        assert_eq!(
            ProjectInstructionsLoader::is_path_accessible(&instr, path),
            PathAccessResult::Denied("Path matches no-access pattern: *.secret".to_string())
        );
    }

    #[test]
    fn test_path_access_read_only() {
        let mut instr = ProjectInstructions::default();
        instr.file_restrictions.read_only = vec!["*.lock".to_string()];

        let path = Path::new("Cargo.lock");
        assert_eq!(
            ProjectInstructionsLoader::is_path_accessible(&instr, path),
            PathAccessResult::ReadOnly
        );
    }

    #[test]
    fn test_path_access_allowed() {
        let instr = ProjectInstructions::default();
        let path = Path::new("src/main.rs");

        assert_eq!(
            ProjectInstructionsLoader::is_path_accessible(&instr, path),
            PathAccessResult::Allowed
        );
    }

    #[test]
    fn test_should_auto_include() {
        let mut instr = ProjectInstructions::default();
        instr.file_restrictions.auto_include = vec!["*.md".to_string()];

        let path = Path::new("README.md");
        assert!(ProjectInstructionsLoader::should_auto_include(&instr, path));

        let path2 = Path::new("src/main.rs");
        assert!(!ProjectInstructionsLoader::should_auto_include(&instr, path2));
    }

    #[test]
    fn test_build_system_context() {
        let mut instr = ProjectInstructions::default();
        instr.instructions = vec!["Be careful".to_string()];
        instr.persona = Some("Expert developer".to_string());
        instr.allowed_tools = Some(vec!["bash".to_string()]);

        let context = ProjectInstructionsLoader::build_system_context(&instr);
        assert!(context.contains("Project Instructions"));
        assert!(context.contains("Role"));
        assert!(context.contains("Allowed Tools"));
    }

    #[tokio::test]
    async fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("HIVECODE.md");

        let content = r#"# Test Project
## Instructions
- Follow rules
"#;

        std::fs::write(&file_path, content).unwrap();

        let result = ProjectInstructionsLoader::load(&file_path).await;
        assert!(result.is_ok());

        let instr = result.unwrap();
        assert_eq!(instr.project_name, Some("Test Project".to_string()));
        assert!(!instr.instructions.is_empty());
    }

    #[test]
    fn test_find_instructions_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("HIVECODE.md");
        std::fs::write(&file_path, "# Project").unwrap();

        let found = ProjectInstructionsLoader::find_instructions_file(temp_dir.path());
        assert!(found.is_some());
    }

    #[test]
    fn test_glob_matches_simple() {
        assert!(ProjectInstructionsLoader::glob_matches("*.rs", "main.rs"));
        assert!(!ProjectInstructionsLoader::glob_matches("*.rs", "main.py"));
        assert!(ProjectInstructionsLoader::glob_matches("*", "anything.txt"));
    }
}

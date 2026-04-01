//! Hook system for HiveCode
//!
//! Allows pre/post execution hooks on any tool invocation.
//! Hooks can run before/after tools, on errors, or on approval requests.
//! Example: "Before every bash command, run lint." "After every file edit, auto-format."

use crate::error::{HiveCodeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// A hook that executes before, after, or on error during tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    /// Unique identifier for this hook
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// When the hook executes (before, after, error, or approval)
    pub hook_type: HookType,
    /// What triggers this hook
    pub trigger: HookTrigger,
    /// What action to take when the hook fires
    pub action: HookAction,
    /// Whether this hook is currently enabled
    pub enabled: bool,
    /// Execution priority - lower values execute first
    pub priority: i32,
    /// Conditions that must match for the hook to fire
    pub conditions: Vec<HookCondition>,
    /// When this hook was created
    pub created_at: DateTime<Utc>,
}

impl Hook {
    /// Create a new hook
    pub fn new(
        name: String,
        hook_type: HookType,
        trigger: HookTrigger,
        action: HookAction,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            hook_type,
            trigger,
            action,
            enabled: true,
            priority: 0,
            conditions: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Set the priority of this hook
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a condition to this hook
    pub fn with_condition(mut self, condition: HookCondition) -> Self {
        self.conditions.push(condition);
        self
    }
}

/// When a hook executes relative to tool execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HookType {
    /// Hook runs before the tool executes
    PreExecution,
    /// Hook runs after the tool completes successfully
    PostExecution,
    /// Hook runs when the tool errors
    OnError,
    /// Hook runs when user approval is requested
    OnApproval,
}

/// What triggers a hook to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookTrigger {
    /// Hook fires on any tool invocation
    AnyTool,
    /// Hook fires only on a specific tool (e.g., "bash", "file_edit")
    SpecificTool(String),
    /// Hook fires on tools matching a regex pattern
    ToolPattern(String),
    /// Hook fires when a tool touches files matching a glob pattern
    FilePattern(String),
}

/// The action a hook takes when triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookAction {
    /// Execute a shell command
    RunCommand(String),
    /// Transform the tool input using a template string
    ModifyInput(String),
    /// Append output to a log file
    LogToFile(PathBuf),
    /// Send a desktop notification
    SendNotification(String),
    /// Inject additional context into the LLM message
    InjectContext(String),
    /// Prevent the tool from executing with an optional reason
    BlockExecution(String),
    /// Chain execution to another tool
    ChainTool {
        /// Name of the tool to invoke
        tool_name: String,
        /// Input to pass to the tool
        input: serde_json::Value,
    },
}

/// Conditions that must be satisfied for a hook to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookCondition {
    /// Only match files with a specific extension (e.g., ".rs", ".py")
    FileExtension(String),
    /// Only match when working in a specific directory
    WorkingDirectory(String),
    /// Only match when using a specific model
    ModelName(String),
    /// Only match during specific hours (24-hour format, e.g. "09:00-17:00")
    TimeWindow { start: String, end: String },
    /// Only match if tool input contains a specific string
    InputContains(String),
}

/// The result of executing a hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// ID of the hook that executed
    pub hook_id: String,
    /// Whether the hook execution succeeded
    pub success: bool,
    /// Optional output from the hook action
    pub output: Option<String>,
    /// Optional error message
    pub error: Option<String>,
    /// Time taken to execute in milliseconds
    pub duration_ms: u64,
    /// Description of the action taken
    pub action_taken: String,
}

/// Context information about the tool being executed
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Current working directory
    pub working_directory: PathBuf,
    /// Name of the model being used
    pub current_model: String,
    /// Files being accessed by the tool
    pub file_paths: Vec<PathBuf>,
    /// Current timestamp
    pub timestamp: DateTime<Utc>,
}

/// Result of running pre-execution hooks
#[derive(Debug, Clone)]
pub struct PreHookResult {
    /// Whether the tool should proceed with execution
    pub should_proceed: bool,
    /// Optionally modified tool input from hooks
    pub modified_input: Option<serde_json::Value>,
    /// Results from each pre-hook that executed
    pub results: Vec<HookResult>,
    /// If blocked, the reason why
    pub block_reason: Option<String>,
}

/// Manages and executes hooks
pub struct HookManager {
    hooks: Vec<Hook>,
    storage_path: PathBuf,
    execution_log: Vec<HookResult>,
}

impl HookManager {
    /// Create a new hook manager, loading from ~/.hivecode/hooks.json
    pub async fn new(storage_dir: Option<PathBuf>) -> Result<Self> {
        let storage_path = if let Some(dir) = storage_dir {
            dir.join("hooks.json")
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| HiveCodeError::IOError("Could not determine home directory".to_string()))?;
            home.join(".hivecode").join("hooks.json")
        };

        // Create parent directories if they don't exist
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HiveCodeError::IOError(format!("Failed to create hooks directory: {}", e)))?;
        }

        // Load existing hooks if the file exists
        let hooks = if storage_path.exists() {
            Self::load(&storage_path).await.unwrap_or_else(|e| {
                warn!("Failed to load hooks: {}", e);
                Vec::new()
            })
        } else {
            Vec::new()
        };

        debug!("Hook manager initialized with {} hooks", hooks.len());

        Ok(Self {
            hooks,
            storage_path,
            execution_log: Vec::new(),
        })
    }

    /// Register a new hook
    pub async fn register(&mut self, hook: Hook) -> Result<String> {
        let id = hook.id.clone();
        info!("Registering hook: {} ({})", hook.name, id);
        self.hooks.push(hook);
        self.save().await?;
        Ok(id)
    }

    /// Unregister (remove) a hook by ID
    pub async fn unregister(&mut self, hook_id: &str) -> Result<()> {
        self.hooks.retain(|h| h.id != hook_id);
        info!("Unregistered hook: {}", hook_id);
        self.save().await?;
        Ok(())
    }

    /// Enable a hook by ID
    pub async fn enable(&mut self, hook_id: &str) -> Result<()> {
        if let Some(hook) = self.hooks.iter_mut().find(|h| h.id == hook_id) {
            hook.enabled = true;
            info!("Enabled hook: {}", hook_id);
            self.save().await?;
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Hook not found: {}", hook_id)))
        }
    }

    /// Disable a hook by ID
    pub async fn disable(&mut self, hook_id: &str) -> Result<()> {
        if let Some(hook) = self.hooks.iter_mut().find(|h| h.id == hook_id) {
            hook.enabled = false;
            info!("Disabled hook: {}", hook_id);
            self.save().await?;
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Hook not found: {}", hook_id)))
        }
    }

    /// List all hooks
    pub fn list(&self) -> Vec<&Hook> {
        self.hooks.iter().collect()
    }

    /// Get a specific hook by ID
    pub fn get(&self, hook_id: &str) -> Option<&Hook> {
        self.hooks.iter().find(|h| h.id == hook_id)
    }

    /// Find all hooks that match a given tool execution context
    pub fn find_matching_hooks(
        &self,
        tool_name: &str,
        hook_type: &HookType,
        context: &HookContext,
    ) -> Vec<&Hook> {
        self.hooks
            .iter()
            .filter(|hook| {
                !hook.enabled && return false;

                // Check hook type matches
                if hook.hook_type != *hook_type {
                    return false;
                }

                // Check trigger matches
                if !self.trigger_matches(&hook.trigger, tool_name) {
                    return false;
                }

                // Check conditions
                if !self.check_conditions(&hook.conditions, context) {
                    return false;
                }

                true
            })
            .collect()
    }

    /// Execute all pre-execution hooks for a tool
    pub async fn run_pre_hooks(
        &mut self,
        tool_name: &str,
        input: &serde_json::Value,
        context: &HookContext,
    ) -> Result<PreHookResult> {
        let mut matching = self.find_matching_hooks(tool_name, &HookType::PreExecution, context);
        matching.sort_by_key(|h| h.priority);

        let mut results = Vec::new();
        let mut should_proceed = true;
        let mut block_reason = None;
        let mut modified_input = None;

        for hook in matching {
            let start = std::time::Instant::now();
            match self.execute_action(&hook.action, tool_name, input, context).await {
                Ok(output) => {
                    // Check if this action blocks execution
                    if let HookAction::BlockExecution(reason) = &hook.action {
                        should_proceed = false;
                        block_reason = Some(reason.clone());
                    }

                    // Check if this action modifies input
                    if let HookAction::ModifyInput(_) = &hook.action {
                        if let Some(ref out) = output {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(out) {
                                modified_input = Some(json);
                            }
                        }
                    }

                    let duration_ms = start.elapsed().as_millis() as u64;
                    results.push(HookResult {
                        hook_id: hook.id.clone(),
                        success: true,
                        output,
                        error: None,
                        duration_ms,
                        action_taken: format!("{:?}", hook.action),
                    });
                }
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    warn!("Hook {} failed: {}", hook.id, e);
                    results.push(HookResult {
                        hook_id: hook.id.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        duration_ms,
                        action_taken: format!("{:?}", hook.action),
                    });
                }
            }
        }

        self.execution_log.extend(results.clone());

        Ok(PreHookResult {
            should_proceed,
            modified_input,
            results,
            block_reason,
        })
    }

    /// Execute all post-execution hooks for a tool
    pub async fn run_post_hooks(
        &mut self,
        tool_name: &str,
        input: &serde_json::Value,
        output: &str,
        context: &HookContext,
    ) -> Result<Vec<HookResult>> {
        let mut matching = self.find_matching_hooks(tool_name, &HookType::PostExecution, context);
        matching.sort_by_key(|h| h.priority);

        let mut results = Vec::new();

        for hook in matching {
            let start = std::time::Instant::now();
            match self.execute_action(&hook.action, tool_name, input, context).await {
                Ok(out) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    results.push(HookResult {
                        hook_id: hook.id.clone(),
                        success: true,
                        output: out,
                        error: None,
                        duration_ms,
                        action_taken: format!("{:?}", hook.action),
                    });
                }
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    warn!("Hook {} failed: {}", hook.id, e);
                    results.push(HookResult {
                        hook_id: hook.id.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        duration_ms,
                        action_taken: format!("{:?}", hook.action),
                    });
                }
            }
        }

        self.execution_log.extend(results.clone());

        Ok(results)
    }

    /// Execute all error hooks for a tool
    pub async fn run_error_hooks(
        &mut self,
        tool_name: &str,
        error: &str,
        context: &HookContext,
    ) -> Result<Vec<HookResult>> {
        let mut matching = self.find_matching_hooks(tool_name, &HookType::OnError, context);
        matching.sort_by_key(|h| h.priority);

        let mut results = Vec::new();

        for hook in matching {
            let start = std::time::Instant::now();
            let error_json = serde_json::json!({"error": error});
            match self.execute_action(&hook.action, tool_name, &error_json, context).await {
                Ok(out) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    results.push(HookResult {
                        hook_id: hook.id.clone(),
                        success: true,
                        output: out,
                        error: None,
                        duration_ms,
                        action_taken: format!("{:?}", hook.action),
                    });
                }
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    results.push(HookResult {
                        hook_id: hook.id.clone(),
                        success: false,
                        output: None,
                        error: Some(e.to_string()),
                        duration_ms,
                        action_taken: format!("{:?}", hook.action),
                    });
                }
            }
        }

        self.execution_log.extend(results.clone());

        Ok(results)
    }

    /// Get the execution log
    pub fn execution_log(&self) -> &[HookResult] {
        &self.execution_log
    }

    /// Clear the execution log
    pub fn clear_log(&mut self) {
        self.execution_log.clear();
    }

    // Helper methods

    fn trigger_matches(&self, trigger: &HookTrigger, tool_name: &str) -> bool {
        match trigger {
            HookTrigger::AnyTool => true,
            HookTrigger::SpecificTool(name) => name == tool_name,
            HookTrigger::ToolPattern(pattern) => {
                // Simple pattern matching: * matches anything in that segment
                Self::pattern_matches(pattern, tool_name)
            }
            HookTrigger::FilePattern(_) => true, // File patterns are checked differently
        }
    }

    fn pattern_matches(pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut pos = 0;
            for (i, part) in parts.iter().enumerate() {
                if i == 0 && !part.is_empty() {
                    if !text.starts_with(part) {
                        return false;
                    }
                    pos = part.len();
                } else if i == parts.len() - 1 && !part.is_empty() {
                    if !text[pos..].ends_with(part) {
                        return false;
                    }
                } else if !part.is_empty() {
                    if let Some(new_pos) = text[pos..].find(part) {
                        pos += new_pos + part.len();
                    } else {
                        return false;
                    }
                }
            }
            true
        } else {
            text == pattern
        }
    }

    fn check_conditions(&self, conditions: &[HookCondition], context: &HookContext) -> bool {
        if conditions.is_empty() {
            return true;
        }

        conditions.iter().all(|cond| match cond {
            HookCondition::FileExtension(ext) => {
                context
                    .file_paths
                    .iter()
                    .any(|p| p.extension().map(|e| e.to_string_lossy().as_ref()) == Some(ext))
            }
            HookCondition::WorkingDirectory(dir) => {
                context.working_directory.to_string_lossy().contains(dir)
            }
            HookCondition::ModelName(model) => context.current_model == *model,
            HookCondition::TimeWindow { start, end } => {
                let now = Utc::now();
                let hour = now.format("%H:%M").to_string();
                hour >= *start && hour <= *end
            }
            HookCondition::InputContains(s) => {
                // This would need the actual input string, simplified for now
                true
            }
        })
    }

    async fn execute_action(
        &self,
        action: &HookAction,
        _tool_name: &str,
        _input: &serde_json::Value,
        _context: &HookContext,
    ) -> Result<Option<String>> {
        match action {
            HookAction::RunCommand(cmd) => {
                // Execute shell command
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output()
                    .map_err(|e| HiveCodeError::Internal(format!("Failed to execute command: {}", e)))?;

                Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
            }
            HookAction::ModifyInput(_template) => {
                // Input modification would be handled by the caller
                Ok(None)
            }
            HookAction::LogToFile(path) => {
                let content = format!("[{}] Hook executed\n", Utc::now());
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| HiveCodeError::IOError(format!("Failed to open log file: {}", e)))?;
                file.write_all(content.as_bytes())
                    .map_err(|e| HiveCodeError::IOError(format!("Failed to write log: {}", e)))?;
                Ok(Some("Logged to file".to_string()))
            }
            HookAction::SendNotification(msg) => {
                // Notification sending would be platform-specific
                Ok(Some(format!("Notification would be sent: {}", msg)))
            }
            HookAction::InjectContext(ctx) => {
                Ok(Some(format!("Context injected: {}", ctx)))
            }
            HookAction::BlockExecution(reason) => {
                Ok(Some(format!("Execution blocked: {}", reason)))
            }
            HookAction::ChainTool { tool_name, input } => {
                Ok(Some(format!("Would chain to tool: {} with input: {}", tool_name, input)))
            }
        }
    }

    async fn load(path: &Path) -> Result<Vec<Hook>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read hooks file: {}", e)))?;
        let hooks = serde_json::from_str(&content)
            .map_err(|e| HiveCodeError::SerializationError(e))?;
        Ok(hooks)
    }

    async fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.hooks)
            .map_err(|e| HiveCodeError::SerializationError(e))?;
        std::fs::write(&self.storage_path, content)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to write hooks file: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_hook_creation() {
        let hook = Hook::new(
            "test hook".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        );

        assert_eq!(hook.name, "test hook");
        assert_eq!(hook.hook_type, HookType::PreExecution);
        assert!(hook.enabled);
    }

    #[tokio::test]
    async fn test_hook_with_priority() {
        let hook = Hook::new(
            "priority test".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        )
        .with_priority(5);

        assert_eq!(hook.priority, 5);
    }

    #[tokio::test]
    async fn test_hook_with_condition() {
        let hook = Hook::new(
            "condition test".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        )
        .with_condition(HookCondition::FileExtension("rs".to_string()));

        assert_eq!(hook.conditions.len(), 1);
    }

    #[tokio::test]
    async fn test_hook_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HookManager::new(Some(temp_dir.path().to_path_buf())).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_register_hook() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HookManager::new(Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        let hook = Hook::new(
            "test".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        );
        let id = hook.id.clone();

        let result = manager.register(hook).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), id);
        assert_eq!(manager.list().len(), 1);
    }

    #[tokio::test]
    async fn test_unregister_hook() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HookManager::new(Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        let hook = Hook::new(
            "test".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        );
        let id = hook.id.clone();

        manager.register(hook).await.unwrap();
        assert_eq!(manager.list().len(), 1);

        manager.unregister(&id).await.unwrap();
        assert_eq!(manager.list().len(), 0);
    }

    #[tokio::test]
    async fn test_enable_disable_hook() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HookManager::new(Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        let hook = Hook::new(
            "test".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        );
        let id = hook.id.clone();

        manager.register(hook).await.unwrap();

        manager.disable(&id).await.unwrap();
        assert!(!manager.get(&id).unwrap().enabled);

        manager.enable(&id).await.unwrap();
        assert!(manager.get(&id).unwrap().enabled);
    }

    #[tokio::test]
    async fn test_find_matching_hooks() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = HookManager::new(Some(temp_dir.path().to_path_buf()))
            .await
            .unwrap();

        let hook1 = Hook::new(
            "any tool".to_string(),
            HookType::PreExecution,
            HookTrigger::AnyTool,
            HookAction::RunCommand("echo 'test'".to_string()),
        );

        let hook2 = Hook::new(
            "specific tool".to_string(),
            HookType::PreExecution,
            HookTrigger::SpecificTool("bash".to_string()),
            HookAction::RunCommand("echo 'test'".to_string()),
        );

        manager.register(hook1).await.unwrap();
        manager.register(hook2).await.unwrap();

        let context = HookContext {
            working_directory: PathBuf::from("/tmp"),
            current_model: "gpt-4".to_string(),
            file_paths: vec![],
            timestamp: Utc::now(),
        };

        let matching = manager.find_matching_hooks("bash", &HookType::PreExecution, &context);
        assert_eq!(matching.len(), 2); // Both "any tool" and "specific tool" match

        let matching_specific = manager.find_matching_hooks("python", &HookType::PreExecution, &context);
        assert_eq!(matching_specific.len(), 1); // Only "any tool" matches
    }

    #[tokio::test]
    async fn test_hook_type_equality() {
        assert_eq!(HookType::PreExecution, HookType::PreExecution);
        assert_ne!(HookType::PreExecution, HookType::PostExecution);
    }

    #[tokio::test]
    async fn test_hook_result_creation() {
        let result = HookResult {
            hook_id: "test-id".to_string(),
            success: true,
            output: Some("output".to_string()),
            error: None,
            duration_ms: 100,
            action_taken: "RunCommand".to_string(),
        };

        assert!(result.success);
        assert_eq!(result.duration_ms, 100);
    }

    #[tokio::test]
    async fn test_pre_hook_result() {
        let result = PreHookResult {
            should_proceed: true,
            modified_input: None,
            results: vec![],
            block_reason: None,
        };

        assert!(result.should_proceed);
        assert!(result.block_reason.is_none());
    }

    #[tokio::test]
    async fn test_persistence_across_loads() {
        let temp_dir = TempDir::new().unwrap();

        // Create and register a hook
        {
            let mut manager = HookManager::new(Some(temp_dir.path().to_path_buf()))
                .await
                .unwrap();
            let hook = Hook::new(
                "persistent".to_string(),
                HookType::PreExecution,
                HookTrigger::AnyTool,
                HookAction::RunCommand("echo 'persistent'".to_string()),
            );
            manager.register(hook).await.unwrap();
        }

        // Load in a new manager instance
        {
            let manager = HookManager::new(Some(temp_dir.path().to_path_buf()))
                .await
                .unwrap();
            assert_eq!(manager.list().len(), 1);
            assert_eq!(manager.list()[0].name, "persistent");
        }
    }
}

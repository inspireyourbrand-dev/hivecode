use async_trait::async_trait;
use serde_json::Value;
use std::path::Path;

use crate::error::SecurityError;
use crate::rules::PermissionRuleSet;
use crate::shell_security::ShellSecurityChecker;
use crate::path_validator::PathValidator;

/// Result of a permission check
#[derive(Clone, Debug, PartialEq)]
pub enum PermissionResult {
    /// Action is allowed
    Allowed,
    /// Action is denied with a reason
    Denied(String),
    /// Action requires explicit user approval with a reason
    NeedsApproval(String),
}

impl PermissionResult {
    /// Convert result to a Result type, treating NeedsApproval as Denied
    pub fn check(self) -> Result<(), SecurityError> {
        match self {
            PermissionResult::Allowed => Ok(()),
            PermissionResult::Denied(reason) => Err(SecurityError::PermissionDenied(reason)),
            PermissionResult::NeedsApproval(reason) => {
                Err(SecurityError::PermissionDenied(reason))
            }
        }
    }
}

/// Core trait for checking permissions
#[async_trait]
pub trait PermissionChecker: Send + Sync {
    /// Check if a tool is allowed to be used with the given input
    async fn check_tool(&self, tool_name: &str, input: &Value) -> PermissionResult;

    /// Check if a path can be accessed (write=true for write, false for read)
    async fn check_path(&self, path: &Path, write: bool) -> PermissionResult;

    /// Check if a command is safe to execute
    async fn check_command(&self, command: &str) -> PermissionResult;
}

/// Default permission checker implementation
pub struct DefaultPermissionChecker {
    rule_set: PermissionRuleSet,
    shell_checker: ShellSecurityChecker,
    path_validator: PathValidator,
}

impl DefaultPermissionChecker {
    pub fn new() -> Self {
        Self {
            rule_set: PermissionRuleSet::new(),
            shell_checker: ShellSecurityChecker::new(),
            path_validator: PathValidator::new(),
        }
    }

    pub fn with_rules(mut self, rule_set: PermissionRuleSet) -> Self {
        self.rule_set = rule_set;
        self
    }
}

impl Default for DefaultPermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PermissionChecker for DefaultPermissionChecker {
    async fn check_tool(&self, tool_name: &str, _input: &Value) -> PermissionResult {
        // Check if tool is in deny list
        if matches!(
            tool_name,
            "dangerous_tool" | "unimplemented" | "system_access"
        ) {
            return PermissionResult::Denied(format!("Tool '{}' is not available", tool_name));
        }

        // All other tools are allowed
        PermissionResult::Allowed
    }

    async fn check_path(&self, path: &Path, write: bool) -> PermissionResult {
        // Check if path is sensitive
        if self.path_validator.is_sensitive_file(path) {
            return PermissionResult::Denied(format!(
                "Access to sensitive file is denied: {}",
                path.display()
            ));
        }

        // Check if path is valid
        if !self.path_validator.is_valid_path(path) {
            return PermissionResult::Denied(format!(
                "Invalid path: {}",
                path.display()
            ));
        }

        // All other paths are allowed
        PermissionResult::Allowed
    }

    async fn check_command(&self, command: &str) -> PermissionResult {
        let risk = self.shell_checker.classify_command(command);

        match risk {
            crate::shell_security::CommandRisk::Safe => PermissionResult::Allowed,
            crate::shell_security::CommandRisk::Moderate => PermissionResult::NeedsApproval(
                format!("Command requires approval: {}", command),
            ),
            crate::shell_security::CommandRisk::Dangerous => PermissionResult::Denied(
                format!("Dangerous command is blocked: {}", command),
            ),
            crate::shell_security::CommandRisk::Blocked => PermissionResult::Denied(
                format!("Command is not allowed: {}", command),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_result_allowed() {
        let result = PermissionResult::Allowed;
        assert!(result.check().is_ok());
    }

    #[test]
    fn test_permission_result_denied() {
        let result = PermissionResult::Denied("test".to_string());
        assert!(result.check().is_err());
    }

    #[test]
    fn test_permission_result_needs_approval() {
        let result = PermissionResult::NeedsApproval("test".to_string());
        assert!(result.check().is_err());
    }

    #[tokio::test]
    async fn test_default_checker_safe_command() {
        let checker = DefaultPermissionChecker::new();
        let result = checker.check_command("echo 'hello'").await;
        assert_eq!(result, PermissionResult::Allowed);
    }

    #[tokio::test]
    async fn test_default_checker_dangerous_command() {
        let checker = DefaultPermissionChecker::new();
        let result = checker.check_command("rm -rf /").await;
        assert!(matches!(result, PermissionResult::Denied(_)));
    }

    #[tokio::test]
    async fn test_default_checker_tool() {
        let checker = DefaultPermissionChecker::new();
        let result = checker
            .check_tool("file_read", &serde_json::json!({}))
            .await;
        assert_eq!(result, PermissionResult::Allowed);
    }
}

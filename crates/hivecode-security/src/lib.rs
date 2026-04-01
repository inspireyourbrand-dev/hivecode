pub mod checker;
pub mod rules;
pub mod shell_security;
pub mod path_validator;
pub mod error;

pub use checker::{PermissionChecker, PermissionResult, DefaultPermissionChecker};
pub use rules::{PermissionRule, PermissionAction, PermissionRuleSet};
pub use shell_security::ShellSecurityChecker;
pub use path_validator::PathValidator;
pub use error::SecurityError;

use std::sync::Arc;

/// Create a default permission checker with sensible defaults
pub fn create_default_permission_checker() -> DefaultPermissionChecker {
    DefaultPermissionChecker::new()
}

/// Create a default shell security checker
pub fn create_shell_security_checker() -> ShellSecurityChecker {
    ShellSecurityChecker::new()
}

/// Create a default path validator
pub fn create_path_validator() -> PathValidator {
    PathValidator::new()
}

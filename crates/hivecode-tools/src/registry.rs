use std::sync::Arc;
use serde_json::json;
use std::collections::HashMap;

use crate::traits::{Tool, ToolContext, ToolResult};
use crate::error::ToolError;
use hivecode_security::PermissionChecker;

/// Registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    permission_checker: Arc<dyn PermissionChecker>,
}

impl ToolRegistry {
    pub fn new(permission_checker: Arc<dyn PermissionChecker>) -> Self {
        Self {
            tools: HashMap::new(),
            permission_checker,
        }
    }

    /// Register a tool in the registry
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// List all registered tool names
    pub fn list(&self) -> Vec<String> {
        let mut names: Vec<_> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all tool schemas
    pub fn get_all_schemas(&self) -> serde_json::Value {
        let mut schemas = json!({});

        for (name, tool) in &self.tools {
            schemas[name] = json!({
                "name": tool.name(),
                "description": tool.description(),
                "enabled": tool.is_enabled(),
                "input_schema": tool.input_schema(),
            });
        }

        schemas
    }

    /// Execute a tool
    pub async fn execute(
        &self,
        tool_name: &str,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let tool = self
            .get(tool_name)
            .ok_or_else(|| ToolError::InvalidInput(format!("Tool '{}' not found", tool_name)))?;

        if !tool.is_enabled() {
            return Err(ToolError::InvalidInput(format!(
                "Tool '{}' is not enabled",
                tool_name
            )));
        }

        tool.execute(input, ctx).await
    }

    pub fn permission_checker(&self) -> Arc<dyn PermissionChecker> {
        self.permission_checker.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock"
        }

        fn description(&self) -> &str {
            "A mock tool for testing"
        }

        fn input_schema(&self) -> serde_json::Value {
            json!({ "type": "object" })
        }

        async fn execute(
            &self,
            _input: serde_json::Value,
            _ctx: &ToolContext,
        ) -> Result<ToolResult, ToolError> {
            Ok(ToolResult::success("mock output"))
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let checker = hivecode_security::create_default_permission_checker();
        let mut registry = ToolRegistry::new(checker);

        registry.register(Arc::new(MockTool));

        assert!(registry.get("mock").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_list() {
        let checker = hivecode_security::create_default_permission_checker();
        let mut registry = ToolRegistry::new(checker);

        registry.register(Arc::new(MockTool));

        let tools = registry.list();
        assert_eq!(tools, vec!["mock"]);
    }

    #[test]
    fn test_registry_get_all_schemas() {
        let checker = hivecode_security::create_default_permission_checker();
        let mut registry = ToolRegistry::new(checker);

        registry.register(Arc::new(MockTool));

        let schemas = registry.get_all_schemas();
        assert!(schemas["mock"]["name"].is_string());
        assert_eq!(schemas["mock"]["name"], "mock");
    }
}

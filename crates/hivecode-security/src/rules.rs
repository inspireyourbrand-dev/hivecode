use glob::Pattern;
use std::path::PathBuf;

/// Action a permission rule can take
#[derive(Clone, Debug, PartialEq)]
pub enum PermissionAction {
    Allow,
    Deny,
    Ask,
}

/// A single permission rule
#[derive(Clone, Debug)]
pub struct PermissionRule {
    pub tool_name: Option<String>,
    pub action: PermissionAction,
    pub pattern: Option<String>,
    pub source: String,
}

impl PermissionRule {
    pub fn new(action: PermissionAction, source: impl Into<String>) -> Self {
        Self {
            tool_name: None,
            action,
            pattern: None,
            source: source.into(),
        }
    }

    pub fn for_tool(mut self, tool_name: impl Into<String>) -> Self {
        self.tool_name = Some(tool_name.into());
        self
    }

    pub fn matching_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Check if this rule matches the given inputs
    pub fn matches(&self, tool_name: &str, pattern: Option<&str>) -> bool {
        // Check tool name
        if let Some(ref rule_tool) = self.tool_name {
            if rule_tool != tool_name {
                return false;
            }
        }

        // Check pattern
        if let Some(ref rule_pattern) = self.pattern {
            if let Some(input_pattern) = pattern {
                if let Ok(glob_pattern) = Pattern::new(rule_pattern) {
                    if !glob_pattern.matches(input_pattern) {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Set of permission rules
pub struct PermissionRuleSet {
    deny_rules: Vec<PermissionRule>,
    allow_rules: Vec<PermissionRule>,
    ask_rules: Vec<PermissionRule>,
}

impl PermissionRuleSet {
    pub fn new() -> Self {
        Self {
            deny_rules: Vec::new(),
            allow_rules: Vec::new(),
            ask_rules: Vec::new(),
        }
    }

    /// Add a deny rule
    pub fn add_deny_rule(&mut self, rule: PermissionRule) {
        self.deny_rules.push(rule);
    }

    /// Add an allow rule
    pub fn add_allow_rule(&mut self, rule: PermissionRule) {
        self.allow_rules.push(rule);
    }

    /// Add an ask rule
    pub fn add_ask_rule(&mut self, rule: PermissionRule) {
        self.ask_rules.push(rule);
    }

    /// Evaluate rules in order: deny first, then allow, then ask, then default to ask
    pub fn evaluate(&self, tool_name: &str, pattern: Option<&str>) -> PermissionAction {
        // Check deny rules first
        for rule in &self.deny_rules {
            if rule.matches(tool_name, pattern) {
                return PermissionAction::Deny;
            }
        }

        // Check allow rules
        for rule in &self.allow_rules {
            if rule.matches(tool_name, pattern) {
                return PermissionAction::Allow;
            }
        }

        // Check ask rules
        for rule in &self.ask_rules {
            if rule.matches(tool_name, pattern) {
                return PermissionAction::Ask;
            }
        }

        // Default to ask
        PermissionAction::Ask
    }
}

impl Default for PermissionRuleSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_rule_matches_tool() {
        let rule = PermissionRule::new(PermissionAction::Deny, "test")
            .for_tool("bash");

        assert!(rule.matches("bash", None));
        assert!(!rule.matches("file_read", None));
    }

    #[test]
    fn test_permission_rule_matches_pattern() {
        let rule = PermissionRule::new(PermissionAction::Deny, "test")
            .matching_pattern("*.log");

        assert!(rule.matches("", Some("test.log")));
        assert!(!rule.matches("", Some("test.txt")));
    }

    #[test]
    fn test_permission_rule_matches_both() {
        let rule = PermissionRule::new(PermissionAction::Deny, "test")
            .for_tool("file_write")
            .matching_pattern("/etc/*");

        assert!(rule.matches("file_write", Some("/etc/passwd")));
        assert!(!rule.matches("file_read", Some("/etc/passwd")));
        assert!(!rule.matches("file_write", Some("/tmp/test")));
    }

    #[test]
    fn test_rule_set_deny_priority() {
        let mut rule_set = PermissionRuleSet::new();

        rule_set.add_allow_rule(
            PermissionRule::new(PermissionAction::Allow, "test")
                .for_tool("bash")
        );

        rule_set.add_deny_rule(
            PermissionRule::new(PermissionAction::Deny, "test")
                .for_tool("bash")
        );

        // Deny should take priority
        assert_eq!(rule_set.evaluate("bash", None), PermissionAction::Deny);
    }

    #[test]
    fn test_rule_set_allow_priority() {
        let mut rule_set = PermissionRuleSet::new();

        rule_set.add_ask_rule(
            PermissionRule::new(PermissionAction::Ask, "test")
                .for_tool("bash")
        );

        rule_set.add_allow_rule(
            PermissionRule::new(PermissionAction::Allow, "test")
                .for_tool("bash")
        );

        // Allow should take priority over ask
        assert_eq!(rule_set.evaluate("bash", None), PermissionAction::Allow);
    }

    #[test]
    fn test_rule_set_default_to_ask() {
        let rule_set = PermissionRuleSet::new();

        // With no rules, should default to ask
        assert_eq!(rule_set.evaluate("bash", None), PermissionAction::Ask);
    }
}

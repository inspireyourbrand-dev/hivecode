//! Parallel tool execution engine for HiveCode
//! Detects independent tool calls and runs them concurrently for massive speed gains.
//! Falls back to sequential execution when tools have dependencies.

use crate::traits::{Tool, ToolContext, ToolResult};
use crate::registry::ToolRegistry;
use std::collections::{HashMap, HashSet};
use tokio::task::JoinSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub tool_name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ToolCallResult {
    pub id: String,
    pub tool_name: String,
    pub result: ToolResult,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct ParallelExecutionReport {
    pub results: Vec<ToolCallResult>,
    pub total_duration: Duration,
    pub sequential_estimate: Duration,
    pub speedup_factor: f64,
    pub execution_mode: ExecutionMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    AllParallel,
    PartialParallel { parallel_groups: usize },
    Sequential,
}

pub struct ParallelExecutor {
    max_concurrent: usize,
    timeout: Duration,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            max_concurrent: 10,
            timeout: Duration::from_secs(120),
        }
    }

    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Analyze tool calls for dependencies and group them
    pub fn analyze_dependencies(&self, calls: &[ToolCall]) -> Vec<Vec<usize>> {
        if calls.is_empty() {
            return vec![];
        }

        let mut groups: Vec<Vec<usize>> = vec![];
        let mut assigned = HashSet::new();

        for i in 0..calls.len() {
            if assigned.contains(&i) {
                continue;
            }

            let mut current_group = vec![i];
            assigned.insert(i);

            // Find all tools that can run in parallel with tool i
            for j in (i + 1)..calls.len() {
                if assigned.contains(&j) {
                    continue;
                }

                // Check if tool j is independent from all tools in current group
                let is_independent = current_group
                    .iter()
                    .all(|&k| Self::are_independent(&calls[k], &calls[j]));

                if is_independent {
                    current_group.push(j);
                    assigned.insert(j);
                }
            }

            groups.push(current_group);
        }

        groups
    }

    /// Execute multiple tool calls with automatic parallelization
    pub async fn execute_batch(
        &self,
        calls: Vec<ToolCall>,
        registry: Arc<ToolRegistry>,
        context: &ToolContext,
    ) -> Result<ParallelExecutionReport, String> {
        if calls.is_empty() {
            return Ok(ParallelExecutionReport {
                results: vec![],
                total_duration: Duration::from_secs(0),
                sequential_estimate: Duration::from_secs(0),
                speedup_factor: 1.0,
                execution_mode: ExecutionMode::AllParallel,
            });
        }

        let total_start = Instant::now();
        let groups = self.analyze_dependencies(&calls);

        debug!(
            "Executing {} tool calls in {} groups",
            calls.len(),
            groups.len()
        );

        let mut all_results = vec![];
        let mut sequential_estimate = Duration::from_secs(0);

        // Execute each group sequentially, but tools within a group in parallel
        for group in groups {
            let group_start = Instant::now();
            let mut group_results = self
                .execute_group(
                    group.iter().map(|&idx| calls[idx].clone()).collect(),
                    registry.clone(),
                    context,
                )
                .await;

            sequential_estimate += group_start.elapsed();
            all_results.append(&mut group_results);
        }

        let total_duration = total_start.elapsed();
        let speedup_factor = if total_duration.as_secs_f64() > 0.0 {
            sequential_estimate.as_secs_f64() / total_duration.as_secs_f64()
        } else {
            1.0
        };

        let execution_mode = if groups.len() == 1 {
            if calls.len() == 1 {
                ExecutionMode::Sequential
            } else {
                ExecutionMode::AllParallel
            }
        } else {
            ExecutionMode::PartialParallel {
                parallel_groups: groups.len(),
            }
        };

        debug!(
            "Execution complete: {} results, {:.2}x speedup",
            all_results.len(),
            speedup_factor
        );

        Ok(ParallelExecutionReport {
            results: all_results,
            total_duration,
            sequential_estimate,
            speedup_factor,
            execution_mode,
        })
    }

    /// Execute a single group of independent tools in parallel
    async fn execute_group(
        &self,
        group: Vec<ToolCall>,
        registry: Arc<ToolRegistry>,
        context: &ToolContext,
    ) -> Vec<ToolCallResult> {
        let mut join_set: JoinSet<ToolCallResult> = JoinSet::new();

        for call in group {
            let tool_name = call.tool_name.clone();
            let call_id = call.id.clone();
            let input = call.input.clone();
            let registry = registry.clone();
            let context_clone = context.clone();
            let timeout = self.timeout;

            join_set.spawn(async move {
                let start = Instant::now();
                let result = tokio::time::timeout(
                    timeout,
                    registry.execute(&tool_name, input, &context_clone),
                )
                .await;

                let tool_result = match result {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => ToolResult::error(e.to_string()),
                    Err(_) => ToolResult::error("Tool execution timed out"),
                };

                ToolCallResult {
                    id: call_id,
                    tool_name,
                    result: tool_result,
                    duration: start.elapsed(),
                }
            });
        }

        let mut results = vec![];
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(r) => results.push(r),
                Err(e) => {
                    warn!("Task join error: {}", e);
                }
            }
        }

        results
    }

    /// Check if two tool calls are independent (can run in parallel)
    fn are_independent(a: &ToolCall, b: &ToolCall) -> bool {
        // Same tool on potentially same file = dependent
        if a.tool_name == b.tool_name {
            if Self::tools_may_conflict(&a.tool_name) {
                let a_paths = Self::extract_paths(a);
                let b_paths = Self::extract_paths(b);

                // Check for path overlaps
                for a_path in &a_paths {
                    if b_paths.contains(a_path) {
                        return false;
                    }
                }
            }
        }

        // Write operations are always dependent on each other and on reads
        if Self::is_write_operation(&a.tool_name) || Self::is_write_operation(&b.tool_name) {
            if Self::is_write_operation(&a.tool_name) && Self::is_write_operation(&b.tool_name) {
                let a_paths = Self::extract_paths(a);
                let b_paths = Self::extract_paths(b);
                for a_path in &a_paths {
                    if b_paths.contains(a_path) {
                        return false;
                    }
                }
            } else if Self::is_write_operation(&a.tool_name) {
                // Write before read on same file
                let write_paths = Self::extract_paths(a);
                let read_paths = Self::extract_paths(b);
                for write_path in &write_paths {
                    if read_paths.contains(write_path) {
                        return false;
                    }
                }
            } else {
                // Write after read on same file
                let read_paths = Self::extract_paths(a);
                let write_paths = Self::extract_paths(b);
                for write_path in &write_paths {
                    if read_paths.contains(write_path) {
                        return false;
                    }
                }
            }
        }

        // Bash is conservative - assume potential dependencies
        if a.tool_name == "bash" || b.tool_name == "bash" {
            // Only allow bash in parallel with truly independent operations
            if a.tool_name == "bash" && b.tool_name == "bash" {
                // Two bash commands might interfere with each other
                return false;
            }
            // Allow bash with read-only operations
            let bash_idx = if a.tool_name == "bash" { 0 } else { 1 };
            let other_idx = 1 - bash_idx;
            let calls = [a, b];
            if !Self::is_read_only(&calls[other_idx].tool_name) {
                return false;
            }
        }

        // Git operations must be sequential
        if Self::is_git_operation(&a.tool_name) || Self::is_git_operation(&b.tool_name) {
            if a.tool_name == "git" || b.tool_name == "git" {
                return false;
            }
        }

        // Otherwise independent
        true
    }

    /// Check if a tool may have file conflicts
    fn tools_may_conflict(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "file_read" | "file_write" | "file_edit" | "grep" | "glob"
        )
    }

    /// Check if tool is a write operation
    fn is_write_operation(tool_name: &str) -> bool {
        matches!(tool_name, "file_write" | "file_edit" | "bash")
    }

    /// Check if tool is read-only
    fn is_read_only(tool_name: &str) -> bool {
        matches!(
            tool_name,
            "file_read" | "grep" | "glob" | "web_fetch" | "tool_search"
        )
    }

    /// Check if tool is git operation
    fn is_git_operation(tool_name: &str) -> bool {
        matches!(tool_name, "git")
    }

    /// Extract file paths from tool input for dependency analysis
    fn extract_paths(call: &ToolCall) -> Vec<String> {
        let mut paths = vec![];

        if let Some(obj) = call.input.as_object() {
            // Common path field names
            for field in &[
                "path",
                "file_path",
                "directory",
                "pattern",
                "from_path",
                "to_path",
                "paths",
            ] {
                if let Some(val) = obj.get(*field) {
                    if let Some(s) = val.as_str() {
                        paths.push(s.to_string());
                    } else if let Some(arr) = val.as_array() {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                paths.push(s.to_string());
                            }
                        }
                    }
                }
            }
        }

        paths
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_are_independent_read_operations() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "file_read".to_string(),
            input: serde_json::json!({"path": "/tmp/file1.txt"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "file_read".to_string(),
            input: serde_json::json!({"path": "/tmp/file2.txt"}),
        };

        assert!(ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_are_dependent_same_file_read_write() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "file_write".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "file_read".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
        };

        assert!(!ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_are_dependent_two_writes_same_file() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "file_write".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "file_edit".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
        };

        assert!(!ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_are_independent_different_files_write() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "file_write".to_string(),
            input: serde_json::json!({"path": "/tmp/file1.txt"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "file_write".to_string(),
            input: serde_json::json!({"path": "/tmp/file2.txt"}),
        };

        assert!(ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_bash_operations_are_dependent() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "bash".to_string(),
            input: serde_json::json!({"command": "ls"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "bash".to_string(),
            input: serde_json::json!({"command": "pwd"}),
        };

        assert!(!ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_bash_with_read_only_independent() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "bash".to_string(),
            input: serde_json::json!({"command": "ls"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "file_read".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
        };

        assert!(ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_git_operations_dependent() {
        let call1 = ToolCall {
            id: "1".to_string(),
            tool_name: "git".to_string(),
            input: serde_json::json!({"command": "status"}),
        };

        let call2 = ToolCall {
            id: "2".to_string(),
            tool_name: "git".to_string(),
            input: serde_json::json!({"command": "log"}),
        };

        assert!(!ParallelExecutor::are_independent(&call1, &call2));
    }

    #[test]
    fn test_analyze_dependencies_all_independent() {
        let executor = ParallelExecutor::new();
        let calls = vec![
            ToolCall {
                id: "1".to_string(),
                tool_name: "file_read".to_string(),
                input: serde_json::json!({"path": "/tmp/file1.txt"}),
            },
            ToolCall {
                id: "2".to_string(),
                tool_name: "file_read".to_string(),
                input: serde_json::json!({"path": "/tmp/file2.txt"}),
            },
            ToolCall {
                id: "3".to_string(),
                tool_name: "grep".to_string(),
                input: serde_json::json!({"pattern": "test", "path": "/tmp/file3.txt"}),
            },
        ];

        let groups = executor.analyze_dependencies(&calls);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn test_analyze_dependencies_with_dependencies() {
        let executor = ParallelExecutor::new();
        let calls = vec![
            ToolCall {
                id: "1".to_string(),
                tool_name: "file_write".to_string(),
                input: serde_json::json!({"path": "/tmp/file.txt"}),
            },
            ToolCall {
                id: "2".to_string(),
                tool_name: "file_read".to_string(),
                input: serde_json::json!({"path": "/tmp/file.txt"}),
            },
        ];

        let groups = executor.analyze_dependencies(&calls);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[1].len(), 1);
    }

    #[test]
    fn test_analyze_dependencies_partial_parallel() {
        let executor = ParallelExecutor::new();
        let calls = vec![
            ToolCall {
                id: "1".to_string(),
                tool_name: "file_read".to_string(),
                input: serde_json::json!({"path": "/tmp/file1.txt"}),
            },
            ToolCall {
                id: "2".to_string(),
                tool_name: "file_read".to_string(),
                input: serde_json::json!({"path": "/tmp/file2.txt"}),
            },
            ToolCall {
                id: "3".to_string(),
                tool_name: "file_write".to_string(),
                input: serde_json::json!({"path": "/tmp/file1.txt"}),
            },
        ];

        let groups = executor.analyze_dependencies(&calls);
        assert!(groups.len() >= 2);
    }

    #[test]
    fn test_extract_paths() {
        let call = ToolCall {
            id: "1".to_string(),
            tool_name: "file_read".to_string(),
            input: serde_json::json!({
                "path": "/tmp/file.txt",
                "file_path": "/tmp/other.txt"
            }),
        };

        let paths = ParallelExecutor::extract_paths(&call);
        assert!(paths.contains(&"/tmp/file.txt".to_string()));
        assert!(paths.contains(&"/tmp/other.txt".to_string()));
    }

    #[test]
    fn test_extract_paths_array() {
        let call = ToolCall {
            id: "1".to_string(),
            tool_name: "bash".to_string(),
            input: serde_json::json!({
                "paths": ["/tmp/file1.txt", "/tmp/file2.txt"]
            }),
        };

        let paths = ParallelExecutor::extract_paths(&call);
        assert!(paths.contains(&"/tmp/file1.txt".to_string()));
        assert!(paths.contains(&"/tmp/file2.txt".to_string()));
    }

    #[test]
    fn test_empty_calls() {
        let executor = ParallelExecutor::new();
        let groups = executor.analyze_dependencies(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_single_call() {
        let executor = ParallelExecutor::new();
        let calls = vec![ToolCall {
            id: "1".to_string(),
            tool_name: "file_read".to_string(),
            input: serde_json::json!({"path": "/tmp/file.txt"}),
        }];

        let groups = executor.analyze_dependencies(&calls);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 1);
    }
}

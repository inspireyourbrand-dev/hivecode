//! Git-aware context system for HiveCode
//!
//! Automatically discovers and includes relevant files based on git diff,
//! recent changes, and project structure. The AI doesn't need to be told
//! "look at file X" — it already knows what's relevant.

use crate::error::{HiveCodeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, warn};

/// Git context information for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContextInfo {
    /// Current branch name
    pub branch: String,
    /// Whether there are uncommitted changes
    pub has_uncommitted_changes: bool,
    /// Files that have been modified
    pub modified_files: Vec<FileChange>,
    /// Files that have been staged for commit
    pub staged_files: Vec<FileChange>,
    /// Untracked files in the repository
    pub untracked_files: Vec<PathBuf>,
    /// Recent commits in the current branch
    pub recent_commits: Vec<CommitInfo>,
    /// Files ranked by relevance
    pub relevant_files: Vec<RelevantFile>,
}

/// Information about a file change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path to the file
    pub path: PathBuf,
    /// Type of change
    pub change_type: ChangeType,
    /// Number of lines added
    pub additions: u32,
    /// Number of lines deleted
    pub deletions: u32,
    /// Preview of the diff (first 500 characters)
    pub diff_preview: Option<String>,
}

/// Type of change made to a file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    /// File was newly added
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed { from: String },
    /// File was copied
    Copied,
}

/// Information about a commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Full commit hash
    pub hash: String,
    /// Abbreviated commit hash
    pub short_hash: String,
    /// Commit message
    pub message: String,
    /// Author name and email
    pub author: String,
    /// Commit timestamp
    pub timestamp: String,
    /// Files changed in this commit
    pub files_changed: Vec<String>,
}

/// A file ranked by relevance to the current changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevantFile {
    /// Path to the file
    pub path: PathBuf,
    /// Relevance score from 0.0 to 1.0
    pub relevance_score: f64,
    /// Reason for relevance
    pub reason: RelevanceReason,
    /// File size in bytes
    pub size_bytes: u64,
    /// Detected programming language
    pub language: Option<String>,
}

/// Reason a file is considered relevant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelevanceReason {
    /// File is currently being modified
    CurrentlyModified,
    /// File was recently modified
    RecentlyModified,
    /// File is imported by a modified file
    ImportedByModified,
    /// File is in the same directory as modified files
    InSameModule,
    /// This is a test file for a modified file
    TestForModified,
    /// This is a project configuration file
    ConfigFile,
    /// File has a similar name pattern
    RelatedByName,
}

/// Builder for creating git context information
pub struct GitContextBuilder {
    project_dir: PathBuf,
    max_recent_commits: usize,
    max_relevant_files: usize,
    max_diff_preview_size: usize,
    include_untracked: bool,
}

impl GitContextBuilder {
    /// Create a new builder for the given project directory
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            project_dir,
            max_recent_commits: 10,
            max_relevant_files: 20,
            max_diff_preview_size: 500,
            include_untracked: true,
        }
    }

    /// Set the maximum number of recent commits to include
    pub fn with_max_commits(mut self, n: usize) -> Self {
        self.max_recent_commits = n;
        self
    }

    /// Set the maximum number of relevant files to include
    pub fn with_max_files(mut self, n: usize) -> Self {
        self.max_relevant_files = n;
        self
    }

    /// Set whether to include untracked files
    pub fn with_include_untracked(mut self, include: bool) -> Self {
        self.include_untracked = include;
        self
    }

    /// Build complete git context for the current project state
    pub async fn build(&self) -> Result<GitContextInfo> {
        let branch = self.get_branch()?;
        let modified_files = self.get_changes("--diff-filter=M")?;
        let staged_files = self.get_changes("--cached")?;
        let untracked_files = if self.include_untracked {
            self.get_untracked_files()?
        } else {
            Vec::new()
        };
        let recent_commits = self.get_recent_commits()?;

        let has_uncommitted_changes = !modified_files.is_empty() || !staged_files.is_empty();

        // Score files for relevance
        let mut relevant_files = Vec::new();
        let all_modified: Vec<_> = modified_files.iter().chain(staged_files.iter()).collect();

        for file_path in self.get_all_files()? {
            if let Ok(metadata) = std::fs::metadata(&file_path) {
                let score_and_reason =
                    self.score_relevance(&file_path, &all_modified, &self.project_dir);
                if let (score, reason) = score_and_reason {
                    if score > 0.1 {
                        let language = self.detect_language(&file_path);
                        relevant_files.push(RelevantFile {
                            path: file_path,
                            relevance_score: score,
                            reason,
                            size_bytes: metadata.len(),
                            language,
                        });
                    }
                }
            }
        }

        // Sort by relevance score (descending)
        relevant_files.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        relevant_files.truncate(self.max_relevant_files);

        Ok(GitContextInfo {
            branch,
            has_uncommitted_changes,
            modified_files,
            staged_files,
            untracked_files,
            recent_commits,
            relevant_files,
        })
    }

    /// Get just the modified/staged files (lightweight operation)
    pub async fn get_changes(&self) -> Result<Vec<FileChange>> {
        self.get_changes("--diff-filter=M")
    }

    /// Find files related to a given file
    pub async fn find_related(&self, file: &Path) -> Result<Vec<RelevantFile>> {
        let modified_files = self.get_changes("")?;
        let all_modified: Vec<_> = modified_files.iter().collect();

        let mut related = Vec::new();
        let parent_dir = file.parent().unwrap_or_else(|| Path::new("."));

        if let Ok(entries) = std::fs::read_dir(parent_dir) {
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().strip_prefix(&self.project_dir) {
                    let (score, reason) =
                        self.score_relevance(path, &all_modified, &self.project_dir);
                    if score > 0.1 {
                        let language = self.detect_language(path);
                        related.push(RelevantFile {
                            path: path.to_path_buf(),
                            relevance_score: score,
                            reason,
                            size_bytes: entry.metadata().ok().map(|m| m.len()).unwrap_or(0),
                            language,
                        });
                    }
                }
            }
        }

        related.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        Ok(related)
    }

    /// Build a context string suitable for including in LLM messages
    pub fn build_context_string(info: &GitContextInfo, max_tokens: usize) -> String {
        let mut context = String::new();
        let mut token_count = 0;
        let avg_chars_per_token = 4;
        let max_chars = max_tokens * avg_chars_per_token;

        context.push_str(&format!("## Git Context\n\nBranch: {}\n\n", info.branch));

        if info.has_uncommitted_changes {
            context.push_str("### Changes\n");

            if !info.staged_files.is_empty() {
                context.push_str("**Staged:**\n");
                for file in &info.staged_files {
                    let line = format!(
                        "- {} ({} +{} -{})\n",
                        file.path.display(),
                        match file.change_type {
                            ChangeType::Added => "added",
                            ChangeType::Modified => "modified",
                            ChangeType::Deleted => "deleted",
                            ChangeType::Renamed { .. } => "renamed",
                            ChangeType::Copied => "copied",
                        },
                        file.additions,
                        file.deletions
                    );
                    context.push_str(&line);
                }
            }

            if !info.modified_files.is_empty() {
                context.push_str("\n**Modified:**\n");
                for file in &info.modified_files {
                    let line = format!(
                        "- {} (+{} -{})\n",
                        file.path.display(),
                        file.additions,
                        file.deletions
                    );
                    context.push_str(&line);
                }
            }

            context.push('\n');
        }

        if !info.relevant_files.is_empty() {
            context.push_str("### Relevant Files\n");
            for file in info.relevant_files.iter().take(5) {
                let line = format!(
                    "- {} (relevance: {:.2}, {})\n",
                    file.path.display(),
                    file.relevance_score,
                    match file.reason {
                        RelevanceReason::CurrentlyModified => "currently modified",
                        RelevanceReason::RecentlyModified => "recently modified",
                        RelevanceReason::ImportedByModified => "imported by modified",
                        RelevanceReason::InSameModule => "same module",
                        RelevanceReason::TestForModified => "test file",
                        RelevanceReason::ConfigFile => "config file",
                        RelevanceReason::RelatedByName => "related by name",
                    }
                );
                context.push_str(&line);
            }
        }

        if context.len() > max_chars {
            context.truncate(max_chars);
            context.push_str("\n... (truncated)");
        }

        context
    }

    // Private helper methods

    fn get_branch(&self) -> Result<String> {
        let output = self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        Ok(output.trim().to_string())
    }

    fn get_changes(&self, diff_filter: &str) -> Result<Vec<FileChange>> {
        let mut args = vec!["diff", "--name-status"];
        if !diff_filter.is_empty() {
            args.push(diff_filter);
        }

        let output = self.run_git_command(&args)?;
        Ok(self.parse_git_diff(&output))
    }

    fn get_untracked_files(&self) -> Result<Vec<PathBuf>> {
        let output = self.run_git_command(&["ls-files", "--others", "--exclude-standard"])?;
        Ok(output
            .lines()
            .map(|line| PathBuf::from(line))
            .collect())
    }

    fn get_recent_commits(&self) -> Result<Vec<CommitInfo>> {
        let output = self.run_git_command(&[
            "log",
            &format!("-{}", self.max_recent_commits),
            "--pretty=format:%H|%h|%s|%an|%ai",
        ])?;
        Ok(self.parse_git_log(&output))
    }

    fn get_all_files(&self) -> Result<Vec<PathBuf>> {
        let output = self.run_git_command(&["ls-files"])?;
        Ok(output
            .lines()
            .map(|line| self.project_dir.join(line))
            .collect())
    }

    fn run_git_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.project_dir)
            .args(args)
            .output()
            .map_err(|e| HiveCodeError::Internal(format!("Failed to run git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Git command failed: {}", stderr);
            return Err(HiveCodeError::Internal(format!(
                "Git command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn parse_git_diff(&self, output: &str) -> Vec<FileChange> {
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() < 2 {
                    return None;
                }

                let change_type = match parts[0] {
                    "A" => ChangeType::Added,
                    "M" => ChangeType::Modified,
                    "D" => ChangeType::Deleted,
                    "R" => ChangeType::Renamed {
                        from: String::new(),
                    },
                    "C" => ChangeType::Copied,
                    _ => return None,
                };

                Some(FileChange {
                    path: PathBuf::from(parts[1]),
                    change_type,
                    additions: 0,
                    deletions: 0,
                    diff_preview: None,
                })
            })
            .collect()
    }

    fn parse_git_log(&self, output: &str) -> Vec<CommitInfo> {
        output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() < 5 {
                    return None;
                }

                Some(CommitInfo {
                    hash: parts[0].to_string(),
                    short_hash: parts[1].to_string(),
                    message: parts[2].to_string(),
                    author: parts[3].to_string(),
                    timestamp: parts[4].to_string(),
                    files_changed: Vec::new(),
                })
            })
            .collect()
    }

    fn score_relevance(
        &self,
        file: &Path,
        modified_files: &[&FileChange],
        project_dir: &Path,
    ) -> (f64, RelevanceReason) {
        let file_name = file.file_name().unwrap_or_default().to_string_lossy();

        // Check if currently modified
        if modified_files.iter().any(|f| f.path == file) {
            return (1.0, RelevanceReason::CurrentlyModified);
        }

        // Check for config files
        if is_config_file(&file_name) {
            return (0.8, RelevanceReason::ConfigFile);
        }

        // Check if in same module as modified files
        if let Some(parent) = file.parent() {
            if modified_files.iter().any(|f| {
                f.path
                    .parent()
                    .map(|p| p == parent)
                    .unwrap_or(false)
            }) {
                return (0.6, RelevanceReason::InSameModule);
            }
        }

        // Check for test files
        if is_test_file(&file_name) {
            if modified_files.iter().any(|f| {
                let test_name = file_name.replace("_test", "").replace("_spec", "");
                f.path.file_name().unwrap_or_default().to_string_lossy().contains(&test_name)
            }) {
                return (0.7, RelevanceReason::TestForModified);
            }
        }

        // Check for name patterns
        for modified in modified_files {
            if let Some(modified_name) = modified.path.file_name() {
                if file_name.contains(&modified_name.to_string_lossy().replace(".rs", "")) {
                    return (0.4, RelevanceReason::RelatedByName);
                }
            }
        }

        // Recently modified
        if let Ok(metadata) = std::fs::metadata(project_dir.join(file)) {
            if let Ok(modified_time) = metadata.modified() {
                if let Ok(elapsed) = modified_time.elapsed() {
                    if elapsed.as_secs() < 86400 {
                        // Less than a day old
                        return (0.3, RelevanceReason::RecentlyModified);
                    }
                }
            }
        }

        (0.0, RelevanceReason::RelatedByName)
    }

    fn detect_language(&self, path: &Path) -> Option<String> {
        match path.extension()?.to_str()? {
            "rs" => Some("Rust".to_string()),
            "ts" | "tsx" => Some("TypeScript".to_string()),
            "js" | "jsx" => Some("JavaScript".to_string()),
            "py" => Some("Python".to_string()),
            "go" => Some("Go".to_string()),
            "java" => Some("Java".to_string()),
            "cpp" | "cc" | "cxx" => Some("C++".to_string()),
            "c" => Some("C".to_string()),
            "rb" => Some("Ruby".to_string()),
            "toml" => Some("TOML".to_string()),
            "json" => Some("JSON".to_string()),
            "yaml" | "yml" => Some("YAML".to_string()),
            "md" => Some("Markdown".to_string()),
            _ => None,
        }
    }
}

fn is_config_file(name: &str) -> bool {
    matches!(
        name,
        "Cargo.toml"
            | "Cargo.lock"
            | "package.json"
            | "package-lock.json"
            | "tsconfig.json"
            | "Makefile"
            | ".gitignore"
            | ".env"
            | "README.md"
    )
}

fn is_test_file(name: &str) -> bool {
    name.contains("_test") || name.contains("_spec") || name.ends_with("test.rs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        let builder = GitContextBuilder::new(PathBuf::from("."));
        assert_eq!(builder.detect_language(Path::new("test.rs")), Some("Rust".to_string()));
        assert_eq!(builder.detect_language(Path::new("app.ts")), Some("TypeScript".to_string()));
        assert_eq!(builder.detect_language(Path::new("script.py")), Some("Python".to_string()));
        assert_eq!(builder.detect_language(Path::new("unknown.xyz")), None);
    }

    #[test]
    fn test_parse_git_diff() {
        let builder = GitContextBuilder::new(PathBuf::from("."));
        let output = "M\tsrc/main.rs\nA\tsrc/lib.rs\nD\tsrc/old.rs";
        let changes = builder.parse_git_diff(output);

        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0].path, PathBuf::from("src/main.rs"));
        assert_eq!(changes[0].change_type, ChangeType::Modified);
        assert_eq!(changes[1].change_type, ChangeType::Added);
        assert_eq!(changes[2].change_type, ChangeType::Deleted);
    }

    #[test]
    fn test_parse_git_log() {
        let builder = GitContextBuilder::new(PathBuf::from("."));
        let output = "abc123|abc12|Initial commit|John Doe|2024-01-01T00:00:00Z";
        let commits = builder.parse_git_log(output);

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].hash, "abc123");
        assert_eq!(commits[0].short_hash, "abc12");
        assert_eq!(commits[0].message, "Initial commit");
    }

    #[test]
    fn test_is_config_file() {
        assert!(is_config_file("Cargo.toml"));
        assert!(is_config_file("package.json"));
        assert!(!is_config_file("main.rs"));
    }

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file("main_test.rs"));
        assert!(is_test_file("util_spec.rs"));
        assert!(is_test_file("test_utils.rs"));
        assert!(!is_test_file("main.rs"));
    }

    #[test]
    fn test_change_type_equality() {
        assert_eq!(ChangeType::Added, ChangeType::Added);
        assert_eq!(ChangeType::Modified, ChangeType::Modified);
        assert_ne!(ChangeType::Added, ChangeType::Modified);
    }

    #[test]
    fn test_relevance_reason_display() {
        let reason = RelevanceReason::CurrentlyModified;
        // Just verify it's constructible and serializable
        let _json = serde_json::to_string(&reason).unwrap();
    }

    #[test]
    fn test_git_context_builder_defaults() {
        let builder = GitContextBuilder::new(PathBuf::from("/tmp"));
        assert_eq!(builder.max_recent_commits, 10);
        assert_eq!(builder.max_relevant_files, 20);
        assert_eq!(builder.include_untracked, true);
    }

    #[test]
    fn test_git_context_builder_with_settings() {
        let builder = GitContextBuilder::new(PathBuf::from("/tmp"))
            .with_max_commits(5)
            .with_max_files(15)
            .with_include_untracked(false);

        assert_eq!(builder.max_recent_commits, 5);
        assert_eq!(builder.max_relevant_files, 15);
        assert_eq!(builder.include_untracked, false);
    }

    #[test]
    fn test_context_string_building() {
        let info = GitContextInfo {
            branch: "main".to_string(),
            has_uncommitted_changes: false,
            modified_files: vec![],
            staged_files: vec![],
            untracked_files: vec![],
            recent_commits: vec![],
            relevant_files: vec![],
        };

        let context = GitContextBuilder::build_context_string(&info, 100);
        assert!(context.contains("main"));
        assert!(context.contains("Git Context"));
    }

    #[test]
    fn test_file_change_serialization() {
        let change = FileChange {
            path: PathBuf::from("src/main.rs"),
            change_type: ChangeType::Modified,
            additions: 10,
            deletions: 5,
            diff_preview: Some("- old line\n+ new line".to_string()),
        };

        let json = serde_json::to_string(&change).unwrap();
        let restored: FileChange = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.path, change.path);
        assert_eq!(restored.additions, 10);
    }
}

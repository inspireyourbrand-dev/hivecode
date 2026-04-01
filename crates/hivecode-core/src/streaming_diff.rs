//! Streaming diff view for HiveCode
//!
//! Tracks file state before/after tool execution and generates
//! real-time diff events as the AI modifies files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, warn};

/// A single diff event for streaming to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEvent {
    /// Path to the file being modified
    pub file_path: PathBuf,
    /// Type of diff event
    pub event_type: DiffEventType,
    /// Unix milliseconds timestamp
    pub timestamp: u64,
    /// Optional line number for line-specific events
    pub line_number: Option<usize>,
    /// Optional content (for added lines, changes, etc.)
    pub content: Option<String>,
}

/// Type of diff event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiffEventType {
    /// File opened for tracking
    FileOpened,
    /// Single line added
    LineAdded(String),
    /// Single line removed
    LineRemoved(String),
    /// Line modified
    LineModified { old: String, new: String },
    /// Block of lines added
    BlockAdded { start_line: usize, content: String },
    /// Block of lines removed
    BlockRemoved { start_line: usize, line_count: usize },
    /// File created
    FileCreated,
    /// File deleted
    FileDeleted,
    /// Diff tracking complete
    FileClosed,
}

/// Represents a unified diff between two file versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    /// Path to the file
    pub path: PathBuf,
    /// Content before modification (None if created)
    pub before: Option<String>,
    /// Content after modification (None if deleted)
    pub after: Option<String>,
    /// Hunks showing changes
    pub hunks: Vec<DiffHunk>,
    /// Number of lines added
    pub additions: usize,
    /// Number of lines deleted
    pub deletions: usize,
    /// Whether this is a binary file
    pub is_binary: bool,
}

/// A contiguous section of changes in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Starting line in original file
    pub old_start: usize,
    /// Number of lines in original
    pub old_count: usize,
    /// Starting line in modified file
    pub new_start: usize,
    /// Number of lines in modified
    pub new_count: usize,
    /// Individual diff lines
    pub lines: Vec<DiffLine>,
}

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLine {
    /// Unchanged line
    Context(String),
    /// Added line
    Added(String),
    /// Removed line
    Removed(String),
}

/// Statistics about diffs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffStats {
    /// Number of files modified
    pub files_modified: usize,
    /// Number of files created
    pub files_created: usize,
    /// Number of files deleted
    pub files_deleted: usize,
    /// Total lines added
    pub total_additions: usize,
    /// Total lines deleted
    pub total_deletions: usize,
}

/// Tracks file changes for diffing
pub struct DiffTracker {
    /// Snapshots of file content before modification
    snapshots: HashMap<PathBuf, String>,
    /// Computed diffs waiting to be processed
    pending_diffs: Vec<FileDiff>,
    /// All events generated
    events: Vec<DiffEvent>,
}

impl DiffTracker {
    /// Create a new diff tracker
    pub fn new() -> Self {
        Self {
            snapshots: HashMap::new(),
            pending_diffs: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Capture file state before a tool modifies it
    pub async fn capture_before(&mut self, path: &Path) -> Result<(), String> {
        match fs::read_to_string(path).await {
            Ok(content) => {
                self.snapshots.insert(path.to_path_buf(), content);
                self.events.push(DiffEvent {
                    file_path: path.to_path_buf(),
                    event_type: DiffEventType::FileOpened,
                    timestamp: Self::current_timestamp(),
                    line_number: None,
                    content: None,
                });
                debug!("Captured before state for {:?}", path);
                Ok(())
            }
            Err(e) => {
                warn!("Could not capture file {:?}: {}", path, e);
                Err(e.to_string())
            }
        }
    }

    /// Capture file state after modification and compute diff
    pub async fn capture_after(&mut self, path: &Path) -> Result<FileDiff, String> {
        let after = fs::read_to_string(path)
            .await
            .map_err(|e| e.to_string())?;

        let before = self.snapshots.get(path).cloned();
        let diff = Self::compute_diff(before.as_deref(), Some(&after), path);

        self.pending_diffs.push(diff.clone());
        self.events.push(DiffEvent {
            file_path: path.to_path_buf(),
            event_type: DiffEventType::FileClosed,
            timestamp: Self::current_timestamp(),
            line_number: None,
            content: None,
        });

        Ok(diff)
    }

    /// Compute unified diff between two strings
    pub fn compute_diff(before: Option<&str>, after: Option<&str>, path: &Path) -> FileDiff {
        let (before_lines, after_lines) = match (before, after) {
            (Some(b), Some(a)) => {
                (b.lines().collect::<Vec<_>>(), a.lines().collect::<Vec<_>>())
            }
            (None, Some(a)) => {
                // File created
                let after_lines = a.lines().collect::<Vec<_>>();
                let additions = after_lines.len();
                return FileDiff {
                    path: path.to_path_buf(),
                    before: None,
                    after: Some(a.to_string()),
                    hunks: vec![],
                    additions,
                    deletions: 0,
                    is_binary: Self::is_binary(a),
                };
            }
            (Some(b), None) => {
                // File deleted
                let before_lines = b.lines().collect::<Vec<_>>();
                let deletions = before_lines.len();
                return FileDiff {
                    path: path.to_path_buf(),
                    before: Some(b.to_string()),
                    after: None,
                    hunks: vec![],
                    additions: 0,
                    deletions,
                    is_binary: Self::is_binary(b),
                };
            }
            (None, None) => {
                return FileDiff {
                    path: path.to_path_buf(),
                    before: None,
                    after: None,
                    hunks: vec![],
                    additions: 0,
                    deletions: 0,
                    is_binary: false,
                };
            }
        };

        // Compute diff using simple LCS-based algorithm
        let diff_lines = Self::lcs_diff(&before_lines, &after_lines);

        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut additions = 0;
        let mut deletions = 0;
        let mut old_line = 1;
        let mut new_line = 1;

        for line in &diff_lines {
            match line {
                DiffLine::Context(_) => {
                    old_line += 1;
                    new_line += 1;

                    if let Some(mut hunk) = current_hunk.take() {
                        hunks.push(hunk);
                    }
                }
                DiffLine::Added(_) => {
                    additions += 1;
                    new_line += 1;

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(line.clone());
                        hunk.new_count += 1;
                    } else {
                        current_hunk = Some(DiffHunk {
                            old_start: old_line,
                            old_count: 0,
                            new_start: new_line,
                            new_count: 1,
                            lines: vec![line.clone()],
                        });
                    }
                }
                DiffLine::Removed(_) => {
                    deletions += 1;
                    old_line += 1;

                    if let Some(ref mut hunk) = current_hunk {
                        hunk.lines.push(line.clone());
                        hunk.old_count += 1;
                    } else {
                        current_hunk = Some(DiffHunk {
                            old_start: old_line,
                            old_count: 1,
                            new_start: new_line,
                            new_count: 0,
                            lines: vec![line.clone()],
                        });
                    }
                }
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        FileDiff {
            path: path.to_path_buf(),
            before: Some(before.unwrap_or("").to_string()),
            after: Some(after.unwrap_or("").to_string()),
            hunks,
            additions,
            deletions,
            is_binary: Self::is_binary(after.unwrap_or("")),
        }
    }

    /// Generate diff events for streaming to frontend
    pub fn generate_events(&self, diff: &FileDiff) -> Vec<DiffEvent> {
        let mut events = Vec::new();

        for hunk in &diff.hunks {
            for (i, line) in hunk.lines.iter().enumerate() {
                let event = match line {
                    DiffLine::Added(content) => DiffEvent {
                        file_path: diff.path.clone(),
                        event_type: DiffEventType::LineAdded(content.clone()),
                        timestamp: Self::current_timestamp(),
                        line_number: Some(hunk.new_start + i),
                        content: Some(content.clone()),
                    },
                    DiffLine::Removed(content) => DiffEvent {
                        file_path: diff.path.clone(),
                        event_type: DiffEventType::LineRemoved(content.clone()),
                        timestamp: Self::current_timestamp(),
                        line_number: Some(hunk.old_start + i),
                        content: Some(content.clone()),
                    },
                    DiffLine::Context(_) => continue,
                };
                events.push(event);
            }
        }

        events
    }

    /// Format diff as unified diff string (like `diff -u`)
    pub fn format_unified(diff: &FileDiff) -> String {
        let mut output = String::new();

        output.push_str(&format!("--- {:?}\n", diff.path));
        output.push_str(&format!("+++ {:?}\n", diff.path));

        for hunk in &diff.hunks {
            output.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count
            ));

            for line in &hunk.lines {
                match line {
                    DiffLine::Context(content) => output.push_str(&format!(" {}\n", content)),
                    DiffLine::Added(content) => output.push_str(&format!("+{}\n", content)),
                    DiffLine::Removed(content) => output.push_str(&format!("-{}\n", content)),
                }
            }
        }

        output
    }

    /// Format diff as side-by-side view
    pub fn format_side_by_side(diff: &FileDiff, width: usize) -> String {
        let col_width = width / 2 - 2;
        let mut output = String::new();

        output.push_str("LEFT (before)");
        for _ in 0..(col_width.saturating_sub(12)) {
            output.push(' ');
        }
        output.push_str("| RIGHT (after)\n");
        output.push_str(&"-".repeat(width));
        output.push('\n');

        let before_lines = diff
            .before
            .as_ref()
            .map(|b| b.lines().collect::<Vec<_>>())
            .unwrap_or_default();
        let after_lines = diff
            .after
            .as_ref()
            .map(|a| a.lines().collect::<Vec<_>>())
            .unwrap_or_default();

        let max_lines = before_lines.len().max(after_lines.len());
        for i in 0..max_lines {
            let left = before_lines.get(i).copied().unwrap_or("");
            let right = after_lines.get(i).copied().unwrap_or("");

            let left_str = if left.len() > col_width {
                format!("{}...", &left[..col_width - 3])
            } else {
                left.to_string()
            };

            let right_str = if right.len() > col_width {
                format!("{}...", &right[..col_width - 3])
            } else {
                right.to_string()
            };

            output.push_str(&format!(
                "{:<width$}| {}\n",
                left_str,
                right_str,
                width = col_width + 2
            ));
        }

        output
    }

    /// Get all pending diffs
    pub fn get_pending(&self) -> &[FileDiff] {
        &self.pending_diffs
    }

    /// Clear all snapshots and diffs
    pub fn reset(&mut self) {
        self.snapshots.clear();
        self.pending_diffs.clear();
        self.events.clear();
    }

    /// Get total statistics
    pub fn stats(&self) -> DiffStats {
        let mut stats = DiffStats::default();

        for diff in &self.pending_diffs {
            if diff.before.is_some() && diff.after.is_some() {
                stats.files_modified += 1;
            } else if diff.before.is_none() {
                stats.files_created += 1;
            } else {
                stats.files_deleted += 1;
            }

            stats.total_additions += diff.additions;
            stats.total_deletions += diff.deletions;
        }

        stats
    }

    /// Simple LCS-based diff algorithm
    fn lcs_diff(before_lines: &[&str], after_lines: &[&str]) -> Vec<DiffLine> {
        // Simple approach: compare line by line
        // For production, a more sophisticated algorithm (Myers' diff) would be better
        let mut result = Vec::new();

        let mut i = 0;
        let mut j = 0;

        while i < before_lines.len() || j < after_lines.len() {
            if i >= before_lines.len() {
                // Rest of after_lines are additions
                result.push(DiffLine::Added(after_lines[j].to_string()));
                j += 1;
            } else if j >= after_lines.len() {
                // Rest of before_lines are deletions
                result.push(DiffLine::Removed(before_lines[i].to_string()));
                i += 1;
            } else if before_lines[i] == after_lines[j] {
                // Lines match
                result.push(DiffLine::Context(before_lines[i].to_string()));
                i += 1;
                j += 1;
            } else {
                // Lines differ - check if next lines match
                if i + 1 < before_lines.len() && before_lines[i + 1] == after_lines[j] {
                    result.push(DiffLine::Removed(before_lines[i].to_string()));
                    i += 1;
                } else if j + 1 < after_lines.len() && before_lines[i] == after_lines[j + 1] {
                    result.push(DiffLine::Added(after_lines[j].to_string()));
                    j += 1;
                } else {
                    // Default: treat as removal then addition
                    result.push(DiffLine::Removed(before_lines[i].to_string()));
                    i += 1;
                }
            }
        }

        result
    }

    /// Check if content is likely binary
    fn is_binary(content: &str) -> bool {
        // Simple heuristic: check for null bytes
        content.contains('\0')
    }

    /// Get current timestamp in milliseconds
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

impl Default for DiffTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_diff_tracker_new() {
        let tracker = DiffTracker::new();
        assert_eq!(tracker.snapshots.len(), 0);
        assert_eq!(tracker.pending_diffs.len(), 0);
    }

    #[test]
    fn test_compute_diff_identical() {
        let before = "line 1\nline 2\nline 3";
        let after = "line 1\nline 2\nline 3";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), Some(after), path);

        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 0);
        assert!(diff.hunks.is_empty());
    }

    #[test]
    fn test_compute_diff_file_created() {
        let after = "new line 1\nnew line 2";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(None, Some(after), path);

        assert_eq!(diff.additions, 2);
        assert_eq!(diff.deletions, 0);
        assert!(diff.before.is_none());
        assert!(diff.after.is_some());
    }

    #[test]
    fn test_compute_diff_file_deleted() {
        let before = "old line 1\nold line 2";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), None, path);

        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 2);
        assert!(diff.before.is_some());
        assert!(diff.after.is_none());
    }

    #[test]
    fn test_compute_diff_with_changes() {
        let before = "line 1\nline 2\nline 3";
        let after = "line 1\nmodified line 2\nline 3\nline 4";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), Some(after), path);

        assert!(diff.additions > 0 || diff.deletions > 0);
    }

    #[test]
    fn test_format_unified() {
        let before = "line 1\nline 2";
        let after = "line 1\nmodified";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), Some(after), path);
        let formatted = DiffTracker::format_unified(&diff);

        assert!(formatted.contains("---"));
        assert!(formatted.contains("+++"));
        assert!(formatted.contains("@@"));
    }

    #[test]
    fn test_format_side_by_side() {
        let before = "line 1\nline 2";
        let after = "line 1\nmodified";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), Some(after), path);
        let formatted = DiffTracker::format_side_by_side(&diff, 80);

        assert!(formatted.contains("LEFT"));
        assert!(formatted.contains("RIGHT"));
    }

    #[test]
    fn test_is_binary() {
        assert!(DiffTracker::is_binary("binary\0content"));
        assert!(!DiffTracker::is_binary("text content"));
    }

    #[test]
    fn test_generate_events() {
        let before = "line 1\nline 2";
        let after = "line 1\nmodified\nline 2";
        let path = Path::new("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), Some(after), path);
        let events = DiffTracker::generate_events(&diff);

        // Should have some events for the changes
        assert!(!events.is_empty());
    }

    #[test]
    fn test_diff_stats() {
        let mut tracker = DiffTracker::new();

        let before = "content";
        let after = "modified content\nnew line";
        let path = PathBuf::from("test.txt");

        let diff = DiffTracker::compute_diff(Some(before), Some(after), &path);
        tracker.pending_diffs.push(diff);

        let stats = tracker.stats();
        assert_eq!(stats.files_modified, 1);
        assert!(stats.total_additions > 0 || stats.total_deletions > 0);
    }

    #[test]
    fn test_diff_reset() {
        let mut tracker = DiffTracker::new();
        tracker.snapshots.insert(PathBuf::from("test.txt"), "content".to_string());

        assert_eq!(tracker.snapshots.len(), 1);
        tracker.reset();
        assert_eq!(tracker.snapshots.len(), 0);
        assert_eq!(tracker.pending_diffs.len(), 0);
    }

    #[test]
    fn test_lcs_diff_simple() {
        let before = vec!["a", "b", "c"];
        let after = vec!["a", "b", "c"];

        let result = DiffTracker::lcs_diff(&before, &after);

        // All should be context
        assert!(result.iter().all(|l| matches!(l, DiffLine::Context(_))));
    }

    #[test]
    fn test_lcs_diff_addition() {
        let before = vec!["a", "b"];
        let after = vec!["a", "b", "c"];

        let result = DiffTracker::lcs_diff(&before, &after);

        assert!(result.iter().any(|l| matches!(l, DiffLine::Added(_))));
    }

    #[test]
    fn test_lcs_diff_removal() {
        let before = vec!["a", "b", "c"];
        let after = vec!["a", "b"];

        let result = DiffTracker::lcs_diff(&before, &after);

        assert!(result.iter().any(|l| matches!(l, DiffLine::Removed(_))));
    }

    #[test]
    fn test_diff_event_serialization() {
        let event = DiffEvent {
            file_path: PathBuf::from("test.txt"),
            event_type: DiffEventType::LineAdded("new line".to_string()),
            timestamp: 1234567890,
            line_number: Some(5),
            content: Some("new line".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: DiffEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.file_path, event.file_path);
        assert_eq!(restored.timestamp, event.timestamp);
        assert_eq!(restored.line_number, Some(5));
    }

    #[test]
    fn test_file_diff_serialization() {
        let diff = FileDiff {
            path: PathBuf::from("test.txt"),
            before: Some("before".to_string()),
            after: Some("after".to_string()),
            hunks: vec![],
            additions: 1,
            deletions: 1,
            is_binary: false,
        };

        let json = serde_json::to_string(&diff).unwrap();
        let restored: FileDiff = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.path, diff.path);
        assert_eq!(restored.additions, 1);
        assert_eq!(restored.deletions, 1);
    }

    #[test]
    fn test_empty_both_files() {
        let path = Path::new("test.txt");
        let diff = DiffTracker::compute_diff(None, None, path);

        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 0);
    }
}

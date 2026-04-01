//! Tauri IPC commands for file diff tracking and visualization
//!
//! These commands track file changes during a session and provide
//! structured diff data for visualization in the UI.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info};

use crate::state::TauriAppState;

/// A single line in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: String, // "add" | "remove" | "context"
    pub content: String,
    pub line_number: Option<i32>,
}

/// A hunk (section) of changes in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub id: String,
    pub old_start: i32,
    pub new_start: i32,
    pub old_lines: i32,
    pub new_lines: i32,
    pub lines: Vec<DiffLine>,
}

/// Changes to a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub filename: String,
    pub additions: i32,
    pub deletions: i32,
    pub hunks: Vec<DiffHunk>,
    pub is_streaming: bool,
}

/// Capture a file's current state before changes
///
/// Stores the current content of a file so that later changes can be diffed.
#[tauri::command]
pub async fn capture_file_before(
    state: State<'_, TauriAppState>,
    file_path: String,
) -> Result<String, String> {
    debug!("capture_file_before command received: file_path={}", file_path);

    if file_path.is_empty() {
        return Err("file_path cannot be empty".to_string());
    }

    // Placeholder: In production, would read file and store snapshot
    let snapshot_id = format!("before-{}", chrono::Utc::now().timestamp_millis());

    info!("Captured file before state: {}", file_path);
    Ok(snapshot_id)
}

/// Capture a file's current state after changes
///
/// Stores the current content of a file for comparison with a previous state.
#[tauri::command]
pub async fn capture_file_after(
    state: State<'_, TauriAppState>,
    file_path: String,
) -> Result<String, String> {
    debug!("capture_file_after command received: file_path={}", file_path);

    if file_path.is_empty() {
        return Err("file_path cannot be empty".to_string());
    }

    // Placeholder: In production, would read file and store snapshot
    let snapshot_id = format!("after-{}", chrono::Utc::now().timestamp_millis());

    info!("Captured file after state: {}", file_path);
    Ok(snapshot_id)
}

/// Get all pending file diffs for the current session
///
/// Returns structured diffs for all modified files that haven't been committed/saved.
/// This is useful for showing users what changed during the AI interaction.
#[tauri::command]
pub async fn get_pending_diffs(
    state: State<'_, TauriAppState>,
) -> Result<Vec<FileDiff>, String> {
    debug!("get_pending_diffs command received");

    // Placeholder: In production, would calculate diffs from captured snapshots
    let diffs: Vec<FileDiff> = vec![];

    info!("Retrieved {} pending diffs", diffs.len());
    Ok(diffs)
}

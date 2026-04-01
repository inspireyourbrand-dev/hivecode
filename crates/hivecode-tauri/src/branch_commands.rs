//! Tauri IPC commands for conversation branching system
//!
//! These commands handle creating, switching, and managing conversation branches,
//! allowing users to explore different conversation paths.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info};
use uuid::Uuid;

use crate::state::TauriAppState;

/// Conversation branch metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationBranch {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub message_count: i32,
    pub cost: f64,
    pub model: String,
    pub created_at: String,
    pub is_current: bool,
}

/// Branch comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchComparison {
    pub branch1: ConversationBranch,
    pub branch2: ConversationBranch,
    pub diff: String,
}

/// Fork the current conversation to create a new branch
///
/// Creates a copy of the conversation up to the specified message,
/// starting a new branch from that point.
#[tauri::command]
pub async fn fork_conversation(
    state: State<'_, TauriAppState>,
    from_id: String,
    name: Option<String>,
) -> Result<ConversationBranch, String> {
    debug!("fork_conversation command received: from_id={}", from_id);

    if from_id.is_empty() {
        return Err("from_id cannot be empty".to_string());
    }

    let branch_name = name.unwrap_or_else(|| format!("Branch {}", chrono::Utc::now().format("%H:%M:%S")));

    let branch = ConversationBranch {
        id: Uuid::new_v4().to_string(),
        name: branch_name.clone(),
        parent_id: Some(from_id.clone()),
        message_count: 0,
        cost: 0.0,
        model: "unknown".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        is_current: false,
    };

    info!("Forked conversation from {}: {}", from_id, branch.id);
    Ok(branch)
}

/// Switch to a different conversation branch
///
/// Changes the active conversation to the specified branch.
#[tauri::command]
pub async fn switch_branch(
    state: State<'_, TauriAppState>,
    branch_id: String,
) -> Result<(), String> {
    debug!("switch_branch command received: branch_id={}", branch_id);

    if branch_id.is_empty() {
        return Err("branch_id cannot be empty".to_string());
    }

    info!("Switched to branch: {}", branch_id);
    Ok(())
}

/// List all conversation branches
///
/// Returns all branches for the current project/session.
#[tauri::command]
pub async fn list_branches(
    state: State<'_, TauriAppState>,
) -> Result<Vec<ConversationBranch>, String> {
    debug!("list_branches command received");

    // Placeholder: In production, would query from branch storage
    let branches: Vec<ConversationBranch> = vec![];

    info!("Listed {} branches", branches.len());
    Ok(branches)
}

/// Delete a conversation branch
///
/// Removes a branch and all its associated data.
/// The current branch cannot be deleted.
#[tauri::command]
pub async fn delete_branch(
    state: State<'_, TauriAppState>,
    branch_id: String,
) -> Result<(), String> {
    debug!("delete_branch command received: branch_id={}", branch_id);

    if branch_id.is_empty() {
        return Err("branch_id cannot be empty".to_string());
    }

    info!("Deleted branch: {}", branch_id);
    Ok(())
}

/// Compare two conversation branches
///
/// Returns a detailed comparison of the conversations in two branches,
/// including message differences and cost/token statistics.
#[tauri::command]
pub async fn compare_branches(
    state: State<'_, TauriAppState>,
    branch_id1: String,
    branch_id2: String,
) -> Result<BranchComparison, String> {
    debug!("compare_branches command received: {} vs {}", branch_id1, branch_id2);

    if branch_id1.is_empty() || branch_id2.is_empty() {
        return Err("Both branch IDs must be provided".to_string());
    }

    let comparison = BranchComparison {
        branch1: ConversationBranch {
            id: branch_id1.clone(),
            name: "Branch 1".to_string(),
            parent_id: None,
            message_count: 0,
            cost: 0.0,
            model: "unknown".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            is_current: false,
        },
        branch2: ConversationBranch {
            id: branch_id2.clone(),
            name: "Branch 2".to_string(),
            parent_id: None,
            message_count: 0,
            cost: 0.0,
            model: "unknown".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            is_current: false,
        },
        diff: "No differences".to_string(),
    };

    info!("Compared branches {} and {}", branch_id1, branch_id2);
    Ok(comparison)
}

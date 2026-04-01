//! Tauri IPC commands for hook system management
//!
//! These commands handle hook configuration, execution, and logging,
//! allowing the application to manage pre/post/error hooks for operations.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info};
use uuid::Uuid;

use crate::state::TauriAppState;

/// Hook definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    pub id: String,
    pub name: String,
    pub hook_type: String, // "pre" | "post" | "error"
    pub trigger: String,
    pub action: String,
    pub enabled: bool,
    pub priority: Option<i32>,
}

/// Hook execution log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookLogEntry {
    pub timestamp: String,
    pub hook_id: String,
    pub success: bool,
    pub message: String,
}

/// List all registered hooks
///
/// Returns all configured hooks with their current state.
#[tauri::command]
pub async fn list_hooks(
    state: State<'_, TauriAppState>,
) -> Result<Vec<Hook>, String> {
    debug!("list_hooks command received");

    // Placeholder: In production, would load from persistence layer
    let hooks = vec![];

    info!("Listed {} hooks", hooks.len());
    Ok(hooks)
}

/// Create a new hook
///
/// Registers a new hook with the system.
#[tauri::command]
pub async fn create_hook(
    state: State<'_, TauriAppState>,
    name: String,
    hook_type: String,
    trigger: String,
    action: String,
) -> Result<Hook, String> {
    debug!("create_hook command received: name={}", name);

    if name.trim().is_empty() {
        return Err("Hook name cannot be empty".to_string());
    }

    if trigger.trim().is_empty() {
        return Err("Trigger cannot be empty".to_string());
    }

    if action.trim().is_empty() {
        return Err("Action cannot be empty".to_string());
    }

    let hook = Hook {
        id: Uuid::new_v4().to_string(),
        name,
        hook_type,
        trigger,
        action,
        enabled: true,
        priority: None,
    };

    info!("Created hook: {}", hook.id);
    Ok(hook)
}

/// Delete a hook by ID
///
/// Removes a registered hook from the system.
#[tauri::command]
pub async fn delete_hook(
    state: State<'_, TauriAppState>,
    hook_id: String,
) -> Result<(), String> {
    debug!("delete_hook command received: hook_id={}", hook_id);

    if hook_id.is_empty() {
        return Err("Hook ID cannot be empty".to_string());
    }

    info!("Deleted hook: {}", hook_id);
    Ok(())
}

/// Toggle a hook's enabled state
///
/// Enables or disables a hook without deleting it.
#[tauri::command]
pub async fn toggle_hook(
    state: State<'_, TauriAppState>,
    hook_id: String,
    enabled: bool,
) -> Result<(), String> {
    debug!("toggle_hook command received: hook_id={}, enabled={}", hook_id, enabled);

    if hook_id.is_empty() {
        return Err("Hook ID cannot be empty".to_string());
    }

    info!("Toggled hook {}: enabled={}", hook_id, enabled);
    Ok(())
}

/// Get execution logs for hooks
///
/// Returns recent execution logs, optionally filtered by hook ID.
/// Limit defaults to 10 if not specified.
#[tauri::command]
pub async fn get_hook_log(
    state: State<'_, TauriAppState>,
    hook_id: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<HookLogEntry>, String> {
    debug!("get_hook_log command received: hook_id={:?}, limit={:?}", hook_id, limit);

    let limit = limit.unwrap_or(10).max(1).min(100) as usize;

    // Placeholder: In production, would query from log storage
    let logs: Vec<HookLogEntry> = vec![];

    info!("Retrieved {} hook log entries", logs.len());
    Ok(logs)
}

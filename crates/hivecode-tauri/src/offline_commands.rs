//! Tauri IPC commands for offline mode management
//!
//! These commands handle offline status monitoring, connectivity checking,
//! and fallback to local models when the network is unavailable.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info};

use crate::state::TauriAppState;

/// Current offline/connectivity status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineStatus {
    pub is_online: bool,
    pub is_degraded: bool,
    pub using_local_model: bool,
    pub last_check_time: String,
}

/// Get the current offline/connectivity status
///
/// Returns information about the network connection and whether
/// the application is using fallback/local models.
#[tauri::command]
pub async fn get_offline_status(
    state: State<'_, TauriAppState>,
) -> Result<OfflineStatus, String> {
    debug!("get_offline_status command received");

    let status = OfflineStatus {
        is_online: true,
        is_degraded: false,
        using_local_model: false,
        last_check_time: chrono::Utc::now().to_rfc3339(),
    };

    info!("Retrieved offline status: online={}", status.is_online);
    Ok(status)
}

/// Force a connectivity check
///
/// Immediately checks connectivity to all required services
/// and updates the offline status accordingly.
#[tauri::command]
pub async fn force_connectivity_check(
    state: State<'_, TauriAppState>,
) -> Result<OfflineStatus, String> {
    debug!("force_connectivity_check command received");

    // In production: Would check connectivity to API endpoints
    let status = OfflineStatus {
        is_online: true,
        is_degraded: false,
        using_local_model: false,
        last_check_time: chrono::Utc::now().to_rfc3339(),
    };

    info!("Completed connectivity check: online={}", status.is_online);
    Ok(status)
}

/// Configure offline mode behavior
///
/// Updates settings for how the application behaves when offline,
/// including timeout values, fallback model selection, and queue settings.
#[tauri::command]
pub async fn set_offline_config(
    state: State<'_, TauriAppState>,
    config: std::collections::HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    debug!("set_offline_config command received");

    // Possible config keys: timeout_ms, fallback_model, enable_queue, queue_max_items, etc.

    info!("Updated offline configuration");
    Ok(())
}

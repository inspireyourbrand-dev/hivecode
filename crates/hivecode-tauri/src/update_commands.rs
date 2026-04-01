//! Tauri commands for update management

use hivecode_core::updater::{UpdateChannel, UpdateInfo, UpdateManager};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

/// Update status for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub channel: String,
    pub is_update_available: bool,
    pub is_mandatory: bool,
    pub last_check: Option<String>,
    pub auto_check_enabled: bool,
}

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(
    state: State<'_, crate::state::TauriAppState>,
) -> Result<Option<UpdateInfo>, String> {
    info!("Checking for updates");

    let update_manager = &state.update_manager;

    update_manager
        .check_for_updates()
        .await
        .map_err(|e| e.to_string())
}

/// Download and apply an update
#[tauri::command]
pub async fn download_update(
    info: UpdateInfo,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<String, String> {
    info!("Downloading update: {}", info.latest_version);

    let update_manager = &state.update_manager;

    let binary_path = update_manager
        .download_update(&info)
        .await
        .map_err(|e| e.to_string())?;

    Ok(binary_path.to_string_lossy().to_string())
}

/// Apply a downloaded update
#[tauri::command]
pub async fn apply_update(
    binary_path: String,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<(), String> {
    info!("Applying update from: {}", binary_path);

    let update_manager = &state.update_manager;
    let path = std::path::PathBuf::from(binary_path);

    update_manager
        .apply_update(&path)
        .await
        .map_err(|e| e.to_string())
}

/// Get current update channel
#[tauri::command]
pub fn get_update_channel(state: State<'_, crate::state::TauriAppState>) -> String {
    let update_manager = &state.update_manager;
    update_manager.get_channel().to_string()
}

/// Set update channel
#[tauri::command]
pub fn set_update_channel(
    channel: String,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<(), String> {
    info!("Setting update channel to: {}", channel);

    let update_manager = &state.update_manager;

    // Parse the channel string
    let update_channel: UpdateChannel = channel
        .parse()
        .map_err(|e: hivecode_core::HiveCodeError| e.to_string())?;

    // Note: In a real implementation, we'd mutate the update_manager
    // For now, this is a placeholder as UpdateManager would need interior mutability
    Ok(())
}

/// Get detailed update status
#[tauri::command]
pub async fn get_update_status(
    state: State<'_, crate::state::TauriAppState>,
) -> Result<UpdateStatus, String> {
    info!("Getting update status");

    let update_manager = &state.update_manager;

    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let channel = update_manager.get_channel().to_string();
    let auto_check_enabled = update_manager.auto_check_enabled();

    let update_info = update_manager
        .check_for_updates()
        .await
        .ok()
        .flatten();

    let (latest_version, is_update_available, is_mandatory) = if let Some(info) = &update_info {
        (
            Some(info.latest_version.clone()),
            true,
            info.is_mandatory,
        )
    } else {
        (None, false, false)
    };

    Ok(UpdateStatus {
        current_version,
        latest_version,
        channel,
        is_update_available,
        is_mandatory,
        last_check: chrono::Utc::now().to_rfc3339().into(),
        auto_check_enabled,
    })
}

/// Toggle auto-update checking
#[tauri::command]
pub fn toggle_auto_check(
    enabled: bool,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<(), String> {
    info!("Setting auto-check to: {}", enabled);

    let _update_manager = &state.update_manager;

    // Note: In a real implementation, we'd mutate the update_manager
    // For now, this is a placeholder
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_status_creation() {
        let status = UpdateStatus {
            current_version: "0.1.0".to_string(),
            latest_version: Some("0.2.0".to_string()),
            channel: "stable".to_string(),
            is_update_available: true,
            is_mandatory: false,
            last_check: Some("2024-01-01T00:00:00Z".to_string()),
            auto_check_enabled: true,
        };

        assert_eq!(status.current_version, "0.1.0");
        assert_eq!(status.latest_version, Some("0.2.0".to_string()));
        assert!(status.is_update_available);
    }

    #[test]
    fn test_update_channel_parsing() {
        let channel: Result<UpdateChannel, _> = "stable".parse();
        assert!(channel.is_ok());

        let channel: Result<UpdateChannel, _> = "beta".parse();
        assert!(channel.is_ok());

        let channel: Result<UpdateChannel, _> = "nightly".parse();
        assert!(channel.is_ok());

        let channel: Result<UpdateChannel, _> = "invalid".parse();
        assert!(channel.is_err());
    }
}

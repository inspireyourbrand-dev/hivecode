//! Tauri commands for plugin management

use hivecode_core::plugins::{Plugin, PluginManager};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

/// Plugin information for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: String,
    pub enabled: bool,
    pub pinned: bool,
}

impl From<Plugin> for PluginInfo {
    fn from(plugin: Plugin) -> Self {
        Self {
            id: plugin.manifest.id,
            name: plugin.manifest.name,
            version: plugin.manifest.version,
            description: plugin.manifest.description,
            author: plugin.manifest.author,
            plugin_type: plugin.manifest.plugin_type.to_string(),
            enabled: plugin.is_enabled(),
            pinned: plugin.pinned,
        }
    }
}

/// List all installed plugins
#[tauri::command]
pub async fn list_plugins(
    state: State<'_, crate::state::TauriAppState>,
) -> Result<Vec<PluginInfo>, String> {
    info!("Listing installed plugins");

    // Access the plugin manager from app state
    let plugin_manager = &state.plugin_manager;
    let plugins = plugin_manager.list_installed();

    let plugin_infos: Vec<PluginInfo> = plugins
        .into_iter()
        .map(PluginInfo::from)
        .collect();

    Ok(plugin_infos)
}

/// Install a plugin from ID or URL
#[tauri::command]
pub async fn install_plugin(
    id_or_url: String,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<PluginInfo, String> {
    info!("Installing plugin: {}", id_or_url);

    let plugin_manager = &state.plugin_manager;

    plugin_manager
        .install(&id_or_url)
        .await
        .map(PluginInfo::from)
        .map_err(|e| e.to_string())
}

/// Uninstall a plugin
#[tauri::command]
pub async fn uninstall_plugin(id: String, state: State<'_, crate::state::TauriAppState>) -> Result<(), String> {
    info!("Uninstalling plugin: {}", id);

    let plugin_manager = &state.plugin_manager;

    plugin_manager
        .uninstall(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Enable a plugin
#[tauri::command]
pub async fn enable_plugin(id: String, state: State<'_, crate::state::TauriAppState>) -> Result<(), String> {
    info!("Enabling plugin: {}", id);

    let plugin_manager = &state.plugin_manager;

    plugin_manager
        .enable(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Disable a plugin
#[tauri::command]
pub async fn disable_plugin(id: String, state: State<'_, crate::state::TauriAppState>) -> Result<(), String> {
    info!("Disabling plugin: {}", id);

    let plugin_manager = &state.plugin_manager;

    plugin_manager
        .disable(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Search the plugin registry
#[tauri::command]
pub async fn search_plugins(
    query: String,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<Vec<serde_json::Value>, String> {
    info!("Searching plugins: {}", query);

    let plugin_manager = &state.plugin_manager;

    plugin_manager
        .search_registry(&query)
        .await
        .map(|results| {
            results
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect()
        })
        .map_err(|e| e.to_string())
}

/// Get a specific plugin
#[tauri::command]
pub fn get_plugin(
    id: String,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<Option<PluginInfo>, String> {
    info!("Getting plugin: {}", id);

    let plugin_manager = &state.plugin_manager;

    Ok(plugin_manager
        .get_plugin(&id)
        .map(PluginInfo::from))
}

/// Pin/unpin a plugin as favorite
#[tauri::command]
pub async fn toggle_plugin_pinned(
    id: String,
    state: State<'_, crate::state::TauriAppState>,
) -> Result<(), String> {
    info!("Toggling plugin pin: {}", id);

    let plugin_manager = &state.plugin_manager;

    if let Some(mut plugin) = plugin_manager.get_plugin(&id) {
        plugin.pinned = !plugin.pinned;
        // In a real implementation, persist this state
        Ok(())
    } else {
        Err(format!("Plugin {} not found", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info_conversion() {
        use std::collections::HashMap;
        use std::path::PathBuf;
        use hivecode_core::plugins::{PluginManifest, PluginStatus, PluginType};

        let plugin = Plugin {
            manifest: PluginManifest {
                id: "test.plugin".to_string(),
                name: "Test Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "A test plugin".to_string(),
                author: "Test".to_string(),
                homepage: None,
                entry_point: "main.js".to_string(),
                plugin_type: PluginType::Tool,
                permissions: vec![],
                dependencies: vec![],
                min_version: None,
                metadata: HashMap::new(),
            },
            status: PluginStatus::Enabled,
            install_path: PathBuf::from("/tmp"),
            installed_at: "2024-01-01T00:00:00Z".to_string(),
            pinned: false,
            config: HashMap::new(),
        };

        let info: PluginInfo = plugin.into();
        assert_eq!(info.id, "test.plugin");
        assert_eq!(info.name, "Test Plugin");
        assert!(info.enabled);
    }
}

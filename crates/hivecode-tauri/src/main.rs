//! HiveCode Tauri main application entry point
//!
//! Initializes the Tauri application, sets up state, registers IPC commands,
//! and launches the desktop application window.

use hivecode_core::state::AppState;
use hivecode_providers::registry::ProviderRegistry;
use hivecode_security::checker::DefaultPermissionChecker;
use hivecode_tauri::commands::*;
use hivecode_tauri::compact_commands::*;
use hivecode_tauri::history_commands::*;
use hivecode_tauri::image_commands::*;
use hivecode_tauri::memory_commands::*;
use hivecode_tauri::notification_commands::*;
use hivecode_tauri::state::TauriAppState;
use hivecode_tools::registry::ToolRegistry;
use std::sync::Arc;
use tauri::Manager;
use tracing::info;
use tracing_subscriber;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("starting HiveCode application");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            info!("initializing Tauri app setup");

            // Initialize permission checker
            let permission_checker: Arc<dyn hivecode_security::checker::PermissionChecker> =
                Arc::new(DefaultPermissionChecker::new());

            // Initialize tool registry
            let tools = ToolRegistry::new(permission_checker.clone());

            // Initialize core state using the async default method
            let core_state = tauri::async_runtime::block_on(async {
                AppState::default().await
            });

            // Load configuration and initialize provider registry
            let providers = tauri::async_runtime::block_on(async {
                match hivecode_core::config::HiveConfig::load() {
                    Ok(config) => {
                        match hivecode_providers::initialize_providers(&config).await {
                            Ok(registry) => {
                                info!("Provider registry initialized with configuration");
                                registry
                            }
                            Err(e) => {
                                info!("Failed to initialize providers from config: {}, using empty registry", e);
                                ProviderRegistry::new()
                            }
                        }
                    }
                    Err(e) => {
                        info!("Failed to load config: {}, using empty registry", e);
                        ProviderRegistry::new()
                    }
                }
            });

            // Create the unified app state
            let app_state = tauri::async_runtime::block_on(async {
                TauriAppState::new(core_state, providers, tools, permission_checker).await
            });

            // Manage the state in Tauri
            app.manage(app_state);

            info!("Tauri app setup complete");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            send_message,
            get_conversation,
            list_providers,
            switch_model,
            list_tools,
            get_config,
            update_config,
            approve_permission,
            open_project,
            get_system_info,
            list_sessions,
            load_session,
            delete_session,
            export_session,
            new_session,
            search_sessions,
            save_current_conversation,
            list_memories,
            add_memory,
            delete_memory,
            search_memories,
            update_memory,
            compact_conversation,
            get_compact_status,
            process_image,
            get_image_info,
            send_notification,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::WindowEvent { label, event, .. } => {
                match event {
                    tauri::WindowEvent::CloseRequested { .. } => {
                        info!("window close requested: {}", label);
                    }
                    _ => {}
                }
            }
            tauri::RunEvent::ExitRequested { .. } => {
                info!("application exit requested");
            }
            _ => {}
        });
}

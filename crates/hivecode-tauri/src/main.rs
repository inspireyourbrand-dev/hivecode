//! HiveCode Tauri main application entry point
//!
//! Initializes the Tauri application, sets up state, registers IPC commands,
//! and launches the desktop application window.

use hivecode_core::state::AppState;
use hivecode_providers::registry::ProviderRegistry;
use hivecode_security::checker::DefaultPermissionChecker;
use hivecode_tauri::agent_commands::*;
use hivecode_tauri::auth_commands::*;
use hivecode_tauri::commands::*;
use hivecode_tauri::compact_commands::*;
use hivecode_tauri::context_commands::*;
use hivecode_tauri::history_commands::*;
use hivecode_tauri::image_commands::*;
use hivecode_tauri::memory_commands::*;
use hivecode_tauri::notification_commands::*;
use hivecode_tauri::plan_commands::*;
use hivecode_tauri::plugin_commands::*;
use hivecode_tauri::state::TauriAppState;
use hivecode_tauri::update_commands::*;
use hivecode_tauri::hooks_commands::*;
use hivecode_tauri::branch_commands::*;
use hivecode_tauri::thinking_commands::*;
use hivecode_tauri::offline_commands::*;
use hivecode_tauri::project_commands::*;
use hivecode_tauri::replay_commands::*;
use hivecode_tauri::cost_optimizer_commands::*;
use hivecode_tauri::diff_commands::*;
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
            // Core commands
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
            // Session history
            list_sessions,
            load_session,
            delete_session,
            export_session,
            new_session,
            search_sessions,
            save_current_conversation,
            // Authentication
            list_auth_profiles,
            add_api_key_profile,
            remove_auth_profile,
            set_default_profile,
            start_oauth_login,
            complete_oauth_login,
            add_chatgpt_session,
            test_auth_profile,
            get_chatgpt_login_instructions,
            // Memory
            list_memories,
            add_memory,
            delete_memory,
            search_memories,
            update_memory,
            // Conversation compaction
            compact_conversation,
            get_compact_status,
            // Image & PDF
            process_image,
            get_image_info,
            // Notifications
            send_notification,
            // Agent management
            spawn_agent,
            list_agents,
            get_agent,
            cancel_agent,
            get_agent_output,
            list_agents_by_type,
            complete_agent,
            get_running_agents_count,
            // Plan mode
            enter_plan_mode,
            exit_plan_mode,
            add_plan_step,
            update_plan_step,
            get_plan,
            is_plan_mode_active,
            cancel_plan,
            get_plan_steps,
            // Context & cost tracking
            record_token_usage,
            get_token_usage,
            get_cost_summary,
            estimate_cost,
            get_remaining_context,
            should_summarize_context,
            reset_session_usage,
            register_model,
            get_registered_models,
            get_model_limit,
            is_context_critical,
            // Plugin management
            list_plugins,
            install_plugin,
            uninstall_plugin,
            enable_plugin,
            disable_plugin,
            search_plugins,
            toggle_plugin_pinned,
            // Auto-updates
            check_for_updates,
            download_update,
            apply_update,
            get_update_status,
            // Hooks
            list_hooks,
            create_hook,
            delete_hook,
            toggle_hook,
            get_hook_log,
            // Branches
            fork_conversation,
            switch_branch,
            list_branches,
            delete_branch,
            compare_branches,
            // Thinking
            get_thinking_session,
            set_thinking_config,
            // Offline
            get_offline_status,
            force_connectivity_check,
            set_offline_config,
            // Project instructions
            load_project_instructions,
            save_project_instructions,
            get_project_instructions_template,
            // Session replay
            start_recording,
            stop_recording,
            list_recordings,
            load_recording,
            delete_recording,
            export_recording,
            // Cost optimizer
            get_cost_analysis,
            get_cost_breakdown,
            get_daily_cost_trend,
            // Diff tracking
            capture_file_before,
            capture_file_after,
            get_pending_diffs,
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

//! Application state management for HiveCode Tauri

use hivecode_core::state::AppState;
use hivecode_providers::registry::ProviderRegistry;
use hivecode_security::checker::PermissionChecker;
use hivecode_tools::registry::ToolRegistry;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Complete HiveCode application state injected into Tauri as managed state
///
/// This struct wraps all stateful components and is shared across all Tauri commands
/// via the Tauri state management system. It uses Arc<RwLock<>> for thread-safe
/// access from async command handlers.
#[derive(Clone)]
pub struct TauriAppState {
    /// Core HiveCode state (conversations, configuration, etc.)
    pub core: Arc<RwLock<AppState>>,

    /// LLM provider registry (OpenAI, Anthropic, etc.)
    pub providers: Arc<ProviderRegistry>,

    /// Tool/MCP registry for available tools and resources
    pub tools: Arc<ToolRegistry>,

    /// Security permission checker for tool execution
    pub permission_checker: Arc<dyn PermissionChecker>,
}

impl TauriAppState {
    /// Create a new application state with initialized components
    pub async fn new(
        core: AppState,
        providers: ProviderRegistry,
        tools: ToolRegistry,
        permission_checker: Arc<dyn PermissionChecker>,
    ) -> Self {
        info!("initializing TauriAppState");

        TauriAppState {
            core: Arc::new(RwLock::new(core)),
            providers: Arc::new(providers),
            tools: Arc::new(tools),
            permission_checker,
        }
    }

    /// Get read access to the core application state
    pub async fn core(&self) -> tokio::sync::RwLockReadGuard<'_, AppState> {
        self.core.read().await
    }

    /// Get write access to the core application state
    pub async fn core_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, AppState> {
        self.core.write().await
    }

    /// Get reference to provider registry
    pub fn providers(&self) -> &ProviderRegistry {
        &self.providers
    }

    /// Get reference to tool registry
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Get reference to permission checker
    pub fn permission_checker(&self) -> &dyn PermissionChecker {
        self.permission_checker.as_ref()
    }
}

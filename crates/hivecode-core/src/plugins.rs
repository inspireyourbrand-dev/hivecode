//! Plugin and extension system for HiveCode
//!
//! Provides a flexible plugin architecture supporting:
//! - Dynamic plugin discovery and loading
//! - Multiple plugin types (tools, providers, themes, skills, MCP)
//! - Plugin manifest validation
//! - Permission and capability management

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tokio::fs;
use tracing::{debug, info, warn};

/// Plugin manifest loaded from plugin.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique identifier for the plugin (e.g., "com.example.my-plugin")
    pub id: String,

    /// Human-readable plugin name
    pub name: String,

    /// Semantic version of the plugin
    pub version: String,

    /// Brief description of plugin functionality
    pub description: String,

    /// Plugin author information
    pub author: String,

    /// Optional homepage or documentation URL
    pub homepage: Option<String>,

    /// Path to the main entry point script/binary (relative to plugin directory)
    pub entry_point: String,

    /// Type of plugin determines how it's loaded and integrated
    pub plugin_type: PluginType,

    /// List of permissions the plugin requires
    #[serde(default)]
    pub permissions: Vec<String>,

    /// List of other plugin IDs this plugin depends on
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Minimum HiveCode version required
    #[serde(default)]
    pub min_version: Option<String>,

    /// Additional plugin-specific metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Classification of plugin functionality
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    /// Adds new tools to the tool registry
    Tool,

    /// Adds new LLM provider implementations
    Provider,

    /// Provides UI theme definitions
    Theme,

    /// Prompt-based skills (slash commands, quick actions)
    Skill,

    /// MCP (Model Context Protocol) server bundle
    Mcp,

    /// Other/custom plugin type
    Custom,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tool => write!(f, "tool"),
            Self::Provider => write!(f, "provider"),
            Self::Theme => write!(f, "theme"),
            Self::Skill => write!(f, "skill"),
            Self::Mcp => write!(f, "mcp"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Current state of a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is successfully installed
    Installed,

    /// Plugin is installed and enabled
    Enabled,

    /// Plugin is installed but disabled
    Disabled,

    /// Plugin encountered an error during initialization
    Error(String),
}

/// Metadata about an installed plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Plugin manifest
    pub manifest: PluginManifest,

    /// Current plugin status
    pub status: PluginStatus,

    /// Filesystem path where plugin is installed
    pub install_path: PathBuf,

    /// ISO 8601 timestamp of when plugin was installed
    pub installed_at: String,

    /// Whether the plugin is marked as pinned/favorite
    #[serde(default)]
    pub pinned: bool,

    /// Custom configuration overrides for this plugin
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

impl Plugin {
    /// Check if plugin is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.status == PluginStatus::Enabled
    }

    /// Check if plugin has an error
    pub fn has_error(&self) -> bool {
        matches!(self.status, PluginStatus::Error(_))
    }
}

/// Search result from plugin registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRegistryEntry {
    /// Plugin manifest from registry
    pub manifest: PluginManifest,

    /// Number of downloads
    pub download_count: u32,

    /// Average rating (0.0 - 5.0)
    pub rating: f32,

    /// Whether this plugin is already installed locally
    pub is_installed: bool,
}

/// Central plugin manager
pub struct PluginManager {
    /// Map of plugin ID to Plugin metadata
    plugins: RwLock<HashMap<String, Plugin>>,

    /// Plugins directory (~/.hivecode/plugins/)
    plugins_dir: PathBuf,

    /// Plugin registry API base URL
    registry_url: String,

    /// HTTP client for registry operations
    http_client: reqwest::Client,

    /// Currently enabled plugin IDs
    enabled_plugins: RwLock<Vec<String>>,
}

impl PluginManager {
    /// Create a new plugin manager and scan for installed plugins
    pub async fn new() -> Result<Self> {
        let plugins_dir = Self::default_plugins_dir()?;

        // Ensure plugins directory exists
        fs::create_dir_all(&plugins_dir).await
            .map_err(|e| HiveCodeError::Internal(format!("Failed to create plugins directory: {}", e)))?;

        let manager = Self {
            plugins: RwLock::new(HashMap::new()),
            plugins_dir,
            registry_url: "https://plugins.hivepowered.ai/api".to_string(),
            http_client: reqwest::Client::new(),
            enabled_plugins: RwLock::new(Vec::new()),
        };

        // Scan installed plugins
        manager.scan_plugins().await?;

        Ok(manager)
    }

    /// Create an empty plugin manager without scanning for plugins.
    ///
    /// This is a fallback for when the full initialization fails (e.g., home directory
    /// inaccessible or plugins directory cannot be created).
    pub fn empty() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            plugins_dir: PathBuf::from(".hivecode/plugins"),
            registry_url: "https://plugins.hivepowered.ai/api".to_string(),
            http_client: reqwest::Client::new(),
            enabled_plugins: RwLock::new(Vec::new()),
        }
    }

    /// Get the default plugins directory (~/.hivecode/plugins/)
    pub fn default_plugins_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| HiveCodeError::ConfigError("Cannot determine home directory".to_string()))?;
        Ok(home.join(".hivecode").join("plugins"))
    }

    /// Scan the plugins directory and load all installed plugins
    async fn scan_plugins(&self) -> Result<()> {
        let mut entries = fs::read_dir(&self.plugins_dir).await
            .map_err(|e| HiveCodeError::Internal(format!("Failed to scan plugins: {}", e)))?;

        let mut plugins = HashMap::new();
        let mut enabled = Vec::new();

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| HiveCodeError::Internal(format!("Error reading plugin entry: {}", e)))? {

            let path = entry.path();
            if path.is_dir() {
                if let Ok(plugin) = self.load_plugin_metadata(&path).await {
                    if plugin.is_enabled() {
                        enabled.push(plugin.manifest.id.clone());
                    }
                    debug!("Discovered plugin: {} v{}", plugin.manifest.name, plugin.manifest.version);
                    plugins.insert(plugin.manifest.id.clone(), plugin);
                }
            }
        }

        *self.plugins.write().unwrap() = plugins;
        *self.enabled_plugins.write().unwrap() = enabled;

        Ok(())
    }

    /// Load plugin metadata from a plugin directory
    async fn load_plugin_metadata(&self, plugin_dir: &Path) -> Result<Plugin> {
        let manifest_path = plugin_dir.join("plugin.toml");

        if !manifest_path.exists() {
            return Err(HiveCodeError::PluginError(
                format!("Plugin manifest not found at {:?}", manifest_path),
            ));
        }

        let manifest_str = fs::read_to_string(&manifest_path).await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to read manifest: {}", e)))?;

        let manifest: PluginManifest = toml::from_str(&manifest_str)
            .map_err(|e| HiveCodeError::PluginError(format!("Invalid manifest TOML: {}", e)))?;

        // Check if enabled
        let status_file = plugin_dir.join("status.json");
        let status = if status_file.exists() {
            let status_str = fs::read_to_string(&status_file).await.ok();
            if let Some(s) = status_str {
                serde_json::from_str(&s).unwrap_or(PluginStatus::Disabled)
            } else {
                PluginStatus::Disabled
            }
        } else {
            PluginStatus::Disabled
        };

        Ok(Plugin {
            manifest,
            status,
            install_path: plugin_dir.to_path_buf(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            pinned: false,
            config: HashMap::new(),
        })
    }

    /// Install a plugin from registry or direct URL
    pub async fn install(&self, id_or_url: &str) -> Result<Plugin> {
        info!("Installing plugin: {}", id_or_url);

        // Check if already installed
        if let Some(plugin) = self.get_plugin(id_or_url) {
            warn!("Plugin {} is already installed", id_or_url);
            return Ok(plugin.clone());
        }

        // Determine if it's a plugin ID or URL
        let (manifest, download_url) = if id_or_url.starts_with("http://") || id_or_url.starts_with("https://") {
            // Direct URL - fetch manifest from URL
            let response = self.http_client.get(id_or_url).send().await
                .map_err(|e| HiveCodeError::PluginError(format!("Failed to fetch plugin: {}", e)))?;

            let manifest_text = response.text().await
                .map_err(|e| HiveCodeError::PluginError(format!("Failed to read plugin: {}", e)))?;

            let manifest: PluginManifest = toml::from_str(&manifest_text)
                .map_err(|e| HiveCodeError::PluginError(format!("Invalid manifest: {}", e)))?;

            (manifest.clone(), id_or_url.to_string())
        } else {
            // Plugin ID - look up in registry
            let entry = self.search_registry(id_or_url).await?
                .into_iter()
                .next()
                .ok_or_else(|| HiveCodeError::PluginError(format!("Plugin {} not found in registry", id_or_url)))?;

            (entry.manifest, format!("{}/plugins/{}/download", self.registry_url, id_or_url))
        };

        // Create plugin directory
        let plugin_dir = self.plugins_dir.join(&manifest.id);
        fs::create_dir_all(&plugin_dir).await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to create plugin dir: {}", e)))?;

        // Download and extract plugin
        let response = self.http_client.get(&download_url).send().await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to download plugin: {}", e)))?;

        let content = response.bytes().await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to read plugin bytes: {}", e)))?;

        // For now, save the manifest
        let manifest_path = plugin_dir.join("plugin.toml");
        let manifest_str = toml::to_string_pretty(&manifest)
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to serialize manifest: {}", e)))?;

        fs::write(&manifest_path, manifest_str).await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to write manifest: {}", e)))?;

        // Create plugin entry
        let plugin = Plugin {
            manifest,
            status: PluginStatus::Installed,
            install_path: plugin_dir,
            installed_at: chrono::Utc::now().to_rfc3339(),
            pinned: false,
            config: HashMap::new(),
        };

        // Store plugin
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.insert(plugin.manifest.id.clone(), plugin.clone());
        }

        info!("Plugin installed successfully");
        Ok(plugin)
    }

    /// Uninstall a plugin
    pub async fn uninstall(&self, id: &str) -> Result<()> {
        info!("Uninstalling plugin: {}", id);

        let plugin = self.get_plugin(id)
            .ok_or_else(|| HiveCodeError::PluginError(format!("Plugin {} not found", id)))?;

        // Remove plugin directory
        fs::remove_dir_all(&plugin.install_path).await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to remove plugin directory: {}", e)))?;

        // Remove from registry
        {
            let mut plugins = self.plugins.write().unwrap();
            plugins.remove(id);
        }

        info!("Plugin uninstalled");
        Ok(())
    }

    /// Enable a plugin
    pub async fn enable(&self, id: &str) -> Result<()> {
        info!("Enabling plugin: {}", id);

        {
            let mut plugins = self.plugins.write().unwrap();
            if let Some(plugin) = plugins.get_mut(id) {
                plugin.status = PluginStatus::Enabled;

                // Write status file
                let status_file = plugin.install_path.join("status.json");
                let status_json = serde_json::to_string(&plugin.status)
                    .map_err(|e| HiveCodeError::PluginError(e.to_string()))?;

                std::fs::write(&status_file, status_json)
                    .map_err(|e| HiveCodeError::PluginError(format!("Failed to write status: {}", e)))?;
            } else {
                return Err(HiveCodeError::PluginError(format!("Plugin {} not found", id)));
            }
        }

        // Update enabled list
        let mut enabled = self.enabled_plugins.write().unwrap();
        if !enabled.contains(&id.to_string()) {
            enabled.push(id.to_string());
        }

        Ok(())
    }

    /// Disable a plugin
    pub async fn disable(&self, id: &str) -> Result<()> {
        info!("Disabling plugin: {}", id);

        {
            let mut plugins = self.plugins.write().unwrap();
            if let Some(plugin) = plugins.get_mut(id) {
                plugin.status = PluginStatus::Disabled;

                // Write status file
                let status_file = plugin.install_path.join("status.json");
                let status_json = serde_json::to_string(&plugin.status)
                    .map_err(|e| HiveCodeError::PluginError(e.to_string()))?;

                std::fs::write(&status_file, status_json)
                    .map_err(|e| HiveCodeError::PluginError(format!("Failed to write status: {}", e)))?;
            } else {
                return Err(HiveCodeError::PluginError(format!("Plugin {} not found", id)));
            }
        }

        // Update enabled list
        let mut enabled = self.enabled_plugins.write().unwrap();
        enabled.retain(|pid| pid != id);

        Ok(())
    }

    /// Get all installed plugins
    pub fn list_installed(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }

    /// Get installed plugins of a specific type
    pub fn list_by_type(&self, plugin_type: PluginType) -> Vec<Plugin> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .values()
            .filter(|p| p.manifest.plugin_type == plugin_type)
            .cloned()
            .collect()
    }

    /// Get currently enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().unwrap();
        let enabled = self.enabled_plugins.read().unwrap();

        enabled
            .iter()
            .filter_map(|id| plugins.get(id).cloned())
            .collect()
    }

    /// Search the plugin registry
    pub async fn search_registry(&self, query: &str) -> Result<Vec<PluginRegistryEntry>> {
        let url = format!("{}/search?q={}", self.registry_url, urlencoding::encode(query));

        let response = self.http_client.get(&url).send().await
            .map_err(|e| HiveCodeError::PluginError(format!("Registry search failed: {}", e)))?;

        let results: Vec<PluginRegistryEntry> = response.json().await
            .map_err(|e| HiveCodeError::PluginError(format!("Failed to parse registry response: {}", e)))?;

        Ok(results)
    }

    /// Get a specific installed plugin
    pub fn get_plugin(&self, id: &str) -> Option<Plugin> {
        self.plugins.read().unwrap().get(id).cloned()
    }

    /// Validate plugin manifest
    pub fn validate_manifest(manifest: &PluginManifest) -> Result<()> {
        if manifest.id.is_empty() {
            return Err(HiveCodeError::PluginError("Plugin ID cannot be empty".to_string()));
        }

        if manifest.name.is_empty() {
            return Err(HiveCodeError::PluginError("Plugin name cannot be empty".to_string()));
        }

        if manifest.version.is_empty() {
            return Err(HiveCodeError::PluginError("Plugin version cannot be empty".to_string()));
        }

        if manifest.entry_point.is_empty() {
            return Err(HiveCodeError::PluginError("Plugin entry point cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Set plugin configuration
    pub async fn set_plugin_config(&self, id: &str, key: &str, value: serde_json::Value) -> Result<()> {
        let mut plugins = self.plugins.write().unwrap();

        if let Some(plugin) = plugins.get_mut(id) {
            plugin.config.insert(key.to_string(), value);

            // Persist to file
            let config_file = plugin.install_path.join("config.json");
            let config_json = serde_json::to_string_pretty(&plugin.config)
                .map_err(|e| HiveCodeError::PluginError(e.to_string()))?;

            fs::write(&config_file, config_json).await
                .map_err(|e| HiveCodeError::PluginError(format!("Failed to write config: {}", e)))?;

            Ok(())
        } else {
            Err(HiveCodeError::PluginError(format!("Plugin {} not found", id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type_display() {
        assert_eq!(PluginType::Tool.to_string(), "tool");
        assert_eq!(PluginType::Provider.to_string(), "provider");
        assert_eq!(PluginType::Theme.to_string(), "theme");
    }

    #[test]
    fn test_plugin_status_enabled() {
        let plugin = Plugin {
            manifest: PluginManifest {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: "Test plugin".to_string(),
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
            install_path: PathBuf::from("/tmp/test"),
            installed_at: "2024-01-01T00:00:00Z".to_string(),
            pinned: false,
            config: HashMap::new(),
        };

        assert!(plugin.is_enabled());
        assert!(!plugin.has_error());
    }

    #[test]
    fn test_plugin_validation() {
        let valid_manifest = PluginManifest {
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
        };

        assert!(PluginManager::validate_manifest(&valid_manifest).is_ok());

        let invalid_manifest = PluginManifest {
            id: "".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            homepage: None,
            entry_point: "main.js".to_string(),
            plugin_type: PluginType::Tool,
            permissions: vec![],
            dependencies: vec![],
            min_version: None,
            metadata: HashMap::new(),
        };

        assert!(PluginManager::validate_manifest(&invalid_manifest).is_err());
    }
}

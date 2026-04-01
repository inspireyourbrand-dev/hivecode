//! Keybinding customization system for HiveCode
//!
//! User-configurable keybindings with validation, conflict detection,
//! and import/export. Supports both standard and vim keybinding schemes.

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// A keybinding mapping keys to a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    /// Unique keybinding identifier
    pub id: String,
    /// Key combination (e.g., ["Ctrl", "Shift", "P"])
    pub keys: Vec<String>,
    /// Command to execute
    pub command: String,
    /// Context condition (e.g., "editorFocus")
    pub when: Option<String>,
    /// Human-readable description
    pub description: String,
    /// Category for organization
    pub category: KeybindingCategory,
    /// Whether this is a default keybinding
    pub is_default: bool,
    /// Whether user has modified this binding
    pub is_user_modified: bool,
}

/// Category for organizing keybindings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeybindingCategory {
    /// Navigation commands
    Navigation,
    /// Text editing commands
    Editing,
    /// Chat-related commands
    Chat,
    /// Tool commands
    Tools,
    /// View/UI commands
    View,
    /// Session management
    Session,
    /// Debugging
    Debug,
    /// Custom category
    Custom,
}

impl std::fmt::Display for KeybindingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeybindingCategory::Navigation => write!(f, "Navigation"),
            KeybindingCategory::Editing => write!(f, "Editing"),
            KeybindingCategory::Chat => write!(f, "Chat"),
            KeybindingCategory::Tools => write!(f, "Tools"),
            KeybindingCategory::View => write!(f, "View"),
            KeybindingCategory::Session => write!(f, "Session"),
            KeybindingCategory::Debug => write!(f, "Debug"),
            KeybindingCategory::Custom => write!(f, "Custom"),
        }
    }
}

/// A keybinding scheme (set of keybindings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingScheme {
    /// Name of the scheme (e.g., "default", "vim")
    pub name: String,
    /// Keybindings in this scheme
    pub bindings: Vec<Keybinding>,
    /// Description of the scheme
    pub description: String,
}

/// Detected keybinding conflict
#[derive(Debug, Clone)]
pub struct KeyConflict {
    /// Key combination
    pub keys: Vec<String>,
    /// Commands that conflict on this key combination
    pub conflicting_commands: Vec<String>,
}

/// Keybinding manager for customization and management
pub struct KeybindingManager {
    active_scheme: Arc<RwLock<String>>,
    schemes: Arc<RwLock<HashMap<String, KeybindingScheme>>>,
    user_overrides: Arc<RwLock<Vec<Keybinding>>>,
    storage_path: PathBuf,
}

impl KeybindingManager {
    /// Create a new keybinding manager
    pub async fn new(storage_dir: Option<PathBuf>) -> Result<Self> {
        debug!("Creating new KeybindingManager");
        let storage_path = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".hivecode");
            path
        });

        let mut schemes = HashMap::new();
        schemes.insert("default".to_string(), Self::create_default_scheme());
        schemes.insert("vim".to_string(), Self::create_vim_scheme());

        Ok(Self {
            active_scheme: Arc::new(RwLock::new("default".to_string())),
            schemes: Arc::new(RwLock::new(schemes)),
            user_overrides: Arc::new(RwLock::new(Vec::new())),
            storage_path,
        })
    }

    /// Get the active keybinding scheme
    pub async fn get_active_scheme(&self) -> KeybindingScheme {
        let active = self.active_scheme.read().await;
        let schemes = self.schemes.read().await;
        schemes
            .get(active.as_str())
            .cloned()
            .unwrap_or_else(|| schemes.get("default").unwrap().clone())
    }

    /// Set the active scheme
    pub async fn set_active_scheme(&mut self, scheme_name: &str) -> Result<()> {
        debug!("Setting active scheme: {}", scheme_name);
        let schemes = self.schemes.read().await;
        if !schemes.contains_key(scheme_name) {
            return Err(HiveCodeError::NotFound(format!(
                "Keybinding scheme '{}' not found",
                scheme_name
            )));
        }
        let mut active = self.active_scheme.write().await;
        *active = scheme_name.to_string();
        self.save().await?;
        info!("Active scheme set to: {}", scheme_name);
        Ok(())
    }

    /// List available schemes
    pub async fn list_schemes(&self) -> Vec<String> {
        self.schemes
            .read()
            .await
            .keys()
            .cloned()
            .collect()
    }

    /// Get a binding by command
    pub async fn get_binding(&self, command: &str) -> Option<Keybinding> {
        let scheme = self.get_active_scheme().await;
        scheme.bindings.iter().find(|b| b.command == command).cloned()
    }

    /// Get a binding by key combination
    pub async fn get_binding_for_keys(&self, keys: &[String]) -> Option<Keybinding> {
        let scheme = self.get_active_scheme().await;
        let key_str = keys.join("+");
        scheme
            .bindings
            .iter()
            .find(|b| b.keys.join("+") == key_str)
            .cloned()
    }

    /// Set a custom keybinding
    pub async fn set_binding(&mut self, command: &str, keys: Vec<String>) -> Result<()> {
        debug!("Setting binding for command: {}", command);
        Self::validate_keys(&keys)?;

        let binding = Keybinding {
            id: uuid::Uuid::new_v4().to_string(),
            keys,
            command: command.to_string(),
            when: None,
            description: format!("Custom binding for {}", command),
            category: KeybindingCategory::Custom,
            is_default: false,
            is_user_modified: true,
        };

        let mut overrides = self.user_overrides.write().await;
        // Remove any existing override for this command
        overrides.retain(|b| b.command != command);
        overrides.push(binding);

        self.save().await?;
        Ok(())
    }

    /// Reset a binding to default
    pub async fn reset_binding(&mut self, command: &str) -> Result<()> {
        debug!("Resetting binding for command: {}", command);
        let mut overrides = self.user_overrides.write().await;
        overrides.retain(|b| b.command != command);
        self.save().await?;
        Ok(())
    }

    /// Reset all bindings to defaults
    pub async fn reset_all(&mut self) -> Result<()> {
        debug!("Resetting all bindings to default");
        let mut overrides = self.user_overrides.write().await;
        overrides.clear();
        self.save().await?;
        Ok(())
    }

    /// Check for keybinding conflicts
    pub async fn check_conflicts(&self) -> Vec<KeyConflict> {
        debug!("Checking for keybinding conflicts");
        let scheme = self.get_active_scheme().await;
        let mut key_map: HashMap<String, Vec<String>> = HashMap::new();

        for binding in &scheme.bindings {
            let key_str = binding.keys.join("+");
            key_map
                .entry(key_str)
                .or_insert_with(Vec::new)
                .push(binding.command.clone());
        }

        key_map
            .into_iter()
            .filter(|(_, commands)| commands.len() > 1)
            .map(|(keys, commands)| KeyConflict {
                keys: keys.split('+').map(|s| s.to_string()).collect(),
                conflicting_commands: commands,
            })
            .collect()
    }

    /// Validate key combination
    pub fn validate_keys(keys: &[String]) -> Result<()> {
        if keys.is_empty() {
            return Err(HiveCodeError::InvalidConfig(
                "Key combination cannot be empty".to_string(),
            ));
        }

        let valid_modifiers = ["Ctrl", "Shift", "Alt", "Cmd", "Meta"];
        let valid_keys = [
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P",
            "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z", "0", "1", "2", "3", "4", "5",
            "6", "7", "8", "9", "Enter", "Escape", "Backspace", "Tab", "Space", "Comma",
            "Period", "Slash", "Semicolon", "Quote", "Bracket", "Backslash", "Minus", "Equal",
        ];

        for (i, key) in keys.iter().enumerate() {
            if i < keys.len() - 1 {
                // All but last should be modifiers
                if !valid_modifiers.contains(&key.as_str()) {
                    return Err(HiveCodeError::InvalidConfig(format!(
                        "Invalid modifier key: {}",
                        key
                    )));
                }
            } else {
                // Last key can be any valid key
                if !valid_keys.contains(&key.as_str()) {
                    warn!("Potentially invalid key: {}", key);
                }
            }
        }

        Ok(())
    }

    /// Get all bindings
    pub async fn get_all_bindings(&self) -> Vec<Keybinding> {
        self.get_active_scheme()
            .await
            .bindings
    }

    /// Get bindings by category
    pub async fn get_bindings_by_category(
        &self,
        category: &KeybindingCategory,
    ) -> Vec<Keybinding> {
        self.get_active_scheme()
            .await
            .bindings
            .into_iter()
            .filter(|b| &b.category == category)
            .collect()
    }

    /// Export a scheme to JSON
    pub async fn export_scheme(&self, scheme_name: &str) -> Result<String> {
        debug!("Exporting scheme: {}", scheme_name);
        let schemes = self.schemes.read().await;
        let scheme = schemes
            .get(scheme_name)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Scheme {} not found", scheme_name)))?;

        serde_json::to_string_pretty(scheme)
            .map_err(|e| HiveCodeError::SerializationError(e))
    }

    /// Import a scheme from JSON
    pub async fn import_scheme(&mut self, json: &str) -> Result<String> {
        debug!("Importing scheme from JSON");
        let scheme: KeybindingScheme = serde_json::from_str(json)
            .map_err(|e| HiveCodeError::SerializationError(e))?;

        let scheme_name = scheme.name.clone();
        let mut schemes = self.schemes.write().await;
        schemes.insert(scheme_name.clone(), scheme);
        self.save().await?;
        info!("Scheme imported: {}", scheme_name);
        Ok(scheme_name)
    }

    /// Create the default keybinding scheme
    pub fn create_default_scheme() -> KeybindingScheme {
        let bindings = vec![
            Keybinding {
                id: uuid::Uuid::new_v4().to_string(),
                keys: vec!["Ctrl".to_string(), "P".to_string()],
                command: "command_palette.open".to_string(),
                when: None,
                description: "Open command palette".to_string(),
                category: KeybindingCategory::View,
                is_default: true,
                is_user_modified: false,
            },
            Keybinding {
                id: uuid::Uuid::new_v4().to_string(),
                keys: vec!["Ctrl".to_string(), "Slash".to_string()],
                command: "editor.toggleComment".to_string(),
                when: None,
                description: "Toggle comment".to_string(),
                category: KeybindingCategory::Editing,
                is_default: true,
                is_user_modified: false,
            },
            Keybinding {
                id: uuid::Uuid::new_v4().to_string(),
                keys: vec!["Ctrl".to_string(), "S".to_string()],
                command: "file.save".to_string(),
                when: None,
                description: "Save file".to_string(),
                category: KeybindingCategory::Session,
                is_default: true,
                is_user_modified: false,
            },
        ];

        KeybindingScheme {
            name: "default".to_string(),
            bindings,
            description: "Default HiveCode keybindings".to_string(),
        }
    }

    /// Create the vim keybinding scheme
    pub fn create_vim_scheme() -> KeybindingScheme {
        let bindings = vec![
            Keybinding {
                id: uuid::Uuid::new_v4().to_string(),
                keys: vec![":".to_string()],
                command: "vim.command_mode".to_string(),
                when: None,
                description: "Enter vim command mode".to_string(),
                category: KeybindingCategory::Editing,
                is_default: true,
                is_user_modified: false,
            },
            Keybinding {
                id: uuid::Uuid::new_v4().to_string(),
                keys: vec!["v".to_string()],
                command: "vim.visual_mode".to_string(),
                when: None,
                description: "Enter vim visual mode".to_string(),
                category: KeybindingCategory::Editing,
                is_default: true,
                is_user_modified: false,
            },
            Keybinding {
                id: uuid::Uuid::new_v4().to_string(),
                keys: vec!["d".to_string()],
                command: "vim.delete".to_string(),
                when: None,
                description: "Vim delete operator".to_string(),
                category: KeybindingCategory::Editing,
                is_default: true,
                is_user_modified: false,
            },
        ];

        KeybindingScheme {
            name: "vim".to_string(),
            bindings,
            description: "Vim-style keybindings".to_string(),
        }
    }

    /// Save configuration to storage
    async fn save(&self) -> Result<()> {
        debug!("Saving keybinding configuration");
        // In production, would persist to ~/.hivecode/keybindings.json
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_category_display() {
        assert_eq!(KeybindingCategory::Navigation.to_string(), "Navigation");
        assert_eq!(KeybindingCategory::Editing.to_string(), "Editing");
        assert_eq!(KeybindingCategory::Chat.to_string(), "Chat");
    }

    #[test]
    fn test_validate_keys_valid() {
        let keys = vec!["Ctrl".to_string(), "Shift".to_string(), "P".to_string()];
        assert!(KeybindingManager::validate_keys(&keys).is_ok());
    }

    #[test]
    fn test_validate_keys_empty() {
        let keys: Vec<String> = vec![];
        assert!(KeybindingManager::validate_keys(&keys).is_err());
    }

    #[test]
    fn test_validate_keys_invalid_modifier() {
        let keys = vec!["Invalid".to_string(), "P".to_string()];
        assert!(KeybindingManager::validate_keys(&keys).is_err());
    }

    #[tokio::test]
    async fn test_keybinding_manager_creation() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let schemes = manager.list_schemes().await;
        assert!(schemes.contains(&"default".to_string()));
        assert!(schemes.contains(&"vim".to_string()));
    }

    #[tokio::test]
    async fn test_get_active_scheme() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let scheme = manager.get_active_scheme().await;
        assert_eq!(scheme.name, "default");
    }

    #[tokio::test]
    async fn test_set_active_scheme() {
        let mut manager = KeybindingManager::new(None).await.unwrap();
        manager.set_active_scheme("vim").await.unwrap();
        let scheme = manager.get_active_scheme().await;
        assert_eq!(scheme.name, "vim");
    }

    #[tokio::test]
    async fn test_get_binding() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let binding = manager.get_binding("command_palette.open").await;
        assert!(binding.is_some());
    }

    #[tokio::test]
    async fn test_get_all_bindings() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let bindings = manager.get_all_bindings().await;
        assert!(!bindings.is_empty());
    }

    #[tokio::test]
    async fn test_get_bindings_by_category() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let bindings = manager
            .get_bindings_by_category(&KeybindingCategory::Editing)
            .await;
        assert!(!bindings.is_empty());
    }

    #[tokio::test]
    async fn test_check_conflicts() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let conflicts = manager.check_conflicts().await;
        // Should be no conflicts in default scheme
        assert!(conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_export_scheme() {
        let manager = KeybindingManager::new(None).await.unwrap();
        let json = manager.export_scheme("default").await.unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("default"));
    }
}

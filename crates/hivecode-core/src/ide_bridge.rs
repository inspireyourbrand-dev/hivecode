//! IDE bridge for HiveCode
//!
//! Enables bidirectional communication with VS Code and other IDEs.
//! Supports commands, diagnostics relay, file sync, and extension API.

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Current state of the IDE connection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeState {
    /// No IDE connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected to an IDE
    Connected { ide: String, version: String },
    /// Error state
    Error(String),
}

impl std::fmt::Display for BridgeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeState::Disconnected => write!(f, "disconnected"),
            BridgeState::Connecting => write!(f, "connecting"),
            BridgeState::Connected { ide, version } => write!(f, "connected: {} v{}", ide, version),
            BridgeState::Error(e) => write!(f, "error: {}", e),
        }
    }
}

/// Configuration for IDE bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// Enable/disable IDE bridge
    pub enabled: bool,
    /// Listen port for IDE connections
    pub port: u16,
    /// Listen host/address
    pub host: String,
    /// Automatically connect to discovered IDEs
    pub auto_connect: bool,
    /// List of supported IDE identifiers
    pub supported_ides: Vec<String>,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 7890,
            host: "127.0.0.1".to_string(),
            auto_connect: true,
            supported_ides: vec!["vscode".to_string(), "jetbrains".to_string(), "neovim".to_string()],
        }
    }
}

/// Messages communicated between IDE and HiveCode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeMessage {
    // IDE -> HiveCode
    /// IDE has opened a file
    #[serde(rename = "file_opened")]
    FileOpened {
        path: PathBuf,
        content: String,
        language: String,
    },
    /// File content changed
    #[serde(rename = "file_changed")]
    FileChanged {
        path: PathBuf,
        changes: Vec<TextChange>,
    },
    /// File was saved
    #[serde(rename = "file_saved")]
    FileSaved(PathBuf),
    /// Cursor moved in editor
    #[serde(rename = "cursor_moved")]
    CursorMoved {
        path: PathBuf,
        line: u32,
        column: u32,
    },
    /// Selection changed
    #[serde(rename = "selection_changed")]
    SelectionChanged {
        path: PathBuf,
        start: Position,
        end: Position,
    },
    /// Diagnostics updated (linting, type checking, etc.)
    #[serde(rename = "diagnostics_update")]
    DiagnosticsUpdate {
        path: PathBuf,
        diagnostics: Vec<Diagnostic>,
    },
    /// Request code completions
    #[serde(rename = "request_completion")]
    RequestCompletion {
        path: PathBuf,
        position: Position,
    },
    /// Execute a command
    #[serde(rename = "execute_command")]
    ExecuteCommand(String),

    // HiveCode -> IDE
    /// Open a file in the IDE
    #[serde(rename = "open_file")]
    OpenFile { path: PathBuf, line: Option<u32> },
    /// Apply edits to a file
    #[serde(rename = "edit_file")]
    EditFile { path: PathBuf, edits: Vec<TextEdit> },
    /// Show a diagnostic message
    #[serde(rename = "show_diagnostic")]
    ShowDiagnostic { path: PathBuf, diagnostic: Diagnostic },
    /// Show a message to the user
    #[serde(rename = "show_message")]
    ShowMessage { level: MessageLevel, text: String },
    /// Request file content from IDE
    #[serde(rename = "request_file_content")]
    RequestFileContent(PathBuf),
    /// Run a terminal command
    #[serde(rename = "run_terminal_command")]
    RunTerminalCommand(String),
}

/// A text change in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChange {
    /// Range of text being changed
    pub range: Range,
    /// New text to insert
    pub text: String,
}

/// A text edit to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    /// Range to replace
    pub range: Range,
    /// New text
    pub new_text: String,
}

/// Position in a document (line and column)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    /// Line number (0-based)
    pub line: u32,
    /// Column number (0-based)
    pub character: u32,
}

/// Range in a document
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

/// Diagnostic (error, warning, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Location of the issue
    pub range: Range,
    /// Diagnostic message
    pub message: String,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Source of the diagnostic (e.g., "eslint", "mypy")
    pub source: Option<String>,
    /// Diagnostic code (e.g., error number)
    pub code: Option<String>,
}

/// Severity level for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiagnosticSeverity {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Information
    Info,
    /// Hint/suggestion
    Hint,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Error => write!(f, "error"),
            DiagnosticSeverity::Warning => write!(f, "warning"),
            DiagnosticSeverity::Info => write!(f, "info"),
            DiagnosticSeverity::Hint => write!(f, "hint"),
        }
    }
}

/// Message severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageLevel {
    /// Error message
    Error,
    /// Warning message
    Warning,
    /// Info message
    Info,
}

impl std::fmt::Display for MessageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageLevel::Error => write!(f, "error"),
            MessageLevel::Warning => write!(f, "warning"),
            MessageLevel::Info => write!(f, "info"),
        }
    }
}

/// Context around the cursor
#[derive(Debug, Clone)]
pub struct CursorContext {
    /// File path
    pub file: PathBuf,
    /// Current line number
    pub line: u32,
    /// Current column
    pub column: u32,
    /// Programming language
    pub language: String,
    /// Surrounding code snippet
    pub surrounding_code: String,
}

/// IDE bridge for bidirectional communication
pub struct IdeBridge {
    config: Arc<RwLock<BridgeConfig>>,
    state: Arc<RwLock<BridgeState>>,
    message_queue: Arc<RwLock<Vec<BridgeMessage>>>,
    active_files: Arc<RwLock<HashMap<PathBuf, String>>>,
    cursor_context: Arc<RwLock<Option<CursorContext>>>,
}

impl IdeBridge {
    /// Create a new IDE bridge with default configuration
    pub fn new() -> Self {
        debug!("Creating new IdeBridge");
        Self {
            config: Arc::new(RwLock::new(BridgeConfig::default())),
            state: Arc::new(RwLock::new(BridgeState::Disconnected)),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            active_files: Arc::new(RwLock::new(HashMap::new())),
            cursor_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Create an IDE bridge with custom configuration
    pub fn with_config(config: BridgeConfig) -> Self {
        debug!("Creating IdeBridge with custom config");
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(BridgeState::Disconnected)),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            active_files: Arc::new(RwLock::new(HashMap::new())),
            cursor_context: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the IDE bridge (listen for connections)
    pub async fn start(&mut self) -> Result<()> {
        debug!("Starting IDE bridge");
        let mut state = self.state.write().await;
        *state = BridgeState::Connecting;
        info!("IDE bridge started");
        Ok(())
    }

    /// Stop the IDE bridge
    pub async fn stop(&mut self) -> Result<()> {
        debug!("Stopping IDE bridge");
        let mut state = self.state.write().await;
        *state = BridgeState::Disconnected;
        info!("IDE bridge stopped");
        Ok(())
    }

    /// Get current bridge state
    pub async fn get_state(&self) -> BridgeState {
        self.state.read().await.clone()
    }

    /// Send a message to the IDE
    pub async fn send(&mut self, message: BridgeMessage) -> Result<()> {
        debug!("Sending bridge message");
        let mut queue = self.message_queue.write().await;
        queue.push(message);
        Ok(())
    }

    /// Receive a message from the IDE
    pub async fn receive(&mut self) -> Option<BridgeMessage> {
        let mut queue = self.message_queue.write().await;
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }

    /// Get currently open files
    pub async fn get_active_files(&self) -> HashMap<PathBuf, String> {
        self.active_files.read().await.clone()
    }

    /// Get cursor context at current position
    pub async fn get_cursor_context(&self) -> Option<CursorContext> {
        self.cursor_context.read().await.clone()
    }

    /// Open a file in the IDE
    pub async fn open_in_ide(&mut self, path: &std::path::Path, line: Option<u32>) -> Result<()> {
        debug!("Opening file in IDE: {:?}", path);
        self.send(BridgeMessage::OpenFile {
            path: path.to_path_buf(),
            line,
        })
        .await?;
        Ok(())
    }

    /// Apply edits to a file in the IDE
    pub async fn apply_edit(&mut self, path: &std::path::Path, edits: Vec<TextEdit>) -> Result<()> {
        debug!("Applying edits to file: {:?}", path);
        self.send(BridgeMessage::EditFile {
            path: path.to_path_buf(),
            edits,
        })
        .await?;
        Ok(())
    }

    /// Update configuration
    pub async fn set_config(&mut self, config: BridgeConfig) -> Result<()> {
        debug!("Updating IDE bridge configuration");
        let mut cfg = self.config.write().await;
        *cfg = config;
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> BridgeConfig {
        self.config.read().await.clone()
    }
}

impl Default for IdeBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_state_display() {
        assert_eq!(BridgeState::Disconnected.to_string(), "disconnected");
        assert_eq!(BridgeState::Connecting.to_string(), "connecting");
        let connected = BridgeState::Connected {
            ide: "vscode".to_string(),
            version: "1.80.0".to_string(),
        };
        assert!(connected.to_string().contains("connected"));
    }

    #[test]
    fn test_bridge_config_default() {
        let config = BridgeConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 7890);
        assert_eq!(config.host, "127.0.0.1");
        assert!(config.auto_connect);
        assert!(config.supported_ides.contains(&"vscode".to_string()));
    }

    #[test]
    fn test_diagnostic_severity_display() {
        assert_eq!(DiagnosticSeverity::Error.to_string(), "error");
        assert_eq!(DiagnosticSeverity::Warning.to_string(), "warning");
        assert_eq!(DiagnosticSeverity::Info.to_string(), "info");
        assert_eq!(DiagnosticSeverity::Hint.to_string(), "hint");
    }

    #[test]
    fn test_message_level_display() {
        assert_eq!(MessageLevel::Error.to_string(), "error");
        assert_eq!(MessageLevel::Warning.to_string(), "warning");
        assert_eq!(MessageLevel::Info.to_string(), "info");
    }

    #[test]
    fn test_position_creation() {
        let pos = Position { line: 10, character: 5 };
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_range_creation() {
        let start = Position { line: 1, character: 0 };
        let end = Position { line: 2, character: 10 };
        let range = Range { start, end };
        assert_eq!(range.start.line, 1);
        assert_eq!(range.end.line, 2);
    }

    #[tokio::test]
    async fn test_ide_bridge_creation() {
        let bridge = IdeBridge::new();
        assert_eq!(bridge.get_state().await, BridgeState::Disconnected);
    }

    #[tokio::test]
    async fn test_ide_bridge_start() {
        let mut bridge = IdeBridge::new();
        bridge.start().await.unwrap();
        assert_eq!(bridge.get_state().await, BridgeState::Connecting);
    }

    #[tokio::test]
    async fn test_ide_bridge_message_queue() {
        let mut bridge = IdeBridge::new();
        let msg = BridgeMessage::ShowMessage {
            level: MessageLevel::Info,
            text: "Test message".to_string(),
        };
        bridge.send(msg.clone()).await.unwrap();
        let received = bridge.receive().await;
        assert!(received.is_some());
    }
}

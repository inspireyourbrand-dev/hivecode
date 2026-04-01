//! Session and conversation history management
//!
//! Provides persistent storage and retrieval of conversation sessions,
//! including auto-saving, session listing, search, and export functionality.

use crate::error::{HiveCodeError, Result};
use crate::types::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Session metadata and messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    /// User-defined or auto-generated title
    pub title: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last modified
    pub updated_at: DateTime<Utc>,
    /// Model used in this session
    pub model_used: String,
    /// Total tokens consumed
    pub token_count: u64,
    /// All messages in the session
    pub messages: Vec<Message>,
}

impl Session {
    /// Create a new session
    pub fn new(model: impl Into<String>) -> Self {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id: id.clone(),
            title: format!("Chat {}", now.format("%Y-%m-%d %H:%M")),
            created_at: now,
            updated_at: now,
            model_used: model.into(),
            token_count: 0,
            messages: Vec::new(),
        }
    }

    /// Generate a title from the first user message (truncate to 50 chars)
    pub fn auto_title_from_messages(&mut self) {
        if let Some(first_user_msg) = self.messages.iter().find(|m| m.role == crate::types::MessageRole::User) {
            let text = first_user_msg.get_text();
            let title = if text.len() > 50 {
                format!("{}...", &text[..47])
            } else {
                text.clone()
            };
            self.title = title;
            self.updated_at = Utc::now();
        }
    }

    /// Add a message to this session
    pub fn add_message(&mut self, message: Message) {
        if let Some(tokens) = &message.tokens {
            self.token_count += tokens.total;
        }
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))
    }

    /// Export to Markdown format
    pub fn to_markdown(&self) -> String {
        use crate::types::ContentBlock;

        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!(
            "**Created:** {} | **Model:** {} | **Tokens:** {}\n\n",
            self.created_at.format("%Y-%m-%d %H:%M:%S"),
            self.model_used,
            self.token_count
        ));

        for msg in &self.messages {
            let role_header = match msg.role {
                crate::types::MessageRole::User => "**User**",
                crate::types::MessageRole::Assistant => "**Assistant**",
                crate::types::MessageRole::System => "**System**",
                crate::types::MessageRole::Tool => "**Tool**",
            };

            md.push_str(&format!("{}\n\n", role_header));

            for block in &msg.content {
                match block {
                    ContentBlock::Text { content } => {
                        md.push_str(&format!("{}\n\n", content));
                    }
                    ContentBlock::ToolUse {
                        id,
                        name,
                        input,
                    } => {
                        md.push_str(&format!("**Tool Call:** {}\n\n", name));
                        md.push_str("```json\n");
                        md.push_str(&serde_json::to_string_pretty(input).unwrap_or_default());
                        md.push_str("\n```\n\n");
                    }
                    ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } => {
                        let label = if *is_error { "Error" } else { "Result" };
                        md.push_str(&format!("**Tool {}:** {}\n\n", label, content));
                    }
                }
            }

            md.push_str("---\n\n");
        }

        md
    }
}

/// Summary of a session (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Session ID
    pub id: String,
    /// Session title
    pub title: String,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last modified
    pub updated_at: DateTime<Utc>,
    /// Model used
    pub model_used: String,
    /// Token count
    pub token_count: u64,
    /// Message count
    pub message_count: usize,
}

impl From<&Session> for SessionSummary {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id.clone(),
            title: session.title.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            model_used: session.model_used.clone(),
            token_count: session.token_count,
            message_count: session.message_count(),
        }
    }
}

/// Manages persistent session storage
pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager with default directory (~/.hivecode/sessions)
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| HiveCodeError::IOError("Could not determine home directory".to_string()))?;
        let sessions_dir = home.join(".hivecode").join("sessions");

        // Create directory if it doesn't exist
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to create sessions directory: {}", e)))?;

        Ok(Self { sessions_dir })
    }

    /// Create with custom sessions directory
    pub fn with_dir(path: impl AsRef<Path>) -> Result<Self> {
        let sessions_dir = path.as_ref().to_path_buf();
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to create sessions directory: {}", e)))?;

        Ok(Self { sessions_dir })
    }

    /// Get the sessions directory path
    pub fn sessions_dir(&self) -> &Path {
        &self.sessions_dir
    }

    /// Save a session to disk
    pub fn save_session(&self, session: &Session) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("{}.json", session.id));
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))?;

        fs::write(&file_path, json)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to save session: {}", e)))?;

        info!("Session saved: {} ({})", session.id, file_path.display());
        Ok(())
    }

    /// Load a session by ID
    pub fn load_session(&self, session_id: &str) -> Result<Session> {
        let file_path = self.sessions_dir.join(format!("{}.json", session_id));

        if !file_path.exists() {
            return Err(HiveCodeError::NotFound(format!("Session not found: {}", session_id)));
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read session: {}", e)))?;

        let session: Session = serde_json::from_str(&content)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))?;

        debug!("Session loaded: {}", session_id);
        Ok(session)
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let file_path = self.sessions_dir.join(format!("{}.json", session_id));

        if !file_path.exists() {
            return Err(HiveCodeError::NotFound(format!("Session not found: {}", session_id)));
        }

        fs::remove_file(&file_path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to delete session: {}", e)))?;

        info!("Session deleted: {}", session_id);
        Ok(())
    }

    /// List all sessions sorted by last modified (newest first)
    pub fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let mut sessions = Vec::new();

        let entries = fs::read_dir(&self.sessions_dir)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read sessions directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| HiveCodeError::IOError(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Session>(&content) {
                        Ok(session) => {
                            sessions.push(SessionSummary::from(&session));
                        }
                        Err(e) => {
                            warn!("Failed to parse session file {}: {}", path.display(), e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read session file {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by updated_at descending (newest first)
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        debug!("Listed {} sessions", sessions.len());
        Ok(sessions)
    }

    /// Search sessions by content (searches in title and all message text)
    pub fn search_sessions(&self, query: &str) -> Result<Vec<SessionSummary>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        let entries = fs::read_dir(&self.sessions_dir)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read sessions directory: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| HiveCodeError::IOError(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match fs::read_to_string(&path) {
                    Ok(content) => match serde_json::from_str::<Session>(&content) {
                        Ok(session) => {
                            // Search in title
                            let title_match = session.title.to_lowercase().contains(&query_lower);

                            // Search in message content
                            let message_match = session
                                .messages
                                .iter()
                                .any(|msg| msg.get_text().to_lowercase().contains(&query_lower));

                            if title_match || message_match {
                                results.push(SessionSummary::from(&session));
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse session file {}: {}", path.display(), e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to read session file {}: {}", path.display(), e);
                    }
                }
            }
        }

        // Sort by updated_at descending
        results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        debug!("Search found {} sessions matching '{}'", results.len(), query);
        Ok(results)
    }

    /// Export a session to markdown
    pub fn export_session_markdown(&self, session_id: &str) -> Result<String> {
        let session = self.load_session(session_id)?;
        Ok(session.to_markdown())
    }

    /// Export a session to JSON
    pub fn export_session_json(&self, session_id: &str) -> Result<String> {
        let session = self.load_session(session_id)?;
        session.to_json()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to current directory if home dir unavailable
            Self {
                sessions_dir: PathBuf::from(".hivecode/sessions"),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContentBlock, MessageRole};

    #[test]
    fn test_new_session() {
        let session = Session::new("gpt-4");
        assert!(!session.id.is_empty());
        assert_eq!(session.model_used, "gpt-4");
        assert_eq!(session.message_count(), 0);
        assert_eq!(session.token_count, 0);
    }

    #[test]
    fn test_auto_title_from_messages() {
        let mut session = Session::new("gpt-4");
        let msg = Message::text(MessageRole::User, "Hello, how can you help me with my project?");
        session.add_message(msg);
        session.auto_title_from_messages();

        assert_eq!(session.title, "Hello, how can you help me with my...");
    }

    #[test]
    fn test_add_message() {
        let mut session = Session::new("gpt-4");
        let msg = Message::text(MessageRole::User, "Test message");
        session.add_message(msg);

        assert_eq!(session.message_count(), 1);
    }

    #[test]
    fn test_session_summary() {
        let session = Session::new("gpt-4");
        let summary = SessionSummary::from(&session);

        assert_eq!(summary.id, session.id);
        assert_eq!(summary.model_used, "gpt-4");
    }

    #[test]
    fn test_session_to_json() {
        let session = Session::new("gpt-4");
        let json = session.to_json().unwrap();
        assert!(json.contains("gpt-4"));
    }

    #[test]
    fn test_session_to_markdown() {
        let mut session = Session::new("gpt-4");
        session.add_message(Message::text(MessageRole::User, "Hello"));
        session.add_message(Message::text(MessageRole::Assistant, "Hi there!"));

        let md = session.to_markdown();
        assert!(md.contains("User"));
        assert!(md.contains("Assistant"));
        assert!(md.contains("Hello"));
        assert!(md.contains("Hi there!"));
    }

    #[test]
    fn test_session_manager_with_temp_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        assert_eq!(manager.sessions_dir(), temp_dir.path());
    }

    #[test]
    fn test_session_manager_save_and_load() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        let mut session = Session::new("gpt-4");
        session.add_message(Message::text(MessageRole::User, "Test"));

        manager.save_session(&session).unwrap();
        let loaded = manager.load_session(&session.id).unwrap();

        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.message_count(), 1);
    }

    #[test]
    fn test_session_manager_delete() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        let session = Session::new("gpt-4");
        manager.save_session(&session).unwrap();

        manager.delete_session(&session.id).unwrap();
        let result = manager.load_session(&session.id);

        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_list() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        let session1 = Session::new("gpt-4");
        let session2 = Session::new("claude");

        manager.save_session(&session1).unwrap();
        manager.save_session(&session2).unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_session_manager_search() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        let mut session1 = Session::new("gpt-4");
        session1.title = "Python Programming".to_string();
        session1.add_message(Message::text(MessageRole::User, "How do I use decorators?"));

        let mut session2 = Session::new("claude");
        session2.title = "JavaScript Help".to_string();
        session2.add_message(Message::text(MessageRole::User, "Explain async/await"));

        manager.save_session(&session1).unwrap();
        manager.save_session(&session2).unwrap();

        let results = manager.search_sessions("Python").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Python Programming");

        let results = manager.search_sessions("decorators").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_export_session_markdown() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        let mut session = Session::new("gpt-4");
        session.title = "Test Session".to_string();
        session.add_message(Message::text(MessageRole::User, "Hello"));

        manager.save_session(&session).unwrap();
        let markdown = manager.export_session_markdown(&session.id).unwrap();

        assert!(markdown.contains("Test Session"));
        assert!(markdown.contains("Hello"));
    }

    #[test]
    fn test_export_session_json() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let manager = SessionManager::with_dir(temp_dir.path()).unwrap();

        let session = Session::new("gpt-4");
        manager.save_session(&session).unwrap();

        let json = manager.export_session_json(&session.id).unwrap();
        assert!(json.contains("gpt-4"));
    }
}

//! Tauri IPC commands for session and conversation history management

use crate::state::TauriAppState;
use hivecode_core::history::{Session, SessionManager, SessionSummary};
use hivecode_core::types::{Message, MessageRole};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info, warn};

/// Serialized session summary for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryResponse {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub model_used: String,
    pub token_count: u64,
    pub message_count: usize,
}

impl From<SessionSummary> for SessionSummaryResponse {
    fn from(session: SessionSummary) -> Self {
        Self {
            id: session.id,
            title: session.title,
            created_at: session.created_at.to_rfc3339(),
            updated_at: session.updated_at.to_rfc3339(),
            model_used: session.model_used,
            token_count: session.token_count,
            message_count: session.message_count,
        }
    }
}

/// Serialized session for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub model_used: String,
    pub token_count: u64,
    pub messages: Vec<SerializedMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// List all sessions sorted by last modified (newest first)
#[tauri::command]
pub async fn list_sessions(
    state: State<'_, TauriAppState>,
) -> Result<Vec<SessionSummaryResponse>, String> {
    debug!("list_sessions command received");

    let manager = SessionManager::new()
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    let sessions = manager
        .list_sessions()
        .map_err(|e| format!("Failed to list sessions: {}", e))?;

    let responses = sessions
        .into_iter()
        .map(SessionSummaryResponse::from)
        .collect();

    info!("Listed {} sessions", responses.len());
    Ok(responses)
}

/// Load a specific session by ID
#[tauri::command]
pub async fn load_session(
    state: State<'_, TauriAppState>,
    session_id: String,
) -> Result<SessionResponse, String> {
    debug!("load_session command received: {}", session_id);

    let manager = SessionManager::new()
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    let session = manager
        .load_session(&session_id)
        .map_err(|e| format!("Failed to load session: {}", e))?;

    let messages = session
        .messages
        .iter()
        .map(|msg| SerializedMessage {
            id: msg.id.clone(),
            role: msg.role.to_string(),
            content: msg.get_text(),
            timestamp: msg.timestamp.to_rfc3339(),
        })
        .collect();

    let response = SessionResponse {
        id: session.id,
        title: session.title,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        model_used: session.model_used,
        token_count: session.token_count,
        messages,
    };

    info!("Session loaded: {}", session_id);
    Ok(response)
}

/// Delete a session by ID
#[tauri::command]
pub async fn delete_session(
    state: State<'_, TauriAppState>,
    session_id: String,
) -> Result<(), String> {
    debug!("delete_session command received: {}", session_id);

    let manager = SessionManager::new()
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    manager
        .delete_session(&session_id)
        .map_err(|e| format!("Failed to delete session: {}", e))?;

    info!("Session deleted: {}", session_id);
    Ok(())
}

/// Export a session to a specified format (json or markdown)
#[tauri::command]
pub async fn export_session(
    state: State<'_, TauriAppState>,
    session_id: String,
    format: String,
) -> Result<String, String> {
    debug!("export_session command received: {} (format: {})", session_id, format);

    let manager = SessionManager::new()
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    let content = match format.to_lowercase().as_str() {
        "json" => manager
            .export_session_json(&session_id)
            .map_err(|e| format!("Failed to export as JSON: {}", e))?,
        "markdown" | "md" => manager
            .export_session_markdown(&session_id)
            .map_err(|e| format!("Failed to export as Markdown: {}", e))?,
        _ => return Err(format!("Unsupported export format: {}. Use 'json' or 'markdown'.", format)),
    };

    info!("Session exported: {} ({})", session_id, format);
    Ok(content)
}

/// Create a new session
#[tauri::command]
pub async fn new_session(
    state: State<'_, TauriAppState>,
    model: Option<String>,
) -> Result<SessionResponse, String> {
    debug!("new_session command received");

    let model = model.unwrap_or_else(|| "claude".to_string());
    let session = Session::new(&model);

    let response = SessionResponse {
        id: session.id,
        title: session.title,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        model_used: session.model_used,
        token_count: session.token_count,
        messages: Vec::new(),
    };

    info!("New session created: {} ({})", response.id, model);
    Ok(response)
}

/// Search sessions by content
#[tauri::command]
pub async fn search_sessions(
    state: State<'_, TauriAppState>,
    query: String,
) -> Result<Vec<SessionSummaryResponse>, String> {
    debug!("search_sessions command received: '{}'", query);

    if query.trim().is_empty() {
        return Err("Search query cannot be empty".to_string());
    }

    let manager = SessionManager::new()
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    let sessions = manager
        .search_sessions(&query)
        .map_err(|e| format!("Failed to search sessions: {}", e))?;

    let responses = sessions
        .into_iter()
        .map(SessionSummaryResponse::from)
        .collect();

    info!("Search found {} sessions matching '{}'", responses.len(), query);
    Ok(responses)
}

/// Save the current conversation as a new session
#[tauri::command]
pub async fn save_current_conversation(
    state: State<'_, TauriAppState>,
    title: Option<String>,
    model: Option<String>,
) -> Result<SessionResponse, String> {
    debug!("save_current_conversation command received");

    let core = state.core().await;
    let messages = core
        .get_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let summary = core
        .get_conversation_summary()
        .await
        .map_err(|e| format!("Failed to get conversation summary: {}", e))?;

    let model = model.unwrap_or_else(|| summary.model.clone());
    let mut session = Session::new(&model);

    // Set title if provided, otherwise will be auto-generated from first message
    if let Some(title) = title {
        session.title = title;
    }

    session.messages = messages;
    session.token_count = summary.total_tokens;

    // Auto-generate title from first message if not provided
    if !session.messages.is_empty() && title.is_none() {
        session.auto_title_from_messages();
    }

    let manager = SessionManager::new()
        .map_err(|e| format!("Failed to initialize session manager: {}", e))?;

    manager
        .save_session(&session)
        .map_err(|e| format!("Failed to save session: {}", e))?;

    let response = SessionResponse {
        id: session.id,
        title: session.title,
        created_at: session.created_at.to_rfc3339(),
        updated_at: session.updated_at.to_rfc3339(),
        model_used: session.model_used,
        token_count: session.token_count,
        messages: session
            .messages
            .iter()
            .map(|msg| SerializedMessage {
                id: msg.id.clone(),
                role: msg.role.to_string(),
                content: msg.get_text(),
                timestamp: msg.timestamp.to_rfc3339(),
            })
            .collect(),
    };

    info!("Conversation saved as session: {}", response.id);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_summary_response_conversion() {
        use hivecode_core::history::SessionSummary;
        use chrono::Utc;

        let summary = SessionSummary {
            id: "test-id".to_string(),
            title: "Test Session".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            model_used: "gpt-4".to_string(),
            token_count: 100,
            message_count: 5,
        };

        let response = SessionSummaryResponse::from(summary.clone());
        assert_eq!(response.id, summary.id);
        assert_eq!(response.title, summary.title);
        assert_eq!(response.model_used, summary.model_used);
    }
}

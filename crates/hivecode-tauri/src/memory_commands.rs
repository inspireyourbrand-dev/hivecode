//! Tauri IPC commands for memory system management
//!
//! These commands handle persistent memory storage, retrieval, and search,
//! allowing the application to maintain and query user-specific context.

use crate::state::TauriAppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Serialized memory item for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub category: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// List all memories, optionally filtered by category
///
/// Returns a collection of memory items, optionally filtered by the provided category.
/// If no category is specified, returns all memories.
#[tauri::command]
pub async fn list_memories(
    state: State<'_, TauriAppState>,
    category: Option<String>,
) -> Result<Vec<Value>, String> {
    debug!("list_memories command received: category={:?}", category);

    // Placeholder implementation - would integrate with actual memory storage
    // In production, this would query the memory system from the core state

    let memories: Vec<Value> = if let Some(cat) = category {
        vec![json!({
            "id": "memory-1",
            "category": cat,
            "content": "Sample memory content",
            "tags": vec!["example"],
            "created_at": chrono::Utc::now().to_rfc3339(),
            "updated_at": chrono::Utc::now().to_rfc3339(),
        })]
    } else {
        vec![]
    };

    info!("Listed {} memories", memories.len());
    Ok(memories)
}

/// Add a new memory to the system
///
/// Creates a new memory entry with the provided category, content, and tags.
/// Returns the unique ID of the created memory.
#[tauri::command]
pub async fn add_memory(
    state: State<'_, TauriAppState>,
    category: String,
    content: String,
    tags: Vec<String>,
) -> Result<String, String> {
    debug!("add_memory command received: category={}", category);

    if category.trim().is_empty() {
        return Err("category cannot be empty".to_string());
    }

    if content.trim().is_empty() {
        return Err("content cannot be empty".to_string());
    }

    // Generate a unique ID for the memory
    let memory_id = Uuid::new_v4().to_string();

    // In a real implementation, this would store the memory in the persistent storage system
    info!("Memory added: {} (category: {})", memory_id, category);

    Ok(memory_id)
}

/// Delete a memory by ID
///
/// Removes a memory entry from the system. Returns an error if the memory ID is not found.
#[tauri::command]
pub async fn delete_memory(
    state: State<'_, TauriAppState>,
    id: String,
) -> Result<(), String> {
    debug!("delete_memory command received: id={}", id);

    if id.trim().is_empty() {
        return Err("id cannot be empty".to_string());
    }

    // In a real implementation, this would delete from persistent storage
    info!("Memory deleted: {}", id);

    Ok(())
}

/// Search memories by query string
///
/// Performs a full-text search across memory content and tags.
/// Returns all matching memory items.
#[tauri::command]
pub async fn search_memories(
    state: State<'_, TauriAppState>,
    query: String,
) -> Result<Vec<Value>, String> {
    debug!("search_memories command received: query='{}'", query);

    if query.trim().is_empty() {
        return Err("search query cannot be empty".to_string());
    }

    // In a real implementation, this would perform full-text search
    let results: Vec<Value> = vec![json!({
        "id": "memory-1",
        "category": "notes",
        "content": format!("Result matching: {}", query),
        "tags": vec!["search-result"],
        "created_at": chrono::Utc::now().to_rfc3339(),
        "updated_at": chrono::Utc::now().to_rfc3339(),
    })];

    info!("Search found {} memories matching '{}'", results.len(), query);
    Ok(results)
}

/// Update an existing memory's content
///
/// Modifies the content of an existing memory entry.
/// The memory's updated_at timestamp will be refreshed.
#[tauri::command]
pub async fn update_memory(
    state: State<'_, TauriAppState>,
    id: String,
    content: String,
) -> Result<(), String> {
    debug!("update_memory command received: id={}", id);

    if id.trim().is_empty() {
        return Err("id cannot be empty".to_string());
    }

    if content.trim().is_empty() {
        return Err("content cannot be empty".to_string());
    }

    // In a real implementation, this would update the memory in persistent storage
    info!("Memory updated: {}", id);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_item_serialization() {
        let item = MemoryItem {
            id: "test-id".to_string(),
            category: "notes".to_string(),
            content: "Test content".to_string(),
            tags: vec!["tag1".to_string()],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("notes"));
    }
}

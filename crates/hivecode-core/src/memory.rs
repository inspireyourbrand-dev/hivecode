//! Memory system for HiveCode
//!
//! Stores user preferences, project context, and learned information across sessions.
//! Provides persistent memory to help the assistant understand user preferences
//! and maintain context across different conversations.

use crate::error::{HiveCodeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// A single entry in the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier for this memory entry
    pub id: String,
    /// Category of memory (preference, context, pattern, etc.)
    pub category: MemoryCategory,
    /// The actual content/data stored
    pub content: String,
    /// Where this memory came from (e.g., "user", "conversation", "auto")
    pub source: String,
    /// When the entry was created
    pub created_at: DateTime<Utc>,
    /// When the entry was last updated
    pub updated_at: DateTime<Utc>,
    /// Relevance score (0.0 to 1.0) for ranking results
    pub relevance_score: f32,
    /// Tags for organizing and searching memories
    pub tags: Vec<String>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(
        category: MemoryCategory,
        content: String,
        source: String,
        tags: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            content,
            source,
            created_at: now,
            updated_at: now,
            relevance_score: 1.0,
            tags,
        }
    }
}

/// Categories for organizing memories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    /// User's working preferences and style
    UserPreference,
    /// Information about the current project
    ProjectContext,
    /// Remembered code patterns or styles
    CodePattern,
    /// Corrections made by the user
    Correction,
    /// Standing instructions to follow
    Instruction,
    /// Custom category
    Custom(String),
}

impl std::fmt::Display for MemoryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryCategory::UserPreference => write!(f, "user_preference"),
            MemoryCategory::ProjectContext => write!(f, "project_context"),
            MemoryCategory::CodePattern => write!(f, "code_pattern"),
            MemoryCategory::Correction => write!(f, "correction"),
            MemoryCategory::Instruction => write!(f, "instruction"),
            MemoryCategory::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Manages persistent memory storage
pub struct MemoryManager {
    memories: Vec<MemoryEntry>,
    storage_path: PathBuf,
}

impl MemoryManager {
    /// Create a new memory manager, loading from default location (~/.hivecode/memory.json)
    pub async fn new(storage_dir: Option<PathBuf>) -> Result<Self> {
        let storage_path = if let Some(dir) = storage_dir {
            dir.join("memory.json")
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| HiveCodeError::IOError("Could not determine home directory".to_string()))?;
            home.join(".hivecode").join("memory.json")
        };

        // Create parent directories if they don't exist
        if let Some(parent) = storage_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| HiveCodeError::IOError(format!("Failed to create memory directory: {}", e)))?;
        }

        // Load existing memories if the file exists
        let memories = if storage_path.exists() {
            Self::load(&storage_path).await.unwrap_or_else(|e| {
                warn!("Failed to load memories: {}", e);
                Vec::new()
            })
        } else {
            Vec::new()
        };

        debug!("Memory manager initialized with {} entries", memories.len());

        Ok(Self {
            memories,
            storage_path,
        })
    }

    /// Add a new memory entry
    pub async fn add(
        &mut self,
        category: MemoryCategory,
        content: String,
        source: String,
        tags: Vec<String>,
    ) -> Result<String> {
        let entry = MemoryEntry::new(category, content, source, tags);
        let id = entry.id.clone();

        self.memories.push(entry);
        self.save().await?;

        info!("Added memory entry: {}", id);
        Ok(id)
    }

    /// Search memories by keyword
    pub async fn search(&self, query: &str, limit: usize) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<&MemoryEntry> = self
            .memories
            .iter()
            .filter(|entry| {
                entry.content.to_lowercase().contains(&query_lower)
                    || entry.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect();

        // Sort by relevance score (descending)
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });

        results.truncate(limit);
        results
    }

    /// Get a memory entry by ID
    pub async fn get(&self, id: &str) -> Option<&MemoryEntry> {
        self.memories.iter().find(|entry| entry.id == id)
    }

    /// Update an existing memory entry
    pub async fn update(&mut self, id: &str, content: String) -> Result<()> {
        if let Some(entry) = self.memories.iter_mut().find(|e| e.id == id) {
            entry.content = content;
            entry.updated_at = Utc::now();
            self.save().await?;
            debug!("Updated memory entry: {}", id);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Memory entry not found: {}", id)))
        }
    }

    /// Delete a memory entry
    pub async fn delete(&mut self, id: &str) -> Result<()> {
        if let Some(pos) = self.memories.iter().position(|e| e.id == id) {
            self.memories.remove(pos);
            self.save().await?;
            info!("Deleted memory entry: {}", id);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Memory entry not found: {}", id)))
        }
    }

    /// Get all memories of a specific category
    pub async fn list_by_category(&self, category: &MemoryCategory) -> Vec<&MemoryEntry> {
        self.memories
            .iter()
            .filter(|entry| &entry.category == category)
            .collect()
    }

    /// Clear all memories of a specific category
    pub async fn clear_category(&mut self, category: &MemoryCategory) -> Result<usize> {
        let original_len = self.memories.len();
        self.memories.retain(|entry| &entry.category != category);
        let removed = original_len - self.memories.len();

        if removed > 0 {
            self.save().await?;
            info!("Cleared {} memories from category: {}", removed, category);
        }

        Ok(removed)
    }

    /// Build context string for including in LLM prompts
    pub fn build_memory_context(&self, query: &str, max_tokens: usize) -> String {
        let mut context = String::new();
        context.push_str("# Remembered Context\n\n");

        // First, search for relevant memories
        let relevant = self.memories
            .iter()
            .filter(|entry| {
                entry.content.to_lowercase().contains(&query.to_lowercase())
                    || entry.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .collect::<Vec<_>>();

        if relevant.is_empty() {
            // If no relevant memories, include top memories by score
            let mut by_score: Vec<_> = self.memories.iter().collect();
            by_score.sort_by(|a, b| {
                b.relevance_score
                    .partial_cmp(&a.relevance_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for entry in by_score.iter().take(5) {
                context.push_str(&format!("**{}**: {}\n", entry.category, entry.content));
                context.push('\n');
            }
        } else {
            // Include relevant memories
            for entry in relevant.iter().take(10) {
                context.push_str(&format!("**{}**: {}\n", entry.category, entry.content));
                context.push('\n');
            }
        }

        // Truncate if needed
        if context.len() > max_tokens {
            context.truncate(max_tokens);
            context.push_str("\n...[truncated]");
        }

        context
    }

    /// Save memories to disk
    async fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.memories)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))?;

        std::fs::write(&self.storage_path, json)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to save memories: {}", e)))?;

        debug!("Saved {} memory entries", self.memories.len());
        Ok(())
    }

    /// Load memories from disk
    async fn load(path: &Path) -> Result<Vec<MemoryEntry>> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read memories: {}", e)))?;

        let memories = serde_json::from_str(&content)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))?;

        debug!("Loaded memories from {}", path.display());
        Ok(memories)
    }

    /// Get total count of stored memories
    pub fn count(&self) -> usize {
        self.memories.len()
    }

    /// Get count of memories by category
    pub fn count_by_category(&self, category: &MemoryCategory) -> usize {
        self.memories
            .iter()
            .filter(|entry| &entry.category == category)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_entry_creation() {
        let entry = MemoryEntry::new(
            MemoryCategory::UserPreference,
            "Use snake_case for variables".to_string(),
            "user".to_string(),
            vec!["style".to_string()],
        );

        assert_eq!(entry.category, MemoryCategory::UserPreference);
        assert_eq!(entry.content, "Use snake_case for variables");
        assert_eq!(entry.source, "user");
        assert_eq!(entry.relevance_score, 1.0);
        assert_eq!(entry.tags.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_add_and_get_memory() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        let id = manager
            .add(
                MemoryCategory::UserPreference,
                "Prefer async/await".to_string(),
                "user".to_string(),
                vec!["rust".to_string()],
            )
            .await
            .unwrap();

        let entry = manager.get(&id).await;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().content, "Prefer async/await");
    }

    #[tokio::test]
    async fn test_search_memories() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        manager
            .add(
                MemoryCategory::CodePattern,
                "Use Result type for error handling".to_string(),
                "auto".to_string(),
                vec!["rust".to_string()],
            )
            .await
            .unwrap();

        manager
            .add(
                MemoryCategory::UserPreference,
                "Prefer clear variable names".to_string(),
                "user".to_string(),
                vec!["style".to_string()],
            )
            .await
            .unwrap();

        let results = manager.search("rust", 10).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Use Result type for error handling");
    }

    #[tokio::test]
    async fn test_update_memory() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        let id = manager
            .add(
                MemoryCategory::ProjectContext,
                "Project uses Rust".to_string(),
                "auto".to_string(),
                vec![],
            )
            .await
            .unwrap();

        manager
            .update(&id, "Project uses Rust 1.70+".to_string())
            .await
            .unwrap();

        let entry = manager.get(&id).await.unwrap();
        assert_eq!(entry.content, "Project uses Rust 1.70+");
    }

    #[tokio::test]
    async fn test_delete_memory() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        let id = manager
            .add(
                MemoryCategory::Correction,
                "Fix indentation".to_string(),
                "user".to_string(),
                vec![],
            )
            .await
            .unwrap();

        manager.delete(&id).await.unwrap();
        assert!(manager.get(&id).await.is_none());
    }

    #[tokio::test]
    async fn test_list_by_category() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        manager
            .add(
                MemoryCategory::UserPreference,
                "Pref 1".to_string(),
                "user".to_string(),
                vec![],
            )
            .await
            .unwrap();

        manager
            .add(
                MemoryCategory::UserPreference,
                "Pref 2".to_string(),
                "user".to_string(),
                vec![],
            )
            .await
            .unwrap();

        manager
            .add(
                MemoryCategory::ProjectContext,
                "Context 1".to_string(),
                "auto".to_string(),
                vec![],
            )
            .await
            .unwrap();

        let prefs = manager.list_by_category(&MemoryCategory::UserPreference).await;
        assert_eq!(prefs.len(), 2);
    }

    #[tokio::test]
    async fn test_clear_category() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        manager
            .add(
                MemoryCategory::Instruction,
                "Instr 1".to_string(),
                "user".to_string(),
                vec![],
            )
            .await
            .unwrap();

        manager
            .add(
                MemoryCategory::Instruction,
                "Instr 2".to_string(),
                "user".to_string(),
                vec![],
            )
            .await
            .unwrap();

        let removed = manager.clear_category(&MemoryCategory::Instruction).await.unwrap();
        assert_eq!(removed, 2);
        assert_eq!(manager.count(), 0);
    }

    #[tokio::test]
    async fn test_build_memory_context() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        manager
            .add(
                MemoryCategory::UserPreference,
                "Use Rust best practices".to_string(),
                "user".to_string(),
                vec!["rust".to_string()],
            )
            .await
            .unwrap();

        let context = manager.build_memory_context("rust", 1000);
        assert!(context.contains("Remembered Context"));
        assert!(context.contains("Rust best practices"));
    }

    #[tokio::test]
    async fn test_memory_category_display() {
        assert_eq!(MemoryCategory::UserPreference.to_string(), "user_preference");
        assert_eq!(MemoryCategory::ProjectContext.to_string(), "project_context");
        assert_eq!(MemoryCategory::Custom("test".to_string()).to_string(), "test");
    }

    #[tokio::test]
    async fn test_memory_serialization() {
        let entry = MemoryEntry::new(
            MemoryCategory::CodePattern,
            "Test pattern".to_string(),
            "user".to_string(),
            vec!["test".to_string()],
        );

        let json = serde_json::to_string(&entry).unwrap();
        let restored: MemoryEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, entry.id);
        assert_eq!(restored.content, entry.content);
        assert_eq!(restored.category, entry.category);
    }

    #[tokio::test]
    async fn test_count_methods() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = MemoryManager::new(Some(temp_dir.path().to_path_buf())).await.unwrap();

        manager
            .add(
                MemoryCategory::UserPreference,
                "Pref".to_string(),
                "user".to_string(),
                vec![],
            )
            .await
            .unwrap();

        manager
            .add(
                MemoryCategory::ProjectContext,
                "Context".to_string(),
                "auto".to_string(),
                vec![],
            )
            .await
            .unwrap();

        assert_eq!(manager.count(), 2);
        assert_eq!(manager.count_by_category(&MemoryCategory::UserPreference), 1);
        assert_eq!(manager.count_by_category(&MemoryCategory::ProjectContext), 1);
    }
}

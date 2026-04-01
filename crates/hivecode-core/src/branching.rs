//! Conversation branching for HiveCode
//!
//! Fork conversations at any point to explore different approaches
//! without losing the original thread.

use crate::error::{HiveCodeError, Result};
use crate::types::Message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// A single conversation branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationBranch {
    /// Unique branch identifier
    pub id: String,
    /// Human-readable branch name
    pub name: String,
    /// ID of the parent branch (None if root)
    pub parent_branch_id: Option<String>,
    /// Message ID at the fork point
    pub fork_point_message_id: Option<String>,
    /// All messages in this branch
    pub messages: Vec<Message>,
    /// When this branch was created
    pub created_at: DateTime<Utc>,
    /// When this branch was last updated
    pub updated_at: DateTime<Utc>,
    /// Whether this is the currently active branch
    pub is_active: bool,
    /// Additional metadata
    pub metadata: BranchMetadata,
}

/// Metadata about a branch
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchMetadata {
    /// Optional description of the branch
    pub description: Option<String>,
    /// AI model used in this branch
    pub model_used: Option<String>,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost
    pub total_cost: f64,
    /// User-defined tags
    pub tags: Vec<String>,
}

/// The complete tree of branches for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchTree {
    /// ID of the root branch
    pub root_branch_id: String,
    /// All branches in the tree
    pub branches: HashMap<String, ConversationBranch>,
    /// Currently active branch ID
    pub active_branch_id: String,
}

/// Summary of a branch for listing
#[derive(Debug, Clone)]
pub struct BranchSummary {
    /// Branch ID
    pub id: String,
    /// Branch name
    pub name: String,
    /// Number of messages
    pub message_count: usize,
    /// Whether this is the active branch
    pub is_active: bool,
    /// Name of parent branch
    pub parent_name: Option<String>,
    /// Creation time
    pub created_at: DateTime<Utc>,
}

/// Result of merging two branches
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// Number of messages added
    pub messages_added: usize,
    /// Message IDs that had conflicts
    pub conflicts: Vec<String>,
}

/// Comparison between two branches
#[derive(Debug, Clone)]
pub struct BranchComparison {
    /// Number of shared messages
    pub shared_messages: usize,
    /// Messages unique to branch A
    pub branch_a_unique: usize,
    /// Messages unique to branch B
    pub branch_b_unique: usize,
    /// Message ID where branches diverged
    pub divergence_point: Option<String>,
}

/// Manages conversation branching
pub struct BranchManager {
    tree: BranchTree,
    storage_path: PathBuf,
}

impl BranchManager {
    /// Create a new branch manager
    pub async fn new(
        conversation_id: &str,
        storage_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let storage_path = match storage_dir {
            Some(dir) => dir.join(conversation_id),
            None => {
                let config_dir = dirs::config_dir()
                    .ok_or_else(|| HiveCodeError::Internal("No config dir found".to_string()))?;
                config_dir.join("hivecode").join("conversations").join(conversation_id)
            }
        };

        // Create root branch
        let root_id = uuid::Uuid::new_v4().to_string();
        let mut branches = HashMap::new();
        let root_branch = ConversationBranch {
            id: root_id.clone(),
            name: "Main".to_string(),
            parent_branch_id: None,
            fork_point_message_id: None,
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
            metadata: BranchMetadata::default(),
        };

        branches.insert(root_id.clone(), root_branch);

        let tree = BranchTree {
            root_branch_id: root_id.clone(),
            branches,
            active_branch_id: root_id,
        };

        let manager = Self {
            tree,
            storage_path,
        };

        manager.save().await?;
        info!("Created branch manager for conversation: {}", conversation_id);

        Ok(manager)
    }

    /// Create a new branch from the current conversation at a specific message
    pub async fn fork(&mut self, at_message_id: &str, branch_name: &str) -> Result<String> {
        let current_branch = self.tree.branches.get(&self.tree.active_branch_id).ok_or_else(|| {
            HiveCodeError::Internal("Active branch not found".to_string())
        })?;

        // Find the fork point message
        let fork_index = current_branch
            .messages
            .iter()
            .position(|m| m.id == at_message_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Message not found: {}", at_message_id)))?;

        // Create new branch with messages up to and including fork point
        let new_branch_id = uuid::Uuid::new_v4().to_string();
        let messages = current_branch.messages[..=fork_index].to_vec();

        let new_branch = ConversationBranch {
            id: new_branch_id.clone(),
            name: branch_name.to_string(),
            parent_branch_id: Some(self.tree.active_branch_id.clone()),
            fork_point_message_id: Some(at_message_id.to_string()),
            messages,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: false,
            metadata: BranchMetadata::default(),
        };

        debug!(
            "Forking branch '{}' at message '{}' -> new branch '{}'",
            current_branch.name, at_message_id, branch_name
        );

        self.tree.branches.insert(new_branch_id.clone(), new_branch);
        self.save().await?;

        Ok(new_branch_id)
    }

    /// Switch to a different branch
    pub async fn switch_to(&mut self, branch_id: &str) -> Result<&ConversationBranch> {
        if !self.tree.branches.contains_key(branch_id) {
            return Err(HiveCodeError::NotFound(format!(
                "Branch not found: {}",
                branch_id
            )));
        }

        // Deactivate current branch
        if let Some(current) = self.tree.branches.get_mut(&self.tree.active_branch_id) {
            current.is_active = false;
        }

        // Activate new branch
        if let Some(new) = self.tree.branches.get_mut(branch_id) {
            new.is_active = true;
            new.updated_at = Utc::now();
        }

        self.tree.active_branch_id = branch_id.to_string();
        self.save().await?;

        info!("Switched to branch: {}", branch_id);

        Ok(self.tree.branches.get(branch_id).unwrap())
    }

    /// Get the currently active branch
    pub fn active_branch(&self) -> &ConversationBranch {
        self.tree.branches.get(&self.tree.active_branch_id).unwrap()
    }

    /// Get mutable reference to active branch
    pub fn active_branch_mut(&mut self) -> &mut ConversationBranch {
        let id = self.tree.active_branch_id.clone();
        self.tree.branches.get_mut(&id).unwrap()
    }

    /// List all branches
    pub fn list_branches(&self) -> Vec<BranchSummary> {
        self.tree
            .branches
            .values()
            .map(|branch| {
                let parent_name = branch
                    .parent_branch_id
                    .as_ref()
                    .and_then(|id| self.tree.branches.get(id).map(|b| b.name.clone()));

                BranchSummary {
                    id: branch.id.clone(),
                    name: branch.name.clone(),
                    message_count: branch.messages.len(),
                    is_active: branch.is_active,
                    parent_name,
                    created_at: branch.created_at,
                }
            })
            .collect()
    }

    /// Delete a branch (cannot delete root or active)
    pub async fn delete_branch(&mut self, branch_id: &str) -> Result<()> {
        if branch_id == self.tree.root_branch_id {
            return Err(HiveCodeError::Internal(
                "Cannot delete root branch".to_string(),
            ));
        }

        if branch_id == self.tree.active_branch_id {
            return Err(HiveCodeError::Internal(
                "Cannot delete active branch".to_string(),
            ));
        }

        if self.tree.branches.remove(branch_id).is_none() {
            return Err(HiveCodeError::NotFound(format!(
                "Branch not found: {}",
                branch_id
            )));
        }

        debug!("Deleted branch: {}", branch_id);
        self.save().await?;

        Ok(())
    }

    /// Rename a branch
    pub async fn rename_branch(&mut self, branch_id: &str, new_name: &str) -> Result<()> {
        let branch = self
            .tree
            .branches
            .get_mut(branch_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Branch not found: {}", branch_id)))?;

        branch.name = new_name.to_string();
        branch.updated_at = Utc::now();

        debug!("Renamed branch '{}' to '{}'", branch_id, new_name);
        self.save().await?;

        Ok(())
    }

    /// Merge messages from source branch into target branch
    pub async fn merge(
        &mut self,
        source_branch_id: &str,
        target_branch_id: &str,
    ) -> Result<MergeResult> {
        let source_branch = self
            .tree
            .branches
            .get(source_branch_id)
            .ok_or_else(|| {
                HiveCodeError::NotFound(format!("Source branch not found: {}", source_branch_id))
            })?
            .clone();

        let target_branch = self
            .tree
            .branches
            .get_mut(target_branch_id)
            .ok_or_else(|| {
                HiveCodeError::NotFound(format!("Target branch not found: {}", target_branch_id))
            })?;

        // Find fork point
        let fork_point = source_branch
            .messages
            .iter()
            .find(|m| {
                target_branch
                    .messages
                    .iter()
                    .find(|tm| tm.id == m.id)
                    .is_none()
            })
            .map(|m| m.id.clone());

        let mut messages_added = 0;
        let mut conflicts = Vec::new();

        // Add messages from source that aren't in target
        for msg in &source_branch.messages {
            if !target_branch.messages.iter().any(|m| m.id == msg.id) {
                target_branch.messages.push(msg.clone());
                messages_added += 1;
            }
        }

        target_branch.updated_at = Utc::now();

        if let Some(fork) = fork_point {
            debug!("Merged branch {} into {} at fork point {}", source_branch_id, target_branch_id, fork);
        }

        self.save().await?;

        Ok(MergeResult {
            messages_added,
            conflicts,
        })
    }

    /// Get the full message history including parent messages up to fork point
    pub fn get_full_history(&self) -> Vec<&Message> {
        let current_branch = self.active_branch();

        if current_branch.parent_branch_id.is_none() {
            // Root branch, just return its messages
            return current_branch.messages.iter().collect();
        }

        // Collect messages from parent branches
        let mut all_messages = Vec::new();
        let mut current_id = Some(self.tree.active_branch_id.clone());

        while let Some(branch_id) = current_id {
            if let Some(branch) = self.tree.branches.get(&branch_id) {
                // Add this branch's messages at the beginning
                for msg in &branch.messages {
                    if !all_messages.iter().any(|m: &&Message| m.id == msg.id) {
                        all_messages.insert(0, msg);
                    }
                }
                current_id = branch.parent_branch_id.clone();
            } else {
                break;
            }
        }

        all_messages
    }

    /// Compare two branches
    pub fn compare_branches(
        &self,
        branch_a: &str,
        branch_b: &str,
    ) -> Result<BranchComparison> {
        let branch_a_obj = self
            .tree
            .branches
            .get(branch_a)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Branch not found: {}", branch_a)))?;

        let branch_b_obj = self
            .tree
            .branches
            .get(branch_b)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Branch not found: {}", branch_b)))?;

        let mut shared = 0;
        let mut a_unique = 0;
        let mut b_unique = 0;
        let mut divergence_point = None;

        // Count shared messages
        for msg_a in &branch_a_obj.messages {
            if let Some(msg_b) = branch_b_obj.messages.iter().find(|m| m.id == msg_a.id) {
                shared += 1;
            } else if divergence_point.is_none() {
                divergence_point = Some(msg_a.id.clone());
                a_unique += 1;
            } else {
                a_unique += 1;
            }
        }

        for msg_b in &branch_b_obj.messages {
            if branch_a_obj.messages.iter().find(|m| m.id == msg_b.id).is_none() {
                b_unique += 1;
            }
        }

        Ok(BranchComparison {
            shared_messages: shared,
            branch_a_unique: a_unique,
            branch_b_unique: b_unique,
            divergence_point,
        })
    }

    // Private helper

    async fn save(&self) -> Result<()> {
        // Create storage directory if it doesn't exist
        tokio::fs::create_dir_all(&self.storage_path)
            .await
            .map_err(|e| {
                HiveCodeError::IOError(format!("Failed to create storage directory: {}", e))
            })?;

        let file_path = self.storage_path.join("branches.json");
        let json = serde_json::to_string(&self.tree).map_err(|e| {
            HiveCodeError::SerializationError(format!("Failed to serialize branches: {}", e))
        })?;

        tokio::fs::write(&file_path, json)
            .await
            .map_err(|e| HiveCodeError::IOError(format!("Failed to save branches: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_branch_manager() {
        let manager = BranchManager::new("test-conv", None).await.unwrap();
        assert_eq!(manager.tree.branches.len(), 1);
        assert!(!manager.tree.root_branch_id.is_empty());
    }

    #[tokio::test]
    async fn test_list_branches() {
        let manager = BranchManager::new("test-conv", None).await.unwrap();
        let branches = manager.list_branches();
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "Main");
    }

    #[test]
    fn test_active_branch() {
        let manager = BranchManager {
            tree: BranchTree {
                root_branch_id: "root".to_string(),
                branches: {
                    let mut map = HashMap::new();
                    map.insert(
                        "root".to_string(),
                        ConversationBranch {
                            id: "root".to_string(),
                            name: "Main".to_string(),
                            parent_branch_id: None,
                            fork_point_message_id: None,
                            messages: vec![],
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            is_active: true,
                            metadata: BranchMetadata::default(),
                        },
                    );
                    map
                },
                active_branch_id: "root".to_string(),
            },
            storage_path: PathBuf::from("/tmp"),
        };

        assert_eq!(manager.active_branch().name, "Main");
    }

    #[tokio::test]
    async fn test_switch_branch() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();
        let root_id = manager.tree.root_branch_id.clone();

        // Create a message
        let msg = Message::text(crate::types::MessageRole::User, "test");
        manager.active_branch_mut().messages.push(msg.clone());

        // Fork at this message
        let new_branch_id = manager.fork(&msg.id, "Feature Branch").await.unwrap();

        // Switch to new branch
        manager.switch_to(&new_branch_id).await.unwrap();
        assert_eq!(manager.tree.active_branch_id, new_branch_id);

        // Switch back
        manager.switch_to(&root_id).await.unwrap();
        assert_eq!(manager.tree.active_branch_id, root_id);
    }

    #[tokio::test]
    async fn test_fork_branch() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();

        let msg = Message::text(crate::types::MessageRole::User, "test");
        manager.active_branch_mut().messages.push(msg.clone());

        let branch_id = manager.fork(&msg.id, "Test Branch").await.unwrap();
        assert!(!branch_id.is_empty());
        assert_eq!(manager.tree.branches.len(), 2);

        let new_branch = manager.tree.branches.get(&branch_id).unwrap();
        assert_eq!(new_branch.name, "Test Branch");
    }

    #[tokio::test]
    async fn test_delete_branch() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();

        let msg = Message::text(crate::types::MessageRole::User, "test");
        manager.active_branch_mut().messages.push(msg.clone());

        let branch_id = manager.fork(&msg.id, "Temp Branch").await.unwrap();
        assert_eq!(manager.tree.branches.len(), 2);

        manager.delete_branch(&branch_id).await.unwrap();
        assert_eq!(manager.tree.branches.len(), 1);
    }

    #[tokio::test]
    async fn test_cannot_delete_active_branch() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();
        let active_id = manager.tree.active_branch_id.clone();

        let result = manager.delete_branch(&active_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cannot_delete_root_branch() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();
        let root_id = manager.tree.root_branch_id.clone();

        let result = manager.delete_branch(&root_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_rename_branch() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();
        let root_id = manager.tree.root_branch_id.clone();

        manager.rename_branch(&root_id, "Custom Name").await.unwrap();
        assert_eq!(manager.active_branch().name, "Custom Name");
    }

    #[tokio::test]
    async fn test_merge_branches() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();
        let root_id = manager.tree.root_branch_id.clone();

        // Add messages to root
        let msg1 = Message::text(crate::types::MessageRole::User, "message 1");
        manager.active_branch_mut().messages.push(msg1.clone());

        // Fork
        let branch_id = manager.fork(&msg1.id, "Branch A").await.unwrap();
        manager.switch_to(&branch_id).await.unwrap();

        // Add message to new branch
        let msg2 = Message::text(crate::types::MessageRole::Assistant, "message 2");
        manager.active_branch_mut().messages.push(msg2.clone());

        // Merge back to root
        let result = manager.merge(&branch_id, &root_id).await.unwrap();
        assert!(result.messages_added > 0);
    }

    #[tokio::test]
    async fn test_compare_branches() {
        let mut manager = BranchManager::new("test-conv", None).await.unwrap();
        let root_id = manager.tree.root_branch_id.clone();

        let msg = Message::text(crate::types::MessageRole::User, "test");
        manager.active_branch_mut().messages.push(msg.clone());

        let branch_id = manager.fork(&msg.id, "Branch A").await.unwrap();

        let comparison = manager.compare_branches(&root_id, &branch_id).unwrap();
        assert_eq!(comparison.shared_messages, 1);
    }

    #[test]
    fn test_get_full_history() {
        let root_msg = Message::text(crate::types::MessageRole::User, "root");
        let mut root_branch = ConversationBranch {
            id: "root".to_string(),
            name: "Main".to_string(),
            parent_branch_id: None,
            fork_point_message_id: None,
            messages: vec![root_msg.clone()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: false,
            metadata: BranchMetadata::default(),
        };

        let child_msg = Message::text(crate::types::MessageRole::Assistant, "child");
        let child_branch = ConversationBranch {
            id: "child".to_string(),
            name: "Child".to_string(),
            parent_branch_id: Some("root".to_string()),
            fork_point_message_id: Some(root_msg.id.clone()),
            messages: vec![root_msg.clone(), child_msg.clone()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
            metadata: BranchMetadata::default(),
        };

        let mut branches = HashMap::new();
        branches.insert("root".to_string(), root_branch);
        branches.insert("child".to_string(), child_branch);

        let manager = BranchManager {
            tree: BranchTree {
                root_branch_id: "root".to_string(),
                branches,
                active_branch_id: "child".to_string(),
            },
            storage_path: PathBuf::from("/tmp"),
        };

        let history = manager.get_full_history();
        assert!(history.len() >= 1);
    }

    #[test]
    fn test_branch_metadata() {
        let metadata = BranchMetadata {
            description: Some("Test branch".to_string()),
            model_used: Some("claude-3".to_string()),
            total_tokens: 1000,
            total_cost: 0.01,
            tags: vec!["feature".to_string()],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let restored: BranchMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.description, Some("Test branch".to_string()));
        assert_eq!(restored.total_tokens, 1000);
    }
}

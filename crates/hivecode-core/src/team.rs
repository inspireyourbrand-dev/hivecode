//! Team/swarm coordination for HiveCode
//!
//! Orchestrates multiple AI agents working together on complex tasks.
//! Supports coordinator mode, shared memory, and progress synchronization.

use crate::error::{HiveCodeError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// A team session coordinating multiple agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSession {
    /// Unique session identifier
    pub id: String,
    /// Human-readable session name
    pub name: String,
    /// ID of the coordinator agent
    pub coordinator_id: String,
    /// Team members (agents)
    pub members: Vec<TeamMember>,
    /// Shared memory accessible to all members
    pub shared_memory: Vec<SharedMemoryEntry>,
    /// Task board
    pub task_board: Vec<TeamTask>,
    /// Overall session status
    pub status: TeamStatus,
    /// When the session was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A team member (agent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    /// Unique agent identifier
    pub agent_id: String,
    /// Role in the team
    pub role: TeamRole,
    /// Human-readable name
    pub name: String,
    /// Model/provider (e.g., "gpt-4", "claude-3-opus")
    pub model: String,
    /// Currently assigned task ID
    pub current_task: Option<String>,
    /// Current status
    pub status: MemberStatus,
    /// Tokens used so far
    pub tokens_used: u64,
    /// Estimated cost
    pub cost: f64,
}

/// Role of a team member
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamRole {
    /// Coordinator (orchestrator)
    Coordinator,
    /// Implementation
    Implementer,
    /// Code review
    Reviewer,
    /// Testing
    Tester,
    /// Research/investigation
    Researcher,
    /// Custom role
    Custom(String),
}

impl std::fmt::Display for TeamRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamRole::Coordinator => write!(f, "Coordinator"),
            TeamRole::Implementer => write!(f, "Implementer"),
            TeamRole::Reviewer => write!(f, "Reviewer"),
            TeamRole::Tester => write!(f, "Tester"),
            TeamRole::Researcher => write!(f, "Researcher"),
            TeamRole::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Status of a team member
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberStatus {
    /// Idle, waiting for assignment
    Idle,
    /// Currently working
    Working,
    /// Waiting for review
    WaitingForReview,
    /// Blocked on dependency
    Blocked,
    /// Task completed
    Done,
}

impl std::fmt::Display for MemberStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberStatus::Idle => write!(f, "idle"),
            MemberStatus::Working => write!(f, "working"),
            MemberStatus::WaitingForReview => write!(f, "waiting_for_review"),
            MemberStatus::Blocked => write!(f, "blocked"),
            MemberStatus::Done => write!(f, "done"),
        }
    }
}

/// Overall team session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeamStatus {
    /// Planning phase
    Planning,
    /// Active work in progress
    InProgress,
    /// Reviewing work
    Reviewing,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
}

impl std::fmt::Display for TeamStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamStatus::Planning => write!(f, "planning"),
            TeamStatus::InProgress => write!(f, "in_progress"),
            TeamStatus::Reviewing => write!(f, "reviewing"),
            TeamStatus::Completed => write!(f, "completed"),
            TeamStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A task in the team task board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTask {
    /// Unique task identifier
    pub id: String,
    /// Task title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Assigned to agent ID
    pub assigned_to: Option<String>,
    /// Task IDs this depends on
    pub depends_on: Vec<String>,
    /// Current status
    pub status: TaskStatus,
    /// Task result/output
    pub result: Option<String>,
    /// Priority (0-10, higher = more important)
    pub priority: u8,
}

/// Status of a task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Not started
    Todo,
    /// Currently in progress
    InProgress,
    /// Under review
    InReview,
    /// Completed
    Done,
    /// Blocked on dependency
    Blocked,
    /// Failed
    Failed,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "todo"),
            TaskStatus::InProgress => write!(f, "in_progress"),
            TaskStatus::InReview => write!(f, "in_review"),
            TaskStatus::Done => write!(f, "done"),
            TaskStatus::Blocked => write!(f, "blocked"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Shared memory entry accessible to all team members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemoryEntry {
    /// Key for the memory entry
    pub key: String,
    /// Value (typically JSON)
    pub value: String,
    /// Agent that wrote this entry
    pub written_by: String,
    /// When it was written
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Progress metrics for a team session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProgress {
    /// Total tasks
    pub total_tasks: usize,
    /// Completed tasks
    pub completed: usize,
    /// Tasks in progress
    pub in_progress: usize,
    /// Blocked tasks
    pub blocked: usize,
    /// Total cost so far
    pub total_cost: f64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Estimated completion percentage
    pub estimated_completion: Option<f64>,
}

/// Team coordinator for managing multi-agent collaboration
pub struct TeamCoordinator {
    sessions: Arc<RwLock<HashMap<String, TeamSession>>>,
    storage_path: PathBuf,
}

impl TeamCoordinator {
    /// Create a new team coordinator
    pub async fn new(storage_dir: Option<PathBuf>) -> Result<Self> {
        debug!("Creating new TeamCoordinator");
        let storage_path = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".hivecode");
            path.push("teams");
            path
        });

        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        })
    }

    /// Create a new team session
    pub async fn create_session(&mut self, name: &str, task_description: &str) -> Result<String> {
        debug!("Creating team session: {}", name);
        let id = uuid::Uuid::new_v4().to_string();
        let session = TeamSession {
            id: id.clone(),
            name: name.to_string(),
            coordinator_id: uuid::Uuid::new_v4().to_string(),
            members: Vec::new(),
            shared_memory: Vec::new(),
            task_board: Vec::new(),
            status: TeamStatus::Planning,
            created_at: Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(id.clone(), session);
        self.save().await?;
        info!("Team session created: {}", id);
        Ok(id)
    }

    /// Add a team member
    pub async fn add_member(
        &mut self,
        session_id: &str,
        role: TeamRole,
        model: &str,
    ) -> Result<String> {
        debug!("Adding member to session: {}", session_id);
        let agent_id = uuid::Uuid::new_v4().to_string();

        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        let member = TeamMember {
            agent_id: agent_id.clone(),
            role,
            name: format!("Agent-{}", agent_id.chars().take(8).collect::<String>()),
            model: model.to_string(),
            current_task: None,
            status: MemberStatus::Idle,
            tokens_used: 0,
            cost: 0.0,
        };

        session.members.push(member);
        self.save().await?;
        info!("Member added to session: {} ({})", session_id, agent_id);
        Ok(agent_id)
    }

    /// Remove a team member
    pub async fn remove_member(&mut self, session_id: &str, member_id: &str) -> Result<()> {
        debug!("Removing member from session: {}", session_id);
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        session.members.retain(|m| m.agent_id != member_id);
        self.save().await?;
        info!("Member removed from session: {}", session_id);
        Ok(())
    }

    /// Assign a task to a member
    pub async fn assign_task(
        &mut self,
        session_id: &str,
        task_id: &str,
        member_id: &str,
    ) -> Result<()> {
        debug!("Assigning task to member: {}", member_id);
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        // Find and update task
        for task in &mut session.task_board {
            if task.id == task_id {
                task.assigned_to = Some(member_id.to_string());
                task.status = TaskStatus::InProgress;
                break;
            }
        }

        // Update member status
        for member in &mut session.members {
            if member.agent_id == member_id {
                member.current_task = Some(task_id.to_string());
                member.status = MemberStatus::Working;
                break;
            }
        }

        self.save().await?;
        Ok(())
    }

    /// Mark task as complete
    pub async fn complete_task(
        &mut self,
        session_id: &str,
        task_id: &str,
        result: &str,
    ) -> Result<()> {
        debug!("Completing task: {}", task_id);
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        for task in &mut session.task_board {
            if task.id == task_id {
                task.status = TaskStatus::Done;
                task.result = Some(result.to_string());
                break;
            }
        }

        self.save().await?;
        info!("Task completed: {}", task_id);
        Ok(())
    }

    /// Add a task to the board
    pub async fn add_task(
        &mut self,
        session_id: &str,
        mut task: TeamTask,
    ) -> Result<String> {
        debug!("Adding task to session: {}", session_id);
        let task_id = uuid::Uuid::new_v4().to_string();
        task.id = task_id.clone();

        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        session.task_board.push(task);
        self.save().await?;
        Ok(task_id)
    }

    /// Write to shared memory
    pub async fn write_shared_memory(
        &mut self,
        session_id: &str,
        key: &str,
        value: &str,
        author: &str,
    ) -> Result<()> {
        debug!("Writing to shared memory: {}", key);
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        let entry = SharedMemoryEntry {
            key: key.to_string(),
            value: value.to_string(),
            written_by: author.to_string(),
            timestamp: Utc::now(),
        };

        // Update or insert
        if let Some(existing) = session.shared_memory.iter_mut().find(|e| e.key == key) {
            *existing = entry;
        } else {
            session.shared_memory.push(entry);
        }

        self.save().await?;
        Ok(())
    }

    /// Read from shared memory
    pub async fn read_shared_memory(
        &self,
        session_id: &str,
        key: &str,
    ) -> Option<SharedMemoryEntry> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .and_then(|s| s.shared_memory.iter().find(|e| e.key == key).cloned())
    }

    /// Get a session
    pub async fn get_session(&self, session_id: &str) -> Option<TeamSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<TeamSession> {
        self.sessions
            .read()
            .await
            .values()
            .cloned()
            .collect()
    }

    /// Get session progress
    pub async fn get_progress(&self, session_id: &str) -> Result<TeamProgress> {
        debug!("Getting progress for session: {}", session_id);
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        let total_tasks = session.task_board.len();
        let completed = session
            .task_board
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count();
        let in_progress = session
            .task_board
            .iter()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();
        let blocked = session
            .task_board
            .iter()
            .filter(|t| t.status == TaskStatus::Blocked)
            .count();

        let total_cost = session.members.iter().map(|m| m.cost).sum();
        let total_tokens = session.members.iter().map(|m| m.tokens_used).sum();

        let estimated_completion = if total_tasks > 0 {
            Some(completed as f64 / total_tasks as f64 * 100.0)
        } else {
            None
        };

        Ok(TeamProgress {
            total_tasks,
            completed,
            in_progress,
            blocked,
            total_cost,
            total_tokens,
            estimated_completion,
        })
    }

    /// End a session
    pub async fn end_session(&mut self, session_id: &str) -> Result<TeamSession> {
        debug!("Ending session: {}", session_id);
        let mut sessions = self.sessions.write().await;
        let mut session = sessions
            .remove(session_id)
            .ok_or_else(|| HiveCodeError::NotFound(format!("Session {} not found", session_id)))?;

        session.status = TeamStatus::Completed;
        self.save().await?;
        info!("Session ended: {}", session_id);
        Ok(session)
    }

    /// Save sessions to storage
    async fn save(&self) -> Result<()> {
        debug!("Saving team sessions");
        // In production, would persist to disk
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_role_display() {
        assert_eq!(TeamRole::Coordinator.to_string(), "Coordinator");
        assert_eq!(TeamRole::Implementer.to_string(), "Implementer");
        assert_eq!(TeamRole::Reviewer.to_string(), "Reviewer");
    }

    #[test]
    fn test_member_status_display() {
        assert_eq!(MemberStatus::Idle.to_string(), "idle");
        assert_eq!(MemberStatus::Working.to_string(), "working");
        assert_eq!(MemberStatus::Done.to_string(), "done");
    }

    #[test]
    fn test_team_status_display() {
        assert_eq!(TeamStatus::Planning.to_string(), "planning");
        assert_eq!(TeamStatus::InProgress.to_string(), "in_progress");
        assert_eq!(TeamStatus::Completed.to_string(), "completed");
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Todo.to_string(), "todo");
        assert_eq!(TaskStatus::InProgress.to_string(), "in_progress");
        assert_eq!(TaskStatus::Done.to_string(), "done");
    }

    #[tokio::test]
    async fn test_team_coordinator_creation() {
        let coordinator = TeamCoordinator::new(None).await.unwrap();
        let sessions = coordinator.list_sessions().await;
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_create_session() {
        let mut coordinator = TeamCoordinator::new(None).await.unwrap();
        let session_id = coordinator
            .create_session("Test Team", "Build a feature")
            .await
            .unwrap();
        assert!(!session_id.is_empty());
    }

    #[tokio::test]
    async fn test_add_member() {
        let mut coordinator = TeamCoordinator::new(None).await.unwrap();
        let session_id = coordinator
            .create_session("Test Team", "Build a feature")
            .await
            .unwrap();
        let member_id = coordinator
            .add_member(&session_id, TeamRole::Implementer, "gpt-4")
            .await
            .unwrap();
        assert!(!member_id.is_empty());
    }

    #[tokio::test]
    async fn test_session_progress() {
        let mut coordinator = TeamCoordinator::new(None).await.unwrap();
        let session_id = coordinator
            .create_session("Test Team", "Build a feature")
            .await
            .unwrap();
        let progress = coordinator.get_progress(&session_id).await.unwrap();
        assert_eq!(progress.total_tasks, 0);
        assert_eq!(progress.completed, 0);
    }

    #[tokio::test]
    async fn test_shared_memory() {
        let mut coordinator = TeamCoordinator::new(None).await.unwrap();
        let session_id = coordinator
            .create_session("Test Team", "Build a feature")
            .await
            .unwrap();
        coordinator
            .write_shared_memory(&session_id, "test_key", "test_value", "agent1")
            .await
            .unwrap();
        let entry = coordinator.read_shared_memory(&session_id, "test_key").await;
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().value, "test_value");
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let mut coordinator = TeamCoordinator::new(None).await.unwrap();
        coordinator
            .create_session("Team 1", "Task 1")
            .await
            .unwrap();
        coordinator
            .create_session("Team 2", "Task 2")
            .await
            .unwrap();
        let sessions = coordinator.list_sessions().await;
        assert_eq!(sessions.len(), 2);
    }
}

//! Multi-agent spawning and management system
//!
//! Allows HiveCode to spawn and manage multiple sub-agents for parallel tasks,
//! each with their own state, messages, and execution context.

use crate::types::Message;
use crate::error::{HiveCodeError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Types of agents that can be spawned
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    /// General purpose reasoning agent
    GeneralPurpose,
    /// Agent for creating and executing plans
    Plan,
    /// Agent for code exploration and analysis
    Explore,
    /// Agent for verifying work and running tests
    Verify,
    /// Agent specialized in code review
    CodeReview,
    /// Agent for security analysis
    SecurityReview,
    /// Custom agent type with custom name
    Custom(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::GeneralPurpose => write!(f, "general_purpose"),
            AgentType::Plan => write!(f, "plan"),
            AgentType::Explore => write!(f, "explore"),
            AgentType::Verify => write!(f, "verify"),
            AgentType::CodeReview => write!(f, "code_review"),
            AgentType::SecurityReview => write!(f, "security_review"),
            AgentType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Status of a running agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is currently running
    Running,
    /// Agent completed successfully
    Completed,
    /// Agent failed with an error
    Failed,
    /// Agent was cancelled
    Cancelled,
}

/// A running sub-agent instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgent {
    /// Unique agent identifier
    pub id: String,
    /// Type of agent
    pub agent_type: AgentType,
    /// Human-readable name
    pub name: String,
    /// Current status
    pub status: AgentStatus,
    /// Messages from this agent
    pub messages: Vec<Message>,
    /// When the agent was spawned
    pub spawned_at: String,
    /// ID of the parent agent/conversation (if any)
    pub parent_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Tokens consumed by this agent
    pub total_tokens: u64,
}

impl SubAgent {
    /// Create a new sub-agent
    pub fn new(agent_type: AgentType, name: impl Into<String>, parent_id: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            agent_type,
            name: name.into(),
            status: AgentStatus::Running,
            messages: Vec::new(),
            spawned_at: Utc::now().to_rfc3339(),
            parent_id,
            error: None,
            total_tokens: 0,
        }
    }

    /// Add a message to this agent
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Mark this agent as completed
    pub fn complete(&mut self) {
        self.status = AgentStatus::Completed;
    }

    /// Mark this agent as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = AgentStatus::Failed;
        self.error = Some(error.into());
    }

    /// Cancel this agent
    pub fn cancel(&mut self) {
        self.status = AgentStatus::Cancelled;
    }
}

/// Manages all running sub-agents in the system
pub struct AgentManager {
    agents: Arc<RwLock<HashMap<String, SubAgent>>>,
    max_concurrent: usize,
}

impl AgentManager {
    /// Create a new agent manager
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    /// Create with default max concurrent agents (10)
    pub fn default() -> Self {
        Self::new(10)
    }

    /// Spawn a new agent
    pub async fn spawn_agent(
        &self,
        agent_type: AgentType,
        name: impl Into<String>,
        parent_id: Option<String>,
    ) -> Result<String> {
        let mut agents = self.agents.write().await;

        // Check concurrent limit
        let running_count = agents.values()
            .filter(|a| a.status == AgentStatus::Running)
            .count();

        if running_count >= self.max_concurrent {
            return Err(HiveCodeError::Internal(format!(
                "Cannot spawn agent: max concurrent limit ({}) reached",
                self.max_concurrent
            )));
        }

        let agent = SubAgent::new(agent_type, name, parent_id);
        let agent_id = agent.id.clone();
        agents.insert(agent_id.clone(), agent);

        debug!("Spawned agent: {} ({})", agent_id, agent_type);
        Ok(agent_id)
    }

    /// Get an agent by ID
    pub async fn get_agent(&self, id: &str) -> Result<Option<SubAgent>> {
        Ok(self.agents.read().await.get(id).cloned())
    }

    /// Get all agents
    pub async fn list_agents(&self) -> Result<Vec<SubAgent>> {
        Ok(self.agents
            .read()
            .await
            .values()
            .cloned()
            .collect())
    }

    /// Get all agents of a specific type
    pub async fn list_agents_by_type(&self, agent_type: AgentType) -> Result<Vec<SubAgent>> {
        Ok(self.agents
            .read()
            .await
            .values()
            .filter(|a| a.agent_type == agent_type)
            .cloned()
            .collect())
    }

    /// Get running agents
    pub async fn list_running_agents(&self) -> Result<Vec<SubAgent>> {
        Ok(self.agents
            .read()
            .await
            .values()
            .filter(|a| a.status == AgentStatus::Running)
            .cloned()
            .collect())
    }

    /// Cancel an agent
    pub async fn cancel_agent(&self, id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;

        if let Some(agent) = agents.get_mut(id) {
            agent.cancel();
            debug!("Cancelled agent: {}", id);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Agent not found: {}", id)))
        }
    }

    /// Send a message to an agent
    pub async fn send_message_to_agent(
        &self,
        id: &str,
        message: Message,
    ) -> Result<()> {
        let mut agents = self.agents.write().await;

        if let Some(agent) = agents.get_mut(id) {
            if agent.status != AgentStatus::Running {
                return Err(HiveCodeError::Internal(
                    format!("Cannot send message to non-running agent: {}", agent.status as u8)
                ));
            }
            agent.add_message(message);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Agent not found: {}", id)))
        }
    }

    /// Get all output messages from an agent
    pub async fn get_agent_output(&self, id: &str) -> Result<Vec<Message>> {
        match self.agents.read().await.get(id) {
            Some(agent) => Ok(agent.messages.clone()),
            None => Err(HiveCodeError::NotFound(format!("Agent not found: {}", id))),
        }
    }

    /// Complete an agent
    pub async fn complete_agent(&self, id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;

        if let Some(agent) = agents.get_mut(id) {
            agent.complete();
            debug!("Completed agent: {}", id);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Agent not found: {}", id)))
        }
    }

    /// Mark an agent as failed
    pub async fn fail_agent(&self, id: &str, error: impl Into<String>) -> Result<()> {
        let mut agents = self.agents.write().await;

        if let Some(agent) = agents.get_mut(id) {
            agent.fail(error);
            warn!("Agent failed: {}", id);
            Ok(())
        } else {
            Err(HiveCodeError::NotFound(format!("Agent not found: {}", id)))
        }
    }

    /// Clean up completed and failed agents
    pub async fn cleanup_completed(&self) -> Result<usize> {
        let mut agents = self.agents.write().await;
        let initial_count = agents.len();

        agents.retain(|_, agent| {
            agent.status == AgentStatus::Running || agent.status == AgentStatus::Cancelled
        });

        let removed = initial_count - agents.len();
        debug!("Cleaned up {} completed/failed agents", removed);
        Ok(removed)
    }

    /// Get agent count
    pub async fn agent_count(&self) -> Result<usize> {
        Ok(self.agents.read().await.len())
    }
}

impl Clone for AgentManager {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            max_concurrent: self.max_concurrent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_agent() {
        let manager = AgentManager::default();
        let agent_id = manager
            .spawn_agent(AgentType::GeneralPurpose, "test_agent", None)
            .await
            .unwrap();

        assert!(!agent_id.is_empty());

        let agent = manager.get_agent(&agent_id).await.unwrap();
        assert!(agent.is_some());
        let agent = agent.unwrap();
        assert_eq!(agent.status, AgentStatus::Running);
    }

    #[tokio::test]
    async fn test_list_agents() {
        let manager = AgentManager::default();
        manager
            .spawn_agent(AgentType::GeneralPurpose, "agent1", None)
            .await
            .unwrap();
        manager
            .spawn_agent(AgentType::Plan, "agent2", None)
            .await
            .unwrap();

        let agents = manager.list_agents().await.unwrap();
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_cancel_agent() {
        let manager = AgentManager::default();
        let agent_id = manager
            .spawn_agent(AgentType::GeneralPurpose, "agent", None)
            .await
            .unwrap();

        manager.cancel_agent(&agent_id).await.unwrap();

        let agent = manager.get_agent(&agent_id).await.unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_concurrent_limit() {
        let manager = AgentManager::new(2);

        manager
            .spawn_agent(AgentType::GeneralPurpose, "agent1", None)
            .await
            .unwrap();
        manager
            .spawn_agent(AgentType::GeneralPurpose, "agent2", None)
            .await
            .unwrap();

        let result = manager
            .spawn_agent(AgentType::GeneralPurpose, "agent3", None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_complete_agent() {
        let manager = AgentManager::default();
        let agent_id = manager
            .spawn_agent(AgentType::GeneralPurpose, "agent", None)
            .await
            .unwrap();

        manager.complete_agent(&agent_id).await.unwrap();

        let agent = manager.get_agent(&agent_id).await.unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Completed);
    }

    #[tokio::test]
    async fn test_cleanup_completed() {
        let manager = AgentManager::default();

        let agent1 = manager
            .spawn_agent(AgentType::GeneralPurpose, "agent1", None)
            .await
            .unwrap();
        let agent2 = manager
            .spawn_agent(AgentType::GeneralPurpose, "agent2", None)
            .await
            .unwrap();

        manager.complete_agent(&agent1).await.unwrap();
        manager.complete_agent(&agent2).await.unwrap();

        let cleaned = manager.cleanup_completed().await.unwrap();
        assert_eq!(cleaned, 2);

        let agents = manager.list_agents().await.unwrap();
        assert_eq!(agents.len(), 0);
    }
}

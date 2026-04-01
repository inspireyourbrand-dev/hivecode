//! Tauri commands for agent spawning and management

use hivecode_core::{AgentManager, AgentType, SubAgent};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request to spawn an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnAgentRequest {
    /// Type of agent to spawn
    pub agent_type: String,
    /// Human-readable name for the agent
    pub name: String,
    /// Parent agent/conversation ID (optional)
    pub parent_id: Option<String>,
}

/// Response with agent details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub status: String,
    pub spawned_at: String,
    pub parent_id: Option<String>,
}

impl From<SubAgent> for AgentResponse {
    fn from(agent: SubAgent) -> Self {
        Self {
            id: agent.id,
            name: agent.name,
            agent_type: agent.agent_type.to_string(),
            status: format!("{:?}", agent.status),
            spawned_at: agent.spawned_at,
            parent_id: agent.parent_id,
        }
    }
}

/// Spawn a new agent
#[tauri::command]
pub async fn spawn_agent(
    request: SpawnAgentRequest,
    manager: State<'_, AgentManager>,
) -> Result<AgentResponse, String> {
    let agent_type = match request.agent_type.as_str() {
        "general_purpose" => AgentType::GeneralPurpose,
        "plan" => AgentType::Plan,
        "explore" => AgentType::Explore,
        "verify" => AgentType::Verify,
        "code_review" => AgentType::CodeReview,
        "security_review" => AgentType::SecurityReview,
        custom => AgentType::Custom(custom.to_string()),
    };

    let agent_id = manager
        .spawn_agent(agent_type, request.name, request.parent_id)
        .await
        .map_err(|e| e.to_string())?;

    let agent = manager
        .get_agent(&agent_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to retrieve spawned agent".to_string())?;

    Ok(agent.into())
}

/// List all agents
#[tauri::command]
pub async fn list_agents(manager: State<'_, AgentManager>) -> Result<Vec<AgentResponse>, String> {
    let agents = manager
        .list_agents()
        .await
        .map_err(|e| e.to_string())?;

    Ok(agents.into_iter().map(|a| a.into()).collect())
}

/// Get a specific agent
#[tauri::command]
pub async fn get_agent(
    id: String,
    manager: State<'_, AgentManager>,
) -> Result<Option<AgentResponse>, String> {
    let agent = manager
        .get_agent(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(agent.map(|a| a.into()))
}

/// Cancel an agent
#[tauri::command]
pub async fn cancel_agent(
    id: String,
    manager: State<'_, AgentManager>,
) -> Result<(), String> {
    manager
        .cancel_agent(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Get agent output messages
#[tauri::command]
pub async fn get_agent_output(
    id: String,
    manager: State<'_, AgentManager>,
) -> Result<Vec<String>, String> {
    let messages = manager
        .get_agent_output(&id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(messages
        .iter()
        .map(|m| m.get_text())
        .collect())
}

/// List agents by type
#[tauri::command]
pub async fn list_agents_by_type(
    agent_type: String,
    manager: State<'_, AgentManager>,
) -> Result<Vec<AgentResponse>, String> {
    let agent_type = match agent_type.as_str() {
        "general_purpose" => AgentType::GeneralPurpose,
        "plan" => AgentType::Plan,
        "explore" => AgentType::Explore,
        "verify" => AgentType::Verify,
        "code_review" => AgentType::CodeReview,
        "security_review" => AgentType::SecurityReview,
        custom => AgentType::Custom(custom.to_string()),
    };

    let agents = manager
        .list_agents_by_type(agent_type)
        .await
        .map_err(|e| e.to_string())?;

    Ok(agents.into_iter().map(|a| a.into()).collect())
}

/// Complete an agent
#[tauri::command]
pub async fn complete_agent(
    id: String,
    manager: State<'_, AgentManager>,
) -> Result<(), String> {
    manager
        .complete_agent(&id)
        .await
        .map_err(|e| e.to_string())
}

/// Get running agents count
#[tauri::command]
pub async fn get_running_agents_count(
    manager: State<'_, AgentManager>,
) -> Result<usize, String> {
    let agents = manager
        .list_running_agents()
        .await
        .map_err(|e| e.to_string())?;

    Ok(agents.len())
}

//! Tauri commands for plan mode

use hivecode_core::{Plan, PlanMode, PlanStep, PlanStepStatus};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request to enter plan mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterPlanModeRequest {
    pub title: String,
    pub description: Option<String>,
}

/// Request to add a step to the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPlanStepRequest {
    pub description: String,
    pub estimated_tokens: Option<u64>,
    pub files_involved: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

/// Request to update a plan step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlanStepRequest {
    pub step_id: String,
    pub status: String,
}

/// Response with plan details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub steps_count: usize,
    pub total_estimated_tokens: u64,
    pub created_at: String,
    pub modified_at: String,
}

impl From<Plan> for PlanResponse {
    fn from(plan: Plan) -> Self {
        Self {
            id: plan.id,
            title: plan.title,
            description: plan.description,
            status: plan.status.to_string(),
            steps_count: plan.steps.len(),
            total_estimated_tokens: plan.total_estimated_tokens(),
            created_at: plan.created_at,
            modified_at: plan.modified_at,
        }
    }
}

/// Response with plan step details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStepResponse {
    pub id: String,
    pub description: String,
    pub status: String,
    pub estimated_tokens: Option<u64>,
    pub files_involved: Vec<String>,
    pub dependencies: Vec<String>,
}

impl From<PlanStep> for PlanStepResponse {
    fn from(step: PlanStep) -> Self {
        Self {
            id: step.id,
            description: step.description,
            status: step.status.to_string(),
            estimated_tokens: step.estimated_tokens,
            files_involved: step.files_involved,
            dependencies: step.dependencies,
        }
    }
}

/// Enter plan mode
#[tauri::command]
pub async fn enter_plan_mode(
    request: EnterPlanModeRequest,
    plan_mode: State<'_, PlanMode>,
) -> Result<PlanResponse, String> {
    let mut plan = plan_mode
        .enter_plan_mode(&request.title)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(desc) = request.description {
        plan.description = Some(desc);
    }

    Ok(plan.into())
}

/// Exit plan mode
#[tauri::command]
pub async fn exit_plan_mode(plan_mode: State<'_, PlanMode>) -> Result<Option<PlanResponse>, String> {
    let plan = plan_mode
        .exit_plan_mode()
        .await
        .map_err(|e| e.to_string())?;

    Ok(plan.map(|p| p.into()))
}

/// Add a step to the current plan
#[tauri::command]
pub async fn add_plan_step(
    request: AddPlanStepRequest,
    plan_mode: State<'_, PlanMode>,
) -> Result<PlanStepResponse, String> {
    let step_id = plan_mode
        .add_step(&request.description)
        .await
        .map_err(|e| e.to_string())?;

    let plan = plan_mode
        .get_plan()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No active plan".to_string())?;

    let step = plan
        .steps
        .into_iter()
        .find(|s| s.id == step_id)
        .ok_or_else(|| "Step not found".to_string())?;

    Ok(step.into())
}

/// Update a plan step status
#[tauri::command]
pub async fn update_plan_step(
    request: UpdatePlanStepRequest,
    plan_mode: State<'_, PlanMode>,
) -> Result<(), String> {
    let status = match request.status.as_str() {
        "pending" => PlanStepStatus::Pending,
        "in_progress" => PlanStepStatus::InProgress,
        "completed" => PlanStepStatus::Completed,
        "skipped" => PlanStepStatus::Skipped,
        "failed" => PlanStepStatus::Failed,
        _ => return Err(format!("Unknown status: {}", request.status)),
    };

    plan_mode
        .update_step(&request.step_id, status)
        .await
        .map_err(|e| e.to_string())
}

/// Get the current plan
#[tauri::command]
pub async fn get_plan(plan_mode: State<'_, PlanMode>) -> Result<Option<PlanResponse>, String> {
    let plan = plan_mode
        .get_plan()
        .await
        .map_err(|e| e.to_string())?;

    Ok(plan.map(|p| p.into()))
}

/// Check if plan mode is active
#[tauri::command]
pub async fn is_plan_mode_active(plan_mode: State<'_, PlanMode>) -> Result<bool, String> {
    plan_mode
        .is_active()
        .await
        .map_err(|e| e.to_string())
}

/// Cancel the current plan
#[tauri::command]
pub async fn cancel_plan(plan_mode: State<'_, PlanMode>) -> Result<Option<PlanResponse>, String> {
    let plan = plan_mode
        .cancel_plan()
        .await
        .map_err(|e| e.to_string())?;

    Ok(plan.map(|p| p.into()))
}

/// Get all steps in the current plan
#[tauri::command]
pub async fn get_plan_steps(plan_mode: State<'_, PlanMode>) -> Result<Vec<PlanStepResponse>, String> {
    let plan = plan_mode
        .get_plan()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No active plan".to_string())?;

    Ok(plan
        .steps
        .into_iter()
        .map(|s| s.into())
        .collect())
}

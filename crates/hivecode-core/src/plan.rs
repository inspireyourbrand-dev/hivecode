//! Plan mode system for HiveCode
//!
//! Enables users to enter "plan mode" where they can outline a multi-step plan
//! before execution, with tracking of dependencies, estimated tokens, and involved files.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use tracing::debug;
use crate::error::{HiveCodeError, Result};

/// Status of an individual plan step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    /// Step not yet started
    Pending,
    /// Step is currently being executed
    InProgress,
    /// Step completed successfully
    Completed,
    /// Step was skipped
    Skipped,
    /// Step failed
    Failed,
}

impl std::fmt::Display for PlanStepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanStepStatus::Pending => write!(f, "pending"),
            PlanStepStatus::InProgress => write!(f, "in_progress"),
            PlanStepStatus::Completed => write!(f, "completed"),
            PlanStepStatus::Skipped => write!(f, "skipped"),
            PlanStepStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A single step in a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Unique step identifier
    pub id: String,
    /// Human-readable description of what this step does
    pub description: String,
    /// Current status of this step
    pub status: PlanStepStatus,
    /// Sub-steps (for hierarchical planning)
    pub substeps: Vec<PlanStep>,
    /// Estimated tokens this step will consume
    pub estimated_tokens: Option<u64>,
    /// Files involved in this step
    pub files_involved: Vec<String>,
    /// IDs of steps this one depends on
    pub dependencies: Vec<String>,
    /// When this step was created
    pub created_at: String,
    /// Error message if failed
    pub error: Option<String>,
}

impl PlanStep {
    /// Create a new plan step
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            status: PlanStepStatus::Pending,
            substeps: Vec::new(),
            estimated_tokens: None,
            files_involved: Vec::new(),
            dependencies: Vec::new(),
            created_at: Utc::now().to_rfc3339(),
            error: None,
        }
    }

    /// Add a dependency on another step
    pub fn depends_on(mut self, step_id: impl Into<String>) -> Self {
        self.dependencies.push(step_id.into());
        self
    }

    /// Add an involved file
    pub fn with_file(mut self, file_path: impl Into<String>) -> Self {
        self.files_involved.push(file_path.into());
        self
    }

    /// Set estimated tokens
    pub fn with_estimated_tokens(mut self, tokens: u64) -> Self {
        self.estimated_tokens = Some(tokens);
        self
    }

    /// Add a substep
    pub fn with_substep(mut self, substep: PlanStep) -> Self {
        self.substeps.push(substep);
        self
    }
}

/// Status of the entire plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    /// Plan is being created
    Draft,
    /// Plan is active and being executed
    Active,
    /// Plan execution completed
    Completed,
    /// Plan was cancelled
    Cancelled,
    /// Plan failed
    Failed,
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanStatus::Draft => write!(f, "draft"),
            PlanStatus::Active => write!(f, "active"),
            PlanStatus::Completed => write!(f, "completed"),
            PlanStatus::Cancelled => write!(f, "cancelled"),
            PlanStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A multi-step plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Unique plan identifier
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Description of the plan
    pub description: Option<String>,
    /// Steps in the plan
    pub steps: Vec<PlanStep>,
    /// Overall plan status
    pub status: PlanStatus,
    /// When the plan was created
    pub created_at: String,
    /// When the plan was last modified
    pub modified_at: String,
}

impl Plan {
    /// Create a new plan
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            description: None,
            steps: Vec::new(),
            status: PlanStatus::Draft,
            created_at: now.clone(),
            modified_at: now,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: PlanStep) -> String {
        let step_id = step.id.clone();
        self.steps.push(step);
        self.modified_at = Utc::now().to_rfc3339();
        step_id
    }

    /// Get total estimated tokens for the plan
    pub fn total_estimated_tokens(&self) -> u64 {
        self.steps
            .iter()
            .map(|step| step.estimated_tokens.unwrap_or(0))
            .sum()
    }

    /// Get all files involved in the plan
    pub fn all_files_involved(&self) -> Vec<String> {
        let mut files = Vec::new();
        for step in &self.steps {
            files.extend(step.files_involved.clone());
        }
        files.sort();
        files.dedup();
        files
    }
}

/// Plan mode state and manager
pub struct PlanMode {
    state: Arc<RwLock<PlanModeState>>,
}

struct PlanModeState {
    active: bool,
    plan: Option<Plan>,
}

impl PlanMode {
    /// Create a new plan mode manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PlanModeState {
                active: false,
                plan: None,
            })),
        }
    }

    /// Enter plan mode and create a new plan
    pub async fn enter_plan_mode(&self, title: impl Into<String>) -> Result<Plan> {
        let mut state = self.state.write().await;

        if state.active {
            return Err(HiveCodeError::Internal(
                "Already in plan mode".to_string(),
            ));
        }

        let plan = Plan::new(title);
        state.active = true;
        state.plan = Some(plan.clone());

        debug!("Entered plan mode with plan: {}", plan.id);
        Ok(plan)
    }

    /// Exit plan mode and return the completed plan
    pub async fn exit_plan_mode(&self) -> Result<Option<Plan>> {
        let mut state = self.state.write().await;

        if !state.active {
            return Err(HiveCodeError::Internal(
                "Not in plan mode".to_string(),
            ));
        }

        state.active = false;
        if let Some(mut plan) = state.plan.take() {
            plan.status = PlanStatus::Completed;
            plan.modified_at = Utc::now().to_rfc3339();
            debug!("Exited plan mode, plan completed: {}", plan.id);
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    /// Add a step to the current plan
    pub async fn add_step(&self, description: impl Into<String>) -> Result<String> {
        let mut state = self.state.write().await;

        if !state.active {
            return Err(HiveCodeError::Internal(
                "Not in plan mode".to_string(),
            ));
        }

        if let Some(plan) = &mut state.plan {
            let step = PlanStep::new(description);
            let step_id = step.id.clone();
            plan.steps.push(step);
            plan.modified_at = Utc::now().to_rfc3339();
            Ok(step_id)
        } else {
            Err(HiveCodeError::Internal(
                "No active plan".to_string(),
            ))
        }
    }

    /// Update a step's status
    pub async fn update_step(&self, step_id: &str, status: PlanStepStatus) -> Result<()> {
        let mut state = self.state.write().await;

        if let Some(plan) = &mut state.plan {
            if let Some(step) = Self::find_step_mut(&mut plan.steps, step_id) {
                step.status = status;
                plan.modified_at = Utc::now().to_rfc3339();
                return Ok(());
            }
        }

        Err(HiveCodeError::NotFound(format!("Step not found: {}", step_id)))
    }

    /// Get the current plan
    pub async fn get_plan(&self) -> Result<Option<Plan>> {
        Ok(self.state.read().await.plan.clone())
    }

    /// Check if plan mode is active
    pub async fn is_active(&self) -> Result<bool> {
        Ok(self.state.read().await.active)
    }

    /// Cancel the current plan
    pub async fn cancel_plan(&self) -> Result<Option<Plan>> {
        let mut state = self.state.write().await;

        if let Some(mut plan) = state.plan.take() {
            plan.status = PlanStatus::Cancelled;
            plan.modified_at = Utc::now().to_rfc3339();
            state.active = false;
            debug!("Plan cancelled: {}", plan.id);
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }

    /// Find a step by ID (mutable)
    fn find_step_mut(steps: &mut [PlanStep], step_id: &str) -> Option<&mut PlanStep> {
        for step in steps.iter_mut() {
            if step.id == step_id {
                return Some(step);
            }
            if let Some(found) = Self::find_step_mut(&mut step.substeps, step_id) {
                return Some(found);
            }
        }
        None
    }
}

impl Clone for PlanMode {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enter_plan_mode() {
        let plan_mode = PlanMode::new();
        let plan = plan_mode.enter_plan_mode("Test Plan").await.unwrap();

        assert_eq!(plan.title, "Test Plan");
        assert_eq!(plan.status, PlanStatus::Draft);
        assert!(plan_mode.is_active().await.unwrap());
    }

    #[tokio::test]
    async fn test_add_step() {
        let plan_mode = PlanMode::new();
        plan_mode.enter_plan_mode("Test Plan").await.unwrap();

        let step_id = plan_mode.add_step("Step 1").await.unwrap();
        assert!(!step_id.is_empty());

        let plan = plan_mode.get_plan().await.unwrap().unwrap();
        assert_eq!(plan.steps.len(), 1);
    }

    #[tokio::test]
    async fn test_update_step_status() {
        let plan_mode = PlanMode::new();
        plan_mode.enter_plan_mode("Test Plan").await.unwrap();

        let step_id = plan_mode.add_step("Step 1").await.unwrap();
        plan_mode
            .update_step(&step_id, PlanStepStatus::InProgress)
            .await
            .unwrap();

        let plan = plan_mode.get_plan().await.unwrap().unwrap();
        assert_eq!(plan.steps[0].status, PlanStepStatus::InProgress);
    }

    #[tokio::test]
    async fn test_exit_plan_mode() {
        let plan_mode = PlanMode::new();
        plan_mode.enter_plan_mode("Test Plan").await.unwrap();
        plan_mode.add_step("Step 1").await.unwrap();

        let plan = plan_mode.exit_plan_mode().await.unwrap().unwrap();
        assert_eq!(plan.status, PlanStatus::Completed);
        assert!(!plan_mode.is_active().await.unwrap());
    }

    #[tokio::test]
    async fn test_cannot_enter_plan_twice() {
        let plan_mode = PlanMode::new();
        plan_mode.enter_plan_mode("Plan 1").await.unwrap();

        let result = plan_mode.enter_plan_mode("Plan 2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cancel_plan() {
        let plan_mode = PlanMode::new();
        plan_mode.enter_plan_mode("Test Plan").await.unwrap();
        plan_mode.add_step("Step 1").await.unwrap();

        let plan = plan_mode.cancel_plan().await.unwrap().unwrap();
        assert_eq!(plan.status, PlanStatus::Cancelled);
        assert!(!plan_mode.is_active().await.unwrap());
    }

    #[test]
    fn test_plan_step_builder() {
        let step = PlanStep::new("Do something")
            .with_estimated_tokens(100)
            .with_file("src/main.rs")
            .depends_on("step-123");

        assert_eq!(step.description, "Do something");
        assert_eq!(step.estimated_tokens, Some(100));
        assert_eq!(step.files_involved.len(), 1);
        assert_eq!(step.dependencies.len(), 1);
    }

    #[test]
    fn test_plan_total_tokens() {
        let mut plan = Plan::new("Test Plan");
        plan.add_step(PlanStep::new("Step 1").with_estimated_tokens(100));
        plan.add_step(PlanStep::new("Step 2").with_estimated_tokens(200));

        assert_eq!(plan.total_estimated_tokens(), 300);
    }
}

//! HiveCode Core Engine
//!
//! Provides the foundation for HiveCode's operation, including:
//! - State management with thread-safe access
//! - Configuration loading from TOML with environment variable expansion
//! - Conversation engine with token tracking and context window management
//! - Event system for reactive state changes
//! - Type definitions and error handling
//! - Multi-agent spawning and management
//! - Plan mode for step-by-step task planning
//! - Context window and token cost management
//! - Image processing and vision API support
//! - PDF text extraction
//! - Conversation compaction and context management
//! - Persistent memory system

pub mod agent_manager;
pub mod auth;
pub mod config;
pub mod context;
pub mod conversation;
pub mod error;
pub mod events;
pub mod history;
pub mod plan;
pub mod state;
pub mod types;
pub mod plugins;
pub mod updater;
pub mod image;
pub mod pdf;
pub mod compact;
pub mod memory;

pub use agent_manager::{AgentManager, AgentStatus, AgentType, SubAgent};
pub use auth::{AuthManager, AuthMode, AuthProfile, AuthProfileSummary};
pub use config::HiveConfig;
pub use context::{ContextManager, CostSummary, ModelPricing, TokenTracker, TokenUsage};
pub use conversation::{ConversationEngine, ConversationState};
pub use error::{HiveCodeError, Result};
pub use events::{EventBroadcaster, StateEvent};
pub use history::{Session, SessionManager, SessionSummary};
pub use plan::{Plan, PlanMode, PlanStatus, PlanStep, PlanStepStatus};
pub use state::AppState;
pub use types::{
    AppSessionState, ContentBlock, ConversationMetadata, Message, MessageRole, ProviderInfo,
    TokenCount,
};
pub use plugins::{Plugin, PluginManager, PluginManifest, PluginType, PluginStatus};
pub use updater::{UpdateManager, UpdateInfo, UpdateChannel};
pub use image::{ImageProcessor, ProcessedImage, ImageConstraints};
pub use pdf::{PdfProcessor, PdfInfo, PdfExtraction};
pub use compact::{ConversationCompactor, CompactSummary, CompactOptions};
pub use memory::{MemoryManager, MemoryEntry, MemoryCategory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_compiles() {
        // Placeholder test to ensure workspace compiles
        assert!(true);
    }
}

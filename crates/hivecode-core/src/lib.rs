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
//! - Intelligent model routing for cost optimization
//! - Git-aware context discovery for automatic file inclusion
//! - Prompt caching for cost reduction
//! - Conversation branching and exploration
//! - Token-efficient tool output compression
//! - Real-time streaming file diffs
//! - Session recording and replay
//! - Extended thinking / chain-of-thought reasoning
//! - Offline mode with local provider fallback
//! - Cost analysis and optimization recommendations
//! - Voice mode with speech-to-text integration
//! - IDE bridge for VS Code and other editors
//! - Vim keybinding mode
//! - Team/swarm coordination for multi-agent tasks
//! - Keybinding customization and management

pub mod agent_manager;
pub mod auth;
pub mod branching;
pub mod compact;
pub mod config;
pub mod context;
pub mod conversation;
pub mod cost_optimizer;
pub mod error;
pub mod events;
pub mod git_context;
pub mod history;
pub mod hooks;
pub mod ide_bridge;
pub mod image;
pub mod keybinding_config;
pub mod memory;
pub mod model_router;
pub mod offline;
pub mod pdf;
pub mod plan;
pub mod plugins;
pub mod project_instructions;
pub mod prompt_cache;
pub mod session_replay;
pub mod state;
pub mod streaming_diff;
pub mod team;
pub mod thinking;
pub mod tool_output_compress;
pub mod types;
pub mod updater;
pub mod vim_mode;
pub mod voice;

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
pub use hooks::{
    Hook, HookManager, HookType, HookTrigger, HookAction, HookCondition, HookResult, HookContext,
    PreHookResult,
};
pub use project_instructions::{
    ProjectInstructions, ProjectInstructionsLoader, FileRestrictions, ProjectHook,
    ModelPreferences, PathAccessResult,
};
pub use model_router::{
    ModelRouter, ModelRoute, ModelTier, TaskComplexity, RoutingDecision, RoutingRequest,
    RoutingConfig, RoutingStats,
};
pub use git_context::{
    GitContextBuilder, GitContextInfo, FileChange, ChangeType, CommitInfo, RelevantFile,
    RelevanceReason,
};
pub use prompt_cache::{
    PromptCacheManager, CacheableBlock, CacheBlockType, CacheControl, CacheStats,
};
pub use branching::{
    BranchManager, ConversationBranch, BranchTree, BranchSummary, MergeResult, BranchComparison,
    BranchMetadata,
};
pub use tool_output_compress::{
    ToolOutputCompressor, CompressedOutput, CompressionConfig, CompressionStrategy,
    OutputMetadata,
};
pub use streaming_diff::{
    DiffTracker, DiffEvent, DiffEventType, FileDiff, DiffHunk, DiffLine, DiffStats,
};
pub use session_replay::{
    SessionRecorder, SessionPlayer, SessionRecording, ReplayEvent, ReplayEventType,
    RecordingSummary,
};
pub use voice::{
    VoiceManager, VoiceConfig, VoiceState, SttProvider, TranscriptionResult, TranscriptionSegment,
    AudioDevice,
};
pub use ide_bridge::{
    IdeBridge, BridgeConfig, BridgeState, BridgeMessage, TextChange, TextEdit, Position, Range,
    Diagnostic, DiagnosticSeverity, MessageLevel, CursorContext,
};
pub use vim_mode::{
    VimEngine, VimMode, VimState, CursorPosition, SearchDirection, VimOperator, VimMotion,
    VimTextObject, VimAction, VimActionType,
};
pub use team::{
    TeamCoordinator, TeamSession, TeamMember, TeamRole, MemberStatus, TeamStatus, TeamTask,
    TaskStatus, SharedMemoryEntry, TeamProgress,
};
pub use keybinding_config::{
    KeybindingManager, Keybinding, KeybindingCategory, KeybindingScheme, KeyConflict,
};
pub use thinking::{
    ThinkingManager, ThinkingBlock, ThinkingSession, ThinkingEvent, ThinkingType, ThinkingConfig,
};
pub use offline::{
    OfflineManager, OfflineConfig, OfflineStatus, ConnectivityState, ProviderHealth,
    FallbackAction, RestoreAction, ConnectivityChange,
};
pub use cost_optimizer::{
    CostOptimizer, CostAnalysis, CostRecommendation, CostBreakdown, ModelCost, UsageRecord,
    RecommendationType, Difficulty,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_compiles() {
        // Placeholder test to ensure workspace compiles
        assert!(true);
    }
}

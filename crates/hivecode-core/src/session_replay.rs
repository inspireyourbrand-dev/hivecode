//! Session replay for HiveCode
//!
//! Records complete coding sessions for playback, review, and sharing.
//! Great for teams, onboarding, and learning.

use crate::error::{HiveCodeError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// A single event in a session recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    /// When this event occurred
    pub timestamp: DateTime<Utc>,
    /// Milliseconds since session start
    pub relative_ms: u64,
    /// The event data
    pub event_type: ReplayEventType,
}

/// Types of events that can be recorded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayEventType {
    /// User sent a message
    UserMessage(String),
    /// Assistant sent a message
    AssistantMessage(String),
    /// Tool was called
    ToolCall { tool: String, input: serde_json::Value },
    /// Tool returned a result
    ToolResult { tool: String, output: String, success: bool },
    /// File was modified
    FileModified { path: PathBuf, diff: String },
    /// New file created
    FileCreated(PathBuf),
    /// File deleted
    FileDeleted(PathBuf),
    /// User switched models
    ModelSwitch { from: String, to: String },
    /// User created a new branch
    BranchCreated { branch_id: String, name: String },
    /// Part of a streamed response
    StreamChunk(String),
    /// Thinking block started
    ThinkingStart,
    /// Thinking block ended
    ThinkingEnd,
    /// Error occurred
    ErrorOccurred(String),
    /// Session paused
    SessionPaused,
    /// Session resumed
    SessionResumed,
}

/// A complete recorded session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecording {
    /// Unique recording ID
    pub id: String,
    /// User-friendly title
    pub title: String,
    /// Optional description
    pub description: Option<String>,
    /// When the recording was created
    pub created_at: DateTime<Utc>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Model used during session
    pub model_used: String,
    /// Total messages exchanged
    pub total_messages: usize,
    /// Total tool calls made
    pub total_tool_calls: usize,
    /// Total tokens consumed
    pub total_tokens: u64,
    /// Estimated cost
    pub total_cost: f64,
    /// All recorded events
    pub events: Vec<ReplayEvent>,
    /// Optional tags for organization
    pub tags: Vec<String>,
}

/// Summary of a recording (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSummary {
    /// Recording ID
    pub id: String,
    /// Title
    pub title: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Number of events
    pub event_count: usize,
    /// Model used
    pub model_used: String,
}

/// Records a coding session
pub struct SessionRecorder {
    /// Current recording (if active)
    recording: Option<SessionRecording>,
    /// When recording started
    start_time: Option<DateTime<Utc>>,
    /// Whether currently recording
    is_recording: bool,
    /// Where to store recordings
    storage_path: PathBuf,
}

impl SessionRecorder {
    /// Create a new session recorder
    pub fn new(storage_dir: Option<PathBuf>) -> Self {
        let storage_path = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".hivecode");
            path.push("recordings");
            path
        });

        Self {
            recording: None,
            start_time: None,
            is_recording: false,
            storage_path,
        }
    }

    /// Start recording a new session
    pub fn start(&mut self, title: &str, model: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        self.recording = Some(SessionRecording {
            id: id.clone(),
            title: title.to_string(),
            description: None,
            created_at: now,
            duration_ms: 0,
            model_used: model.to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: Vec::new(),
            tags: Vec::new(),
        });

        self.start_time = Some(now);
        self.is_recording = true;

        info!("Started recording session: {} ({})", title, id);
        id
    }

    /// Record an event
    pub fn record(&mut self, event_type: ReplayEventType) {
        if !self.is_recording {
            warn!("Attempted to record event while not recording");
            return;
        }

        if let Some(ref mut recording) = self.recording {
            let now = Utc::now();
            let relative_ms = self
                .start_time
                .map(|st| (now - st).num_milliseconds() as u64)
                .unwrap_or(0);

            // Update counters based on event type
            match &event_type {
                ReplayEventType::UserMessage(_) | ReplayEventType::AssistantMessage(_) => {
                    recording.total_messages += 1;
                }
                ReplayEventType::ToolCall { .. } => {
                    recording.total_tool_calls += 1;
                }
                _ => {}
            }

            recording.events.push(ReplayEvent {
                timestamp: now,
                relative_ms,
                event_type,
            });

            debug!("Recorded event at {}ms", relative_ms);
        }
    }

    /// Stop recording and save
    pub async fn stop(&mut self) -> Result<SessionRecording> {
        if let Some(mut recording) = self.recording.take() {
            self.is_recording = false;

            if let Some(start) = self.start_time {
                recording.duration_ms = (Utc::now() - start).num_milliseconds() as u64;
            }

            // Save to disk
            let _ = fs::create_dir_all(&self.storage_path).await;
            let file_path = self.storage_path.join(format!("{}.json", recording.id));

            let json = serde_json::to_string_pretty(&recording)
                .map_err(|e| HiveCodeError::SerializationError(e.to_string()))?;

            fs::write(&file_path, json)
                .await
                .map_err(|e| HiveCodeError::IOError(e.to_string()))?;

            info!(
                "Stopped recording session: {} ({} events)",
                recording.title,
                recording.events.len()
            );

            Ok(recording)
        } else {
            Err(HiveCodeError::ConversationError(
                "No recording in progress".to_string(),
            ))
        }
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.is_recording && self.recording.is_some()
    }

    /// Pause recording (keeps state, doesn't finalize)
    pub fn pause(&mut self) {
        if self.is_recording {
            self.is_recording = false;
            debug!("Recording paused");
        }
    }

    /// Resume recording
    pub fn resume(&mut self) {
        if !self.is_recording && self.recording.is_some() {
            self.is_recording = true;
            debug!("Recording resumed");
        }
    }

    /// Get current recording duration in milliseconds
    pub fn current_duration(&self) -> Option<u64> {
        self.start_time
            .map(|st| (Utc::now() - st).num_milliseconds() as u64)
    }
}

/// Plays back a recorded session
pub struct SessionPlayer {
    recording: SessionRecording,
    current_index: usize,
    playback_speed: f64,
    is_playing: bool,
}

impl SessionPlayer {
    /// Create a new player for a recording
    pub fn new(recording: SessionRecording) -> Self {
        Self {
            recording,
            current_index: 0,
            playback_speed: 1.0,
            is_playing: false,
        }
    }

    /// Load a recording from disk
    pub async fn load(recording_id: &str, storage_dir: Option<PathBuf>) -> Result<Self> {
        let storage_path = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".hivecode");
            path.push("recordings");
            path
        });

        let file_path = storage_path.join(format!("{}.json", recording_id));

        let content = fs::read_to_string(&file_path)
            .await
            .map_err(|e| HiveCodeError::IOError(e.to_string()))?;

        let recording: SessionRecording = serde_json::from_str(&content)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))?;

        debug!("Loaded recording: {}", recording_id);
        Ok(Self::new(recording))
    }

    /// List all available recordings
    pub async fn list_recordings(storage_dir: Option<PathBuf>) -> Result<Vec<RecordingSummary>> {
        let storage_path = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".hivecode");
            path.push("recordings");
            path
        });

        let mut summaries = Vec::new();

        if let Ok(mut entries) = fs::read_dir(&storage_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(path) = entry.path().canonicalize() {
                    if path.extension().map_or(false, |ext| ext == "json") {
                        if let Ok(content) = fs::read_to_string(&path).await {
                            if let Ok(recording) =
                                serde_json::from_str::<SessionRecording>(&content)
                            {
                                summaries.push(RecordingSummary {
                                    id: recording.id,
                                    title: recording.title,
                                    created_at: recording.created_at,
                                    duration_ms: recording.duration_ms,
                                    event_count: recording.events.len(),
                                    model_used: recording.model_used,
                                });
                            }
                        }
                    }
                }
            }
        }

        summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(summaries)
    }

    /// Get the next event in playback
    pub fn next_event(&mut self) -> Option<&ReplayEvent> {
        if self.current_index < self.recording.events.len() {
            let event = &self.recording.events[self.current_index];
            self.current_index += 1;
            Some(event)
        } else {
            None
        }
    }

    /// Seek to a specific timestamp
    pub fn seek_to(&mut self, relative_ms: u64) {
        self.current_index = self
            .recording
            .events
            .iter()
            .position(|e| e.relative_ms >= relative_ms)
            .unwrap_or(0);
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f64) {
        self.playback_speed = speed.max(0.25).min(4.0); // 0.25x to 4x
    }

    /// Jump to a specific event index
    pub fn jump_to(&mut self, index: usize) {
        self.current_index = index.min(self.recording.events.len());
    }

    /// Get total event count
    pub fn total_events(&self) -> usize {
        self.recording.events.len()
    }

    /// Get progress as 0.0 to 1.0
    pub fn progress(&self) -> f64 {
        if self.recording.events.is_empty() {
            0.0
        } else {
            self.current_index as f64 / self.recording.events.len() as f64
        }
    }

    /// Export recording as markdown
    pub fn export_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", self.recording.title));

        if let Some(desc) = &self.recording.description {
            md.push_str(&format!("{}\n\n", desc));
        }

        md.push_str(&format!(
            "**Model:** {} | **Duration:** {}s | **Events:** {}\n\n",
            self.recording.model_used,
            self.recording.duration_ms / 1000,
            self.recording.events.len()
        ));

        md.push_str("## Timeline\n\n");

        for (i, event) in self.recording.events.iter().enumerate() {
            let secs = event.relative_ms / 1000;
            let millis = event.relative_ms % 1000;

            md.push_str(&format!(
                "{}. **[{:02}:{:03}]** ",
                i + 1,
                secs,
                millis
            ));

            match &event.event_type {
                ReplayEventType::UserMessage(msg) => {
                    md.push_str(&format!("**User:** {}\n\n", Self::truncate(msg, 100)));
                }
                ReplayEventType::AssistantMessage(msg) => {
                    md.push_str(&format!("**Assistant:** {}\n\n", Self::truncate(msg, 100)));
                }
                ReplayEventType::ToolCall { tool, input } => {
                    md.push_str(&format!(
                        "**Tool Call:** `{}` with {}\n\n",
                        tool,
                        input.to_string()
                    ));
                }
                ReplayEventType::ToolResult { tool, success, .. } => {
                    let status = if *success { "Success" } else { "Error" };
                    md.push_str(&format!("**Tool Result:** {} ({})\n\n", tool, status));
                }
                ReplayEventType::FileModified { path, .. } => {
                    md.push_str(&format!("**File Modified:** {:?}\n\n", path));
                }
                ReplayEventType::FileCreated(path) => {
                    md.push_str(&format!("**File Created:** {:?}\n\n", path));
                }
                ReplayEventType::ModelSwitch { from, to } => {
                    md.push_str(&format!("**Model Switch:** {} → {}\n\n", from, to));
                }
                _ => {
                    md.push_str(&format!("**Event:** {:?}\n\n", event.event_type));
                }
            }
        }

        md
    }

    /// Export recording as JSON
    pub fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.recording)
            .map_err(|e| HiveCodeError::SerializationError(e.to_string()))
    }

    /// Delete a recording
    pub async fn delete_recording(recording_id: &str, storage_dir: Option<PathBuf>) -> Result<()> {
        let storage_path = storage_dir.unwrap_or_else(|| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".hivecode");
            path.push("recordings");
            path
        });

        let file_path = storage_path.join(format!("{}.json", recording_id));

        fs::remove_file(&file_path)
            .await
            .map_err(|e| HiveCodeError::IOError(e.to_string()))?;

        info!("Deleted recording: {}", recording_id);
        Ok(())
    }

    /// Helper to truncate text
    fn truncate(s: &str, max_len: usize) -> String {
        if s.len() > max_len {
            format!("{}...", &s[..max_len - 3])
        } else {
            s.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_event_creation() {
        let now = Utc::now();
        let event = ReplayEvent {
            timestamp: now,
            relative_ms: 1000,
            event_type: ReplayEventType::UserMessage("Hello".to_string()),
        };

        assert_eq!(event.relative_ms, 1000);
        assert!(matches!(event.event_type, ReplayEventType::UserMessage(_)));
    }

    #[test]
    fn test_session_recording_new() {
        let now = Utc::now();
        let recording = SessionRecording {
            id: "test".to_string(),
            title: "Test Session".to_string(),
            description: Some("A test".to_string()),
            created_at: now,
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 5,
            total_tool_calls: 2,
            total_tokens: 1000,
            total_cost: 0.05,
            events: Vec::new(),
            tags: vec!["test".to_string()],
        };

        assert_eq!(recording.total_messages, 5);
        assert_eq!(recording.total_tool_calls, 2);
        assert!(!recording.tags.is_empty());
    }

    #[test]
    fn test_session_recorder_new() {
        let recorder = SessionRecorder::new(None);
        assert!(!recorder.is_recording);
        assert!(recorder.recording.is_none());
    }

    #[test]
    fn test_session_recorder_start() {
        let mut recorder = SessionRecorder::new(None);
        let id = recorder.start("Test", "gpt-4");

        assert!(!id.is_empty());
        assert!(recorder.is_recording());
        assert!(recorder.recording.is_some());
    }

    #[test]
    fn test_session_recorder_record() {
        let mut recorder = SessionRecorder::new(None);
        recorder.start("Test", "gpt-4");
        recorder.record(ReplayEventType::UserMessage("Hi".to_string()));

        if let Some(recording) = &recorder.recording {
            assert_eq!(recording.events.len(), 1);
            assert_eq!(recording.total_messages, 1);
        }
    }

    #[test]
    fn test_session_recorder_pause_resume() {
        let mut recorder = SessionRecorder::new(None);
        recorder.start("Test", "gpt-4");
        assert!(recorder.is_recording());

        recorder.pause();
        assert!(!recorder.is_recording());

        recorder.resume();
        assert!(recorder.is_recording());
    }

    #[test]
    fn test_session_recorder_current_duration() {
        let mut recorder = SessionRecorder::new(None);
        recorder.start("Test", "gpt-4");

        let duration = recorder.current_duration();
        assert!(duration.is_some());
        assert!(duration.unwrap() >= 0);
    }

    #[test]
    fn test_recording_summary_serialization() {
        let now = Utc::now();
        let summary = RecordingSummary {
            id: "test".to_string(),
            title: "Test".to_string(),
            created_at: now,
            duration_ms: 1000,
            event_count: 5,
            model_used: "gpt-4".to_string(),
        };

        let json = serde_json::to_string(&summary).unwrap();
        let restored: RecordingSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, "test");
        assert_eq!(restored.event_count, 5);
    }

    #[test]
    fn test_session_player_new() {
        let recording = SessionRecording {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            created_at: Utc::now(),
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        let player = SessionPlayer::new(recording);
        assert_eq!(player.current_index, 0);
        assert_eq!(player.playback_speed, 1.0);
    }

    #[test]
    fn test_session_player_next_event() {
        let mut recording = SessionRecording {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            created_at: Utc::now(),
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        recording.events.push(ReplayEvent {
            timestamp: Utc::now(),
            relative_ms: 0,
            event_type: ReplayEventType::UserMessage("Hi".to_string()),
        });

        let mut player = SessionPlayer::new(recording);
        let event = player.next_event();

        assert!(event.is_some());
        assert_eq!(player.current_index, 1);
    }

    #[test]
    fn test_session_player_seek_to() {
        let mut recording = SessionRecording {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            created_at: Utc::now(),
            duration_ms: 5000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        for i in 0..5 {
            recording.events.push(ReplayEvent {
                timestamp: Utc::now(),
                relative_ms: i * 1000,
                event_type: ReplayEventType::UserMessage(format!("Message {}", i)),
            });
        }

        let mut player = SessionPlayer::new(recording);
        player.seek_to(2500);

        // Should be positioned at event closest to 2500ms
        assert!(player.current_index > 0 && player.current_index <= 5);
    }

    #[test]
    fn test_session_player_progress() {
        let mut recording = SessionRecording {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            created_at: Utc::now(),
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        for i in 0..10 {
            recording.events.push(ReplayEvent {
                timestamp: Utc::now(),
                relative_ms: i * 100,
                event_type: ReplayEventType::UserMessage(format!("Message {}", i)),
            });
        }

        let mut player = SessionPlayer::new(recording);
        assert_eq!(player.progress(), 0.0);

        player.next_event();
        assert!(player.progress() > 0.0);

        player.current_index = 10;
        assert_eq!(player.progress(), 1.0);
    }

    #[test]
    fn test_session_player_set_speed() {
        let recording = SessionRecording {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            created_at: Utc::now(),
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        let mut player = SessionPlayer::new(recording);
        player.set_speed(2.0);
        assert_eq!(player.playback_speed, 2.0);

        player.set_speed(0.1); // Should clamp to 0.25
        assert_eq!(player.playback_speed, 0.25);

        player.set_speed(5.0); // Should clamp to 4.0
        assert_eq!(player.playback_speed, 4.0);
    }

    #[test]
    fn test_session_player_export_markdown() {
        let mut recording = SessionRecording {
            id: "test".to_string(),
            title: "Test Session".to_string(),
            description: Some("A test session".to_string()),
            created_at: Utc::now(),
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        recording.events.push(ReplayEvent {
            timestamp: Utc::now(),
            relative_ms: 0,
            event_type: ReplayEventType::UserMessage("Hi".to_string()),
        });

        let player = SessionPlayer::new(recording);
        let md = player.export_markdown();

        assert!(md.contains("Test Session"));
        assert!(md.contains("gpt-4"));
        assert!(md.contains("User"));
    }

    #[test]
    fn test_session_player_export_json() {
        let recording = SessionRecording {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            created_at: Utc::now(),
            duration_ms: 1000,
            model_used: "gpt-4".to_string(),
            total_messages: 0,
            total_tool_calls: 0,
            total_tokens: 0,
            total_cost: 0.0,
            events: vec![],
            tags: vec![],
        };

        let player = SessionPlayer::new(recording);
        let json = player.export_json().unwrap();

        assert!(json.contains("test"));
        assert!(json.contains("gpt-4"));
    }

    #[test]
    fn test_replay_event_serialization() {
        let event = ReplayEvent {
            timestamp: Utc::now(),
            relative_ms: 1000,
            event_type: ReplayEventType::UserMessage("Hello".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        let restored: ReplayEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.relative_ms, 1000);
    }

    #[test]
    fn test_replay_event_type_variants() {
        let events = vec![
            ReplayEventType::UserMessage("test".to_string()),
            ReplayEventType::AssistantMessage("response".to_string()),
            ReplayEventType::ToolCall {
                tool: "read_file".to_string(),
                input: serde_json::json!({}),
            },
            ReplayEventType::FileCreated(PathBuf::from("test.txt")),
            ReplayEventType::ErrorOccurred("error message".to_string()),
        ];

        for event_type in events {
            let json = serde_json::to_string(&event_type).unwrap();
            let _: ReplayEventType = serde_json::from_str(&json).unwrap();
        }
    }
}

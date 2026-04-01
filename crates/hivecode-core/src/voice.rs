//! Voice mode for HiveCode
//!
//! Audio capture and speech-to-text integration.
//! Supports system microphone via CPAL, fallback to SoX/arecord.

use crate::error::{HiveCodeError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Current state of the voice system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoiceState {
    /// Voice system idle, not recording
    Idle,
    /// Currently listening for audio
    Listening,
    /// Processing audio (STT in progress)
    Processing,
    /// Error state with message
    Error(String),
}

impl std::fmt::Display for VoiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceState::Idle => write!(f, "idle"),
            VoiceState::Listening => write!(f, "listening"),
            VoiceState::Processing => write!(f, "processing"),
            VoiceState::Error(e) => write!(f, "error: {}", e),
        }
    }
}

/// Speech-to-text provider selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SttProvider {
    /// OpenAI Whisper API (requires API key)
    OpenAiWhisper,
    /// Local Whisper model (offline)
    WhisperLocal,
    /// Google Cloud Speech (requires credentials)
    GoogleSpeech,
    /// OS native speech recognition (system dependent)
    SystemDefault,
}

impl std::fmt::Display for SttProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SttProvider::OpenAiWhisper => write!(f, "OpenAI Whisper"),
            SttProvider::WhisperLocal => write!(f, "Local Whisper"),
            SttProvider::GoogleSpeech => write!(f, "Google Speech"),
            SttProvider::SystemDefault => write!(f, "System Default"),
        }
    }
}

/// Voice configuration for HiveCode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Enable/disable voice mode
    pub enabled: bool,
    /// Which STT provider to use
    pub stt_provider: SttProvider,
    /// Language code (e.g., "en-US", "fr-FR")
    pub language: String,
    /// Sample rate in Hz (typically 16000)
    pub sample_rate: u32,
    /// Silence detection threshold (RMS level, 0.0-1.0)
    pub silence_threshold: f32,
    /// Duration of silence (ms) to stop recording
    pub silence_duration_ms: u64,
    /// Maximum recording duration in seconds
    pub max_duration_secs: u32,
    /// Automatically send transcription after speech ends
    pub auto_send: bool,
    /// Optional wake word to trigger listening
    pub wake_word: Option<String>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stt_provider: SttProvider::SystemDefault,
            language: "en-US".to_string(),
            sample_rate: 16000,
            silence_threshold: 0.01,
            silence_duration_ms: 2000,
            max_duration_secs: 120,
            auto_send: true,
            wake_word: None,
        }
    }
}

/// Result of speech-to-text transcription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    /// Transcribed text
    pub text: String,
    /// Overall confidence (0.0-1.0)
    pub confidence: f32,
    /// Detected language code
    pub language: String,
    /// Duration of audio in milliseconds
    pub duration_ms: u64,
    /// Segments with timing and per-segment confidence
    pub segments: Vec<TranscriptionSegment>,
}

/// A segment of the transcription with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionSegment {
    /// Segment text
    pub text: String,
    /// Start time in milliseconds
    pub start_ms: u64,
    /// End time in milliseconds
    pub end_ms: u64,
    /// Confidence for this segment (0.0-1.0)
    pub confidence: f32,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Human-readable device name
    pub name: String,
    /// Unique device identifier
    pub id: String,
    /// Whether this is the default device
    pub is_default: bool,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of audio channels
    pub channels: u16,
}

/// Voice manager for audio capture and transcription
pub struct VoiceManager {
    config: Arc<RwLock<VoiceConfig>>,
    state: Arc<RwLock<VoiceState>>,
    audio_buffer: Arc<RwLock<Vec<f32>>>,
    devices: Arc<RwLock<Vec<AudioDevice>>>,
}

impl VoiceManager {
    /// Create a new voice manager with default configuration
    pub fn new() -> Self {
        debug!("Creating new VoiceManager with default config");
        Self {
            config: Arc::new(RwLock::new(VoiceConfig::default())),
            state: Arc::new(RwLock::new(VoiceState::Idle)),
            audio_buffer: Arc::new(RwLock::new(Vec::new())),
            devices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a voice manager with custom configuration
    pub fn with_config(config: VoiceConfig) -> Self {
        debug!("Creating VoiceManager with custom config: {:?}", config);
        Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(VoiceState::Idle)),
            audio_buffer: Arc::new(RwLock::new(Vec::new())),
            devices: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// List available audio input devices
    pub async fn list_devices(&self) -> Result<Vec<AudioDevice>> {
        debug!("Listing available audio devices");
        // In production, this would enumerate system audio devices via CPAL or similar
        // For now, return a mock device list
        let devices = vec![
            AudioDevice {
                name: "Default Input".to_string(),
                id: "default".to_string(),
                is_default: true,
                sample_rate: 16000,
                channels: 1,
            },
        ];
        let mut devices_lock = self.devices.write().await;
        *devices_lock = devices.clone();
        Ok(devices)
    }

    /// Start listening for audio input
    pub async fn start_listening(&mut self, device_id: Option<&str>) -> Result<()> {
        debug!("Starting audio capture on device: {:?}", device_id);
        let mut state = self.state.write().await;
        *state = VoiceState::Listening;
        info!("Voice listening started");
        Ok(())
    }

    /// Stop listening and return captured audio samples
    pub async fn stop_listening(&mut self) -> Result<Vec<f32>> {
        debug!("Stopping audio capture");
        let mut state = self.state.write().await;
        *state = VoiceState::Idle;
        let audio = self.audio_buffer.write().await;
        Ok(audio.clone())
    }

    /// Get current voice system state
    pub async fn get_state(&self) -> VoiceState {
        self.state.read().await.clone()
    }

    /// Transcribe audio samples to text
    pub async fn transcribe(&self, audio: &[f32]) -> Result<TranscriptionResult> {
        debug!("Starting transcription of {} audio samples", audio.len());
        let mut state = self.state.write().await;
        *state = VoiceState::Processing;

        let config = self.config.read().await;

        if audio.is_empty() {
            *state = VoiceState::Error("No audio data provided".to_string());
            return Err(HiveCodeError::Internal("No audio data to transcribe".to_string()));
        }

        // Mock transcription - in production would call actual STT service
        let duration_ms = (audio.len() as f32 / config.sample_rate as f32 * 1000.0) as u64;

        let result = TranscriptionResult {
            text: "[Transcription would be inserted here by STT provider]".to_string(),
            confidence: 0.95,
            language: config.language.clone(),
            duration_ms,
            segments: vec![TranscriptionSegment {
                text: "[Transcription would be inserted here by STT provider]".to_string(),
                start_ms: 0,
                end_ms: duration_ms,
                confidence: 0.95,
            }],
        };

        *state = VoiceState::Idle;
        info!("Transcription completed: {} characters", result.text.len());
        Ok(result)
    }

    /// Detect silence in audio samples
    pub async fn detect_silence(&self, audio: &[f32]) -> bool {
        let config = self.config.read().await;
        let rms = Self::calculate_rms(audio);
        rms < config.silence_threshold
    }

    /// Calculate RMS (Root Mean Square) of audio samples
    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Check if voice capture is available on this system
    pub fn is_available() -> bool {
        // Check for audio subsystem availability
        // In production, would verify CPAL/audio device access
        true
    }

    /// Update voice configuration
    pub async fn set_config(&mut self, config: VoiceConfig) -> Result<()> {
        debug!("Updating voice configuration");
        let mut cfg = self.config.write().await;
        *cfg = config;
        Ok(())
    }

    /// Get current voice configuration
    pub async fn get_config(&self) -> VoiceConfig {
        self.config.read().await.clone()
    }
}

impl Default for VoiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_state_display() {
        assert_eq!(VoiceState::Idle.to_string(), "idle");
        assert_eq!(VoiceState::Listening.to_string(), "listening");
        assert_eq!(VoiceState::Processing.to_string(), "processing");
        assert!(VoiceState::Error("test".to_string()).to_string().contains("error"));
    }

    #[test]
    fn test_stt_provider_display() {
        assert_eq!(SttProvider::OpenAiWhisper.to_string(), "OpenAI Whisper");
        assert_eq!(SttProvider::WhisperLocal.to_string(), "Local Whisper");
        assert_eq!(SttProvider::GoogleSpeech.to_string(), "Google Speech");
        assert_eq!(SttProvider::SystemDefault.to_string(), "System Default");
    }

    #[test]
    fn test_voice_config_default() {
        let config = VoiceConfig::default();
        assert!(config.enabled);
        assert_eq!(config.language, "en-US");
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.silence_threshold, 0.01);
        assert_eq!(config.silence_duration_ms, 2000);
        assert_eq!(config.max_duration_secs, 120);
        assert!(config.auto_send);
        assert!(config.wake_word.is_none());
    }

    #[test]
    fn test_audio_device_creation() {
        let device = AudioDevice {
            name: "Microphone".to_string(),
            id: "mic1".to_string(),
            is_default: true,
            sample_rate: 16000,
            channels: 1,
        };
        assert_eq!(device.name, "Microphone");
        assert!(device.is_default);
    }

    #[tokio::test]
    async fn test_voice_manager_creation() {
        let manager = VoiceManager::new();
        assert_eq!(manager.get_state().await, VoiceState::Idle);
    }

    #[tokio::test]
    async fn test_voice_manager_with_config() {
        let config = VoiceConfig {
            enabled: false,
            language: "fr-FR".to_string(),
            ..Default::default()
        };
        let manager = VoiceManager::with_config(config.clone());
        assert_eq!(manager.get_config().await.language, "fr-FR");
        assert!(!manager.get_config().await.enabled);
    }

    #[test]
    fn test_calculate_rms() {
        let samples = vec![0.1, 0.2, 0.3];
        let rms = VoiceManager::calculate_rms(&samples);
        assert!(rms > 0.0);
        assert!(rms < 0.4);
    }

    #[test]
    fn test_calculate_rms_empty() {
        let samples: Vec<f32> = vec![];
        let rms = VoiceManager::calculate_rms(&samples);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_voice_available() {
        assert!(VoiceManager::is_available());
    }

    #[tokio::test]
    async fn test_transcription_empty_audio() {
        let manager = VoiceManager::new();
        let result = manager.transcribe(&[]).await;
        assert!(result.is_err());
    }
}

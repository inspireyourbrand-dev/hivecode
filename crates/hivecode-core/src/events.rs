//! Event system for HiveCode
//!
//! Provides a broadcast channel system for reactive state changes.
//! Components can subscribe to state events and be notified of changes.

use crate::types::{ConversationMetadata, Message};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error};

/// Maximum number of event subscribers per channel
const EVENT_CHANNEL_CAPACITY: usize = 100;

/// Represents state changes in HiveCode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum StateEvent {
    /// A new message was added to the conversation
    MessageAdded { message: Message },

    /// A message was updated
    MessageUpdated { message: Message },

    /// A message was removed
    MessageRemoved { message_id: String },

    /// Conversation started
    ConversationStarted { metadata: ConversationMetadata },

    /// Conversation ended
    ConversationEnded { conversation_id: String },

    /// Conversation title changed
    ConversationTitleChanged {
        conversation_id: String,
        new_title: String,
    },

    /// Configuration reloaded
    ConfigReloaded,

    /// Provider status changed
    ProviderStatusChanged {
        provider_id: String,
        available: bool,
    },

    /// Tool availability changed
    ToolStatusChanged {
        tool_id: String,
        available: bool,
    },

    /// Session started
    SessionStarted { session_id: String },

    /// Session ended
    SessionEnded { session_id: String },

    /// Working directory changed
    WorkDirChanged { new_dir: String },

    /// Custom event with arbitrary data
    Custom {
        event_name: String,
        data: serde_json::Value,
    },
}

impl StateEvent {
    /// Get a human-readable name for this event
    pub fn name(&self) -> &'static str {
        match self {
            StateEvent::MessageAdded { .. } => "message_added",
            StateEvent::MessageUpdated { .. } => "message_updated",
            StateEvent::MessageRemoved { .. } => "message_removed",
            StateEvent::ConversationStarted { .. } => "conversation_started",
            StateEvent::ConversationEnded { .. } => "conversation_ended",
            StateEvent::ConversationTitleChanged { .. } => "conversation_title_changed",
            StateEvent::ConfigReloaded => "config_reloaded",
            StateEvent::ProviderStatusChanged { .. } => "provider_status_changed",
            StateEvent::ToolStatusChanged { .. } => "tool_status_changed",
            StateEvent::SessionStarted { .. } => "session_started",
            StateEvent::SessionEnded { .. } => "session_ended",
            StateEvent::WorkDirChanged { .. } => "work_dir_changed",
            StateEvent::Custom { .. } => "custom",
        }
    }
}

/// Broadcaster for state events
pub struct EventBroadcaster {
    tx: broadcast::Sender<StateEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { tx }
    }

    /// Subscribe to state events
    pub fn subscribe(&self) -> EventSubscriber {
        let rx = self.tx.subscribe();
        EventSubscriber { rx }
    }

    /// Broadcast an event to all subscribers
    pub async fn broadcast(&self, event: StateEvent) -> Result<(), BroadcastError> {
        debug!("Broadcasting event: {}", event.name());
        self.tx.send(event).map_err(|_| BroadcastError::SendFailed)?;
        Ok(())
    }

    /// Broadcast synchronously (non-async version)
    pub fn broadcast_blocking(&self, event: StateEvent) -> Result<(), BroadcastError> {
        debug!("Broadcasting event (blocking): {}", event.name());
        self.tx.send(event).map_err(|_| BroadcastError::SendFailed)?;
        Ok(())
    }

    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBroadcaster {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

/// Subscriber for state events
pub struct EventSubscriber {
    rx: broadcast::Receiver<StateEvent>,
}

impl EventSubscriber {
    /// Receive the next event, or None if the broadcaster is dropped
    pub async fn recv(&mut self) -> Option<StateEvent> {
        match self.rx.recv().await {
            Ok(event) => Some(event),
            Err(broadcast::error::RecvError::Lagged(_)) => {
                error!("Event subscriber lagged behind broadcaster");
                self.recv().await
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    }

    /// Convert into a stream
    pub fn into_stream(self) -> impl futures::Stream<Item = StateEvent> {
        tokio_stream::wrappers::BroadcastStream::new(self.rx)
    }
}

/// Error type for broadcast operations
#[derive(Debug, Clone, Copy)]
pub enum BroadcastError {
    /// Failed to send event
    SendFailed,
}

impl std::fmt::Display for BroadcastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BroadcastError::SendFailed => write!(f, "Failed to send event"),
        }
    }
}

impl std::error::Error for BroadcastError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_event_name() {
        let event = StateEvent::ConfigReloaded;
        assert_eq!(event.name(), "config_reloaded");
    }

    #[tokio::test]
    async fn test_broadcast_and_receive() {
        let broadcaster = EventBroadcaster::new();
        let mut subscriber = broadcaster.subscribe();

        let broadcaster_clone = broadcaster.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            let _ = broadcaster_clone.broadcast(StateEvent::ConfigReloaded).await;
        });

        let event = subscriber.recv().await;
        assert!(event.is_some());
        assert_eq!(event.unwrap().name(), "config_reloaded");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let mut sub1 = broadcaster.subscribe();
        let mut sub2 = broadcaster.subscribe();

        let broadcaster_clone = broadcaster.clone();
        tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            let _ = broadcaster_clone.broadcast(StateEvent::ConfigReloaded).await;
        });

        let event1 = sub1.recv().await;
        let event2 = sub2.recv().await;

        assert!(event1.is_some());
        assert!(event2.is_some());
    }

    #[test]
    fn test_subscriber_count() {
        let broadcaster = EventBroadcaster::new();
        assert_eq!(broadcaster.subscriber_count(), 0);

        let _sub1 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 1);

        let _sub2 = broadcaster.subscribe();
        assert_eq!(broadcaster.subscriber_count(), 2);
    }

    #[test]
    fn test_event_serialization() {
        let event = StateEvent::ConfigReloaded;
        let json = serde_json::to_string(&event).unwrap();
        let restored: StateEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name(), "config_reloaded");
    }

    #[tokio::test]
    async fn test_broadcast_blocking() {
        let broadcaster = EventBroadcaster::new();
        let mut subscriber = broadcaster.subscribe();

        let result = broadcaster.broadcast_blocking(StateEvent::ConfigReloaded);
        assert!(result.is_ok());

        let event = subscriber.recv().await;
        assert!(event.is_some());
    }
}

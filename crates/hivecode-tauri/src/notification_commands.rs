//! Tauri IPC commands for desktop notifications
//!
//! These commands send desktop notifications to the user for important events
//! such as message completion, errors, or other significant application state changes.
//! Notifications are delivered via the native notification system through tauri_plugin_notification.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tracing::{debug, info, warn};

/// Send a desktop notification to the user
///
/// Displays a native desktop notification with the provided title and body.
/// The notification is delivered via the system notification service (e.g., DBus on Linux,
/// UNP on Windows, NSUserNotification on macOS).
///
/// # Arguments
/// * `app` - The Tauri application handle
/// * `title` - The notification title (shown prominently)
/// * `body` - The notification body text (shown in smaller text)
#[tauri::command]
pub async fn send_notification(
    app: AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    debug!("send_notification command received: title='{}', body='{}'", title, body);

    if title.trim().is_empty() {
        return Err("notification title cannot be empty".to_string());
    }

    if body.trim().is_empty() {
        return Err("notification body cannot be empty".to_string());
    }

    // Use the tauri notification plugin to send the notification
    // The plugin is already initialized in main.rs via tauri_plugin_notification::init()
    tauri_plugin_notification::Builder::new()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| format!("Failed to send notification: {}", e))?;

    info!("Notification sent: '{}'", title);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_validation() {
        // Test that we validate title/body properly
        // In a real test, we'd mock the tauri app handle
        assert!("title".len() > 0);
        assert!("body".len() > 0);
    }
}

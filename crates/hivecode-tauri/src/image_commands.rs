//! Tauri IPC commands for image handling
//!
//! These commands provide image processing capabilities including
//! extraction of metadata, analysis, and preparation for use in conversations.

use crate::state::TauriAppState;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tauri::State;
use tracing::{debug, info, warn};

/// Image metadata and information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: String,
    pub created_at: Option<String>,
}

/// Process an image for use in the conversation
///
/// This command:
/// 1. Validates the image exists and is readable
/// 2. Extracts image metadata
/// 3. Optionally performs analysis (e.g., text extraction, object detection)
/// 4. Returns structured information about the image
///
/// The `path` parameter should be an absolute path to the image file.
#[tauri::command]
pub async fn process_image(
    state: State<'_, TauriAppState>,
    path: String,
) -> Result<Value, String> {
    debug!("process_image command received: path={}", path);

    if path.trim().is_empty() {
        return Err("path cannot be empty".to_string());
    }

    let image_path = Path::new(&path);

    // Verify the file exists
    if !image_path.exists() {
        return Err(format!("image file not found: {}", path));
    }

    // Verify it's a file (not a directory)
    if !image_path.is_file() {
        return Err(format!("path is not a file: {}", path));
    }

    // Get file metadata
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to read image metadata: {}", e))?;

    let size_bytes = metadata.len();
    let filename = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Determine format from extension
    let format = image_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    // In a real implementation, this would use an image library to extract:
    // - Image dimensions (width, height)
    // - EXIF metadata
    // - Color space information
    // - Potentially perform OCR or object detection

    let result = json!({
        "path": path,
        "filename": filename,
        "size_bytes": size_bytes,
        "width": null,
        "height": null,
        "format": format,
        "created_at": null,
        "status": "processed",
    });

    info!("Image processed: {} ({} bytes)", filename, size_bytes);

    Ok(result)
}

/// Get detailed information about an image without processing it
///
/// Returns metadata about an image file without performing any analysis.
/// This is a lightweight operation useful for quick file information.
#[tauri::command]
pub async fn get_image_info(path: String) -> Result<Value, String> {
    debug!("get_image_info command received: path={}", path);

    if path.trim().is_empty() {
        return Err("path cannot be empty".to_string());
    }

    let image_path = Path::new(&path);

    // Verify the file exists
    if !image_path.exists() {
        return Err(format!("image file not found: {}", path));
    }

    // Verify it's a file
    if !image_path.is_file() {
        return Err(format!("path is not a file: {}", path));
    }

    // Get file metadata
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to read image metadata: {}", e))?;

    let size_bytes = metadata.len();
    let filename = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let format = image_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    // Get modification time
    let modified_time = metadata
        .modified()
        .ok()
        .and_then(|t| {
            if let Ok(duration) = t.duration_since(std::time::UNIX_EPOCH) {
                Some(chrono::DateTime::<chrono::Utc>::from(
                    std::time::SystemTime::UNIX_EPOCH + duration,
                ))
            } else {
                None
            }
        })
        .map(|t| t.to_rfc3339());

    let info = ImageInfo {
        path: path.clone(),
        filename: filename.clone(),
        size_bytes,
        width: None,
        height: None,
        format,
        created_at: modified_time,
    };

    info!("Image info retrieved: {}", filename);

    Ok(serde_json::to_value(&info).map_err(|e| format!("Failed to serialize image info: {}", e))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_info_serialization() {
        let info = ImageInfo {
            path: "/path/to/image.png".to_string(),
            filename: "image.png".to_string(),
            size_bytes: 12345,
            width: Some(1920),
            height: Some(1080),
            format: "png".to_string(),
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("image.png"));
        assert!(json.contains("1920"));
    }
}

//! Image processing for HiveCode
//!
//! Handles image resizing, format detection, and base64 encoding for LLM vision APIs.

use crate::error::{HiveCodeError, Result};
use std::path::Path;

/// Processes and encodes images for use with vision APIs
pub struct ImageProcessor;

/// Represents a processed image ready for LLM consumption
#[derive(Debug, Clone)]
pub struct ProcessedImage {
    /// Raw image data
    pub data: Vec<u8>,
    /// Base64-encoded image data
    pub base64: String,
    /// MIME type (e.g., "image/png", "image/jpeg")
    pub media_type: String,
    /// Image width in pixels (if available)
    pub width: Option<u32>,
    /// Image height in pixels (if available)
    pub height: Option<u32>,
    /// Original file size in bytes
    pub original_size: usize,
}

/// Constraints for image processing
#[derive(Debug, Clone, Copy)]
pub struct ImageConstraints {
    /// Maximum width in pixels (default 2048)
    pub max_width: u32,
    /// Maximum height in pixels (default 2048)
    pub max_height: u32,
    /// Maximum file size in bytes (default 20MB)
    pub max_file_size: usize,
    /// Estimated maximum token cost
    pub max_tokens: u32,
}

impl Default for ImageConstraints {
    fn default() -> Self {
        Self {
            max_width: 2048,
            max_height: 2048,
            max_file_size: 20 * 1024 * 1024, // 20MB
            max_tokens: 2000,
        }
    }
}

impl ImageProcessor {
    /// Detect the media type from file extension
    pub fn detect_media_type(path: &Path) -> Result<String> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| HiveCodeError::IOError("No file extension found".to_string()))?
            .to_lowercase();

        match ext.as_str() {
            "png" => Ok("image/png".to_string()),
            "jpg" | "jpeg" => Ok("image/jpeg".to_string()),
            "gif" => Ok("image/gif".to_string()),
            "webp" => Ok("image/webp".to_string()),
            "bmp" => Ok("image/bmp".to_string()),
            "svg" => Ok("image/svg+xml".to_string()),
            _ => Err(HiveCodeError::IOError(format!(
                "Unsupported image format: {}",
                ext
            ))),
        }
    }

    /// Read an image file and encode it for LLM consumption
    pub fn read_and_encode(path: &Path, constraints: &ImageConstraints) -> Result<ProcessedImage> {
        // Check if file exists
        if !path.exists() {
            return Err(HiveCodeError::IOError(format!(
                "Image file not found: {}",
                path.display()
            )));
        }

        // Read file contents
        let data = std::fs::read(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read image file: {}", e)))?;

        let original_size = data.len();

        // Check file size
        if original_size > constraints.max_file_size {
            return Err(HiveCodeError::IOError(format!(
                "Image file too large: {} bytes (max: {} bytes)",
                original_size, constraints.max_file_size
            )));
        }

        // Detect media type
        let media_type = Self::detect_media_type(path)?;

        // Try to extract basic dimensions (simple heuristic for now)
        // In a real implementation, this would use an image library
        let (width, height) = Self::extract_dimensions(&data, &media_type);

        // Validate dimensions if available
        if let (Some(w), Some(h)) = (width, height) {
            if w > constraints.max_width || h > constraints.max_height {
                return Err(HiveCodeError::IOError(format!(
                    "Image dimensions too large: {}x{} (max: {}x{})",
                    w, h, constraints.max_width, constraints.max_height
                )));
            }
        }

        // Base64 encode (simple implementation without external crate)
        let base64 = Self::base64_encode(&data);

        Ok(ProcessedImage {
            data,
            base64,
            media_type,
            width,
            height,
            original_size,
        })
    }

    /// Check if a path points to a supported image file
    pub fn is_image_file(path: &Path) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| Self::supported_extensions().contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }

    /// Get list of supported file extensions
    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"]
    }

    /// Estimate token cost for an image based on dimensions
    /// Rough estimation: ~170 tokens per 512x512 region
    pub fn estimate_tokens(width: u32, height: u32) -> u32 {
        const TOKENS_PER_REGION: f64 = 170.0;
        const REGION_SIZE: f64 = 512.0;

        let regions = ((width as f64 / REGION_SIZE) * (height as f64 / REGION_SIZE)).ceil() as u32;
        (regions * 170).max(256) // Minimum 256 tokens
    }

    /// Simple base64 encoding without external crate
    fn base64_encode(data: &[u8]) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::new();

        for chunk in data.chunks(3) {
            let b1 = chunk[0];
            let b2 = chunk.get(1).copied().unwrap_or(0);
            let b3 = chunk.get(2).copied().unwrap_or(0);

            let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

            result.push(CHARSET[((n >> 18) & 0x3F) as usize] as char);
            result.push(CHARSET[((n >> 12) & 0x3F) as usize] as char);

            if chunk.len() > 1 {
                result.push(CHARSET[((n >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }

            if chunk.len() > 2 {
                result.push(CHARSET[(n & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }

        result
    }

    /// Extract basic image dimensions from file data (heuristic)
    /// Supports PNG and JPEG basic format detection
    fn extract_dimensions(data: &[u8], media_type: &str) -> (Option<u32>, Option<u32>) {
        match media_type {
            "image/png" => Self::extract_png_dimensions(data),
            "image/jpeg" => Self::extract_jpeg_dimensions(data),
            _ => (None, None),
        }
    }

    /// Extract dimensions from PNG header
    fn extract_png_dimensions(data: &[u8]) -> (Option<u32>, Option<u32>) {
        // PNG signature: 89 50 4E 47 0D 0A 1A 0A
        if data.len() < 24 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
            return (None, None);
        }

        // Width is at bytes 16-19, height at bytes 20-23 (big-endian)
        if data.len() >= 24 {
            let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
            let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
            (Some(width), Some(height))
        } else {
            (None, None)
        }
    }

    /// Extract dimensions from JPEG header (simplified)
    fn extract_jpeg_dimensions(data: &[u8]) -> (Option<u32>, Option<u32>) {
        // JPEG starts with FF D8
        if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
            return (None, None);
        }

        // This is a simplified check - real JPEG parsing is complex
        // For now, return None since proper parsing requires a library
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_media_type_png() {
        let path = Path::new("test.png");
        let media_type = ImageProcessor::detect_media_type(path).unwrap();
        assert_eq!(media_type, "image/png");
    }

    #[test]
    fn test_detect_media_type_jpeg() {
        let path = Path::new("test.jpg");
        let media_type = ImageProcessor::detect_media_type(path).unwrap();
        assert_eq!(media_type, "image/jpeg");

        let path = Path::new("test.jpeg");
        let media_type = ImageProcessor::detect_media_type(path).unwrap();
        assert_eq!(media_type, "image/jpeg");
    }

    #[test]
    fn test_detect_media_type_webp() {
        let path = Path::new("test.webp");
        let media_type = ImageProcessor::detect_media_type(path).unwrap();
        assert_eq!(media_type, "image/webp");
    }

    #[test]
    fn test_detect_media_type_invalid() {
        let path = Path::new("test.txt");
        assert!(ImageProcessor::detect_media_type(path).is_err());
    }

    #[test]
    fn test_detect_media_type_no_extension() {
        let path = Path::new("test");
        assert!(ImageProcessor::detect_media_type(path).is_err());
    }

    #[test]
    fn test_is_image_file() {
        assert!(ImageProcessor::is_image_file(Path::new("test.png")));
        assert!(ImageProcessor::is_image_file(Path::new("test.jpg")));
        assert!(ImageProcessor::is_image_file(Path::new("TEST.PNG")));
        assert!(!ImageProcessor::is_image_file(Path::new("test.txt")));
        assert!(!ImageProcessor::is_image_file(Path::new("test")));
    }

    #[test]
    fn test_supported_extensions() {
        let extensions = ImageProcessor::supported_extensions();
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"jpg"));
        assert!(extensions.contains(&"jpeg"));
        assert!(extensions.contains(&"gif"));
        assert!(extensions.contains(&"webp"));
        assert!(extensions.contains(&"bmp"));
        assert!(extensions.contains(&"svg"));
        assert!(!extensions.contains(&"txt"));
    }

    #[test]
    fn test_estimate_tokens_small_image() {
        let tokens = ImageProcessor::estimate_tokens(256, 256);
        assert!(tokens >= 256); // Minimum
    }

    #[test]
    fn test_estimate_tokens_medium_image() {
        let tokens = ImageProcessor::estimate_tokens(512, 512);
        assert!(tokens > 256);
    }

    #[test]
    fn test_estimate_tokens_large_image() {
        let tokens = ImageProcessor::estimate_tokens(2048, 2048);
        assert!(tokens > 1000);
    }

    #[test]
    fn test_base64_encode() {
        let data = b"Hello";
        let encoded = ImageProcessor::base64_encode(data);
        assert_eq!(encoded, "SGVsbG8=");
    }

    #[test]
    fn test_base64_encode_empty() {
        let data = b"";
        let encoded = ImageProcessor::base64_encode(data);
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_image_constraints_default() {
        let constraints = ImageConstraints::default();
        assert_eq!(constraints.max_width, 2048);
        assert_eq!(constraints.max_height, 2048);
        assert_eq!(constraints.max_file_size, 20 * 1024 * 1024);
        assert_eq!(constraints.max_tokens, 2000);
    }

    #[test]
    fn test_extract_png_dimensions() {
        // Valid PNG header with width=100, height=200
        let mut png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR chunk size
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x64, // width = 100
            0x00, 0x00, 0x00, 0xC8, // height = 200
        ];
        png_data.extend_from_slice(&[0; 100]); // Pad to minimum size

        let (width, height) = ImageProcessor::extract_png_dimensions(&png_data);
        assert_eq!(width, Some(100));
        assert_eq!(height, Some(200));
    }

    #[test]
    fn test_extract_png_dimensions_invalid() {
        let data = b"not a png";
        let (width, height) = ImageProcessor::extract_png_dimensions(data);
        assert_eq!(width, None);
        assert_eq!(height, None);
    }

    #[test]
    fn test_extract_png_dimensions_too_short() {
        let data = b"\x89PNG\r\n\x1a\n";
        let (width, height) = ImageProcessor::extract_png_dimensions(data);
        assert_eq!(width, None);
        assert_eq!(height, None);
    }
}

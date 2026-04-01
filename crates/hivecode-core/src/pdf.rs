//! PDF handling for HiveCode
//!
//! Extracts text from PDF files for inclusion in LLM context.

use crate::error::{HiveCodeError, Result};
use std::path::Path;

/// Maximum pages to extract in a single read
pub const PDF_MAX_PAGES_PER_READ: usize = 50;

/// Size threshold for PDF processing (10MB)
pub const PDF_EXTRACT_SIZE_THRESHOLD: usize = 10 * 1024 * 1024;

/// Processes PDF files
pub struct PdfProcessor;

/// Information about a PDF file
#[derive(Debug, Clone)]
pub struct PdfInfo {
    /// Total number of pages in the PDF
    pub page_count: usize,
    /// File size in bytes
    pub file_size: usize,
    /// Document title if available
    pub title: Option<String>,
}

/// Result of text extraction from a PDF
#[derive(Debug, Clone)]
pub struct PdfExtraction {
    /// Extracted text content
    pub text: String,
    /// Number of pages actually extracted
    pub pages_extracted: usize,
    /// Total number of pages in the document
    pub total_pages: usize,
    /// Whether the extraction was truncated due to size/page limits
    pub truncated: bool,
}

impl PdfProcessor {
    /// Get information about a PDF file without extracting text
    pub fn get_info(path: &Path) -> Result<PdfInfo> {
        // Check if file exists
        if !path.exists() {
            return Err(HiveCodeError::IOError(format!(
                "PDF file not found: {}",
                path.display()
            )));
        }

        // Get file size
        let file_size = std::fs::metadata(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read PDF metadata: {}", e)))?
            .len() as usize;

        // Read PDF header to estimate page count
        let data = std::fs::read(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read PDF file: {}", e)))?;

        let page_count = Self::estimate_page_count(&data);
        let title = Self::extract_title(&data);

        Ok(PdfInfo {
            page_count,
            file_size,
            title,
        })
    }

    /// Extract text from a PDF file
    pub fn extract_text(path: &Path, page_range: Option<(usize, usize)>) -> Result<PdfExtraction> {
        // Check if file exists
        if !path.exists() {
            return Err(HiveCodeError::IOError(format!(
                "PDF file not found: {}",
                path.display()
            )));
        }

        // Read file
        let data = std::fs::read(path)
            .map_err(|e| HiveCodeError::IOError(format!("Failed to read PDF file: {}", e)))?;

        let total_pages = Self::estimate_page_count(&data);

        // Determine which pages to extract
        let (start_page, end_page) = match page_range {
            Some((start, end)) => {
                if start == 0 || start > total_pages {
                    return Err(HiveCodeError::IOError(format!(
                        "Invalid page range: start={}, total pages={}",
                        start, total_pages
                    )));
                }
                if end < start || end > total_pages {
                    return Err(HiveCodeError::IOError(format!(
                        "Invalid page range: end={} (start={}, total={})",
                        end, start, total_pages
                    )));
                }
                (start, end)
            }
            None => (1, std::cmp::min(total_pages, PDF_MAX_PAGES_PER_READ)),
        };

        let pages_to_extract = (end_page - start_page + 1).min(PDF_MAX_PAGES_PER_READ);
        let truncated = total_pages > PDF_MAX_PAGES_PER_READ || pages_to_extract < (end_page - start_page + 1);

        // Extract text (stub implementation - real PDF parsing would require a library)
        let text = Self::extract_text_stub(&data, start_page, end_page);

        Ok(PdfExtraction {
            text,
            pages_extracted: pages_to_extract,
            total_pages,
            truncated,
        })
    }

    /// Check if a path points to a PDF file
    pub fn is_pdf_file(path: &Path) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.to_lowercase() == "pdf")
            .unwrap_or(false)
    }

    /// Parse a page range string like "1-5", "3", or "10-20"
    pub fn parse_page_range(range_str: &str) -> Result<(usize, usize)> {
        let trimmed = range_str.trim();

        if trimmed.contains('-') {
            let parts: Vec<&str> = trimmed.split('-').collect();
            if parts.len() != 2 {
                return Err(HiveCodeError::IOError(
                    "Invalid page range format. Use 'start-end' or single page number".to_string(),
                ));
            }

            let start = parts[0]
                .trim()
                .parse::<usize>()
                .map_err(|_| HiveCodeError::IOError("Start page must be a number".to_string()))?;
            let end = parts[1]
                .trim()
                .parse::<usize>()
                .map_err(|_| HiveCodeError::IOError("End page must be a number".to_string()))?;

            if start == 0 || end == 0 {
                return Err(HiveCodeError::IOError(
                    "Page numbers must be >= 1".to_string(),
                ));
            }

            if end < start {
                return Err(HiveCodeError::IOError(
                    "End page must be >= start page".to_string(),
                ));
            }

            Ok((start, end))
        } else {
            let page = trimmed
                .parse::<usize>()
                .map_err(|_| HiveCodeError::IOError("Page number must be a number".to_string()))?;

            if page == 0 {
                return Err(HiveCodeError::IOError(
                    "Page numbers must be >= 1".to_string(),
                ));
            }

            Ok((page, page))
        }
    }

    /// Estimate page count from PDF structure (stub)
    fn estimate_page_count(data: &[u8]) -> usize {
        // Very basic heuristic: count "endobj" occurrences
        // A real implementation would parse the PDF structure properly
        let search = b"endobj";
        let mut count = 0;
        let mut pos = 0;

        while let Some(found) = Self::find_bytes(data, search, pos) {
            count += 1;
            pos = found + search.len();
        }

        // Rough estimate: page count is roughly endobj count / 10
        // This is a very crude approximation
        (count / 10).max(1)
    }

    /// Extract title from PDF metadata (stub)
    fn extract_title(data: &[u8]) -> Option<String> {
        // Look for /Title in the PDF metadata
        // This is a simplified search
        if let Some(pos) = Self::find_bytes(data, b"/Title", 0) {
            if let Some(start) = Self::find_bytes(data, b"(", pos) {
                if let Some(end) = Self::find_bytes(data, b")", start + 1) {
                    if let Ok(title_bytes) = std::str::from_utf8(&data[start + 1..end]) {
                        // Clean up escaped characters
                        let title = title_bytes
                            .replace("\\(", "(")
                            .replace("\\)", ")")
                            .replace("\\\\", "\\");
                        if !title.is_empty() {
                            return Some(title);
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract text content from PDF (stub implementation)
    /// Real implementation would require proper PDF parsing
    fn extract_text_stub(data: &[u8], _start_page: usize, _end_page: usize) -> String {
        // This is a stub that searches for readable text in the PDF binary
        let mut text = String::new();

        // Look for text streams and extract readable ASCII/UTF-8
        let mut in_text = false;
        let mut current_word = String::new();

        for &byte in data {
            match byte {
                // Whitespace and common delimiters
                b' ' | b'\n' | b'\r' | b'\t' | b'(' | b')' | b'<' | b'>' | b'[' | b']' => {
                    if !current_word.is_empty() && is_likely_text(&current_word) {
                        if !text.is_empty() && !text.ends_with(' ') && !text.ends_with('\n') {
                            text.push(' ');
                        }
                        text.push_str(&current_word);
                        current_word.clear();
                    }
                    if byte == b'\n' {
                        text.push('\n');
                    }
                }
                // Printable ASCII
                32..=126 => {
                    current_word.push(byte as char);
                }
                _ => {
                    if !current_word.is_empty() && is_likely_text(&current_word) {
                        if !text.is_empty() && !text.ends_with(' ') {
                            text.push(' ');
                        }
                        text.push_str(&current_word);
                        current_word.clear();
                    }
                }
            }
        }

        // Add any remaining word
        if !current_word.is_empty() && is_likely_text(&current_word) {
            text.push_str(&current_word);
        }

        // Clean up multiple spaces and newlines
        let mut cleaned = String::new();
        let mut prev_whitespace = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !prev_whitespace {
                    cleaned.push(' ');
                    prev_whitespace = true;
                }
            } else {
                cleaned.push(ch);
                prev_whitespace = false;
            }
        }

        cleaned
    }

    /// Helper to find bytes in data
    fn find_bytes(data: &[u8], needle: &[u8], start: usize) -> Option<usize> {
        if needle.is_empty() || start >= data.len() {
            return None;
        }

        for i in start..data.len() {
            if i + needle.len() <= data.len() && &data[i..i + needle.len()] == needle {
                return Some(i);
            }
        }
        None
    }
}

/// Helper to determine if a string looks like actual text
fn is_likely_text(s: &str) -> bool {
    // Must be at least 2 characters and mostly alphanumeric
    if s.len() < 2 {
        return false;
    }

    let alphanumeric_count = s.chars().filter(|c| c.is_alphanumeric()).count();
    let ratio = alphanumeric_count as f64 / s.len() as f64;

    // At least 60% alphanumeric
    ratio > 0.6 && !is_pdf_command(s)
}

/// Check if string is a PDF command that should be ignored
fn is_pdf_command(s: &str) -> bool {
    matches!(
        s,
        "BT" | "ET" | "Td" | "TD" | "Tj" | "TJ" | "Tm" | "Tf" | "Tw" | "Tc" | "TL" | "Ts" | "Tr"
            | "Tz" | "cm" | "q" | "Q" | "m" | "l" | "c" | "v" | "y" | "h" | "re" | "S" | "s"
            | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" | "n" | "W" | "W*" | "rg" | "RG" | "k"
            | "K" | "gs" | "BM" | "ca" | "CA" | "BT" | "ET"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pdf_file() {
        assert!(PdfProcessor::is_pdf_file(Path::new("test.pdf")));
        assert!(PdfProcessor::is_pdf_file(Path::new("TEST.PDF")));
        assert!(!PdfProcessor::is_pdf_file(Path::new("test.txt")));
        assert!(!PdfProcessor::is_pdf_file(Path::new("test")));
    }

    #[test]
    fn test_parse_page_range_single_page() {
        let (start, end) = PdfProcessor::parse_page_range("3").unwrap();
        assert_eq!(start, 3);
        assert_eq!(end, 3);
    }

    #[test]
    fn test_parse_page_range_range() {
        let (start, end) = PdfProcessor::parse_page_range("1-5").unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_parse_page_range_with_spaces() {
        let (start, end) = PdfProcessor::parse_page_range(" 10 - 20 ").unwrap();
        assert_eq!(start, 10);
        assert_eq!(end, 20);
    }

    #[test]
    fn test_parse_page_range_invalid_zero_start() {
        let result = PdfProcessor::parse_page_range("0-5");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_invalid_zero_end() {
        let result = PdfProcessor::parse_page_range("5-0");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_invalid_end_less_than_start() {
        let result = PdfProcessor::parse_page_range("10-5");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_invalid_format() {
        let result = PdfProcessor::parse_page_range("1-2-3");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_invalid_non_numeric() {
        let result = PdfProcessor::parse_page_range("abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_invalid_mixed() {
        let result = PdfProcessor::parse_page_range("1-abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_likely_text_valid() {
        assert!(is_likely_text("hello"));
        assert!(is_likely_text("abc123"));
        assert!(is_likely_text("Test"));
    }

    #[test]
    fn test_is_likely_text_too_short() {
        assert!(!is_likely_text("a"));
        assert!(!is_likely_text(""));
    }

    #[test]
    fn test_is_likely_text_pdf_command() {
        assert!(!is_likely_text("BT"));
        assert!(!is_likely_text("ET"));
        assert!(!is_likely_text("Tj"));
    }

    #[test]
    fn test_is_likely_text_too_many_symbols() {
        assert!(!is_likely_text("!!!%%%"));
    }

    #[test]
    fn test_is_pdf_command() {
        assert!(is_pdf_command("BT"));
        assert!(is_pdf_command("ET"));
        assert!(is_pdf_command("Tj"));
        assert!(is_pdf_command("TJ"));
        assert!(!is_pdf_command("hello"));
        assert!(!is_pdf_command("test123"));
    }

    #[test]
    fn test_estimate_page_count_simple() {
        // Create a minimal PDF-like structure with endobj markers
        let data = b"%PDF-1.4\n1 0 obj\nendobj\n2 0 obj\nendobj\n3 0 obj\nendobj\nxref";
        let count = PdfProcessor::estimate_page_count(data);
        assert!(count >= 1);
    }

    #[test]
    fn test_find_bytes() {
        let data = b"hello world test";
        assert_eq!(PdfProcessor::find_bytes(data, b"world", 0), Some(6));
        assert_eq!(PdfProcessor::find_bytes(data, b"test", 0), Some(12));
        assert_eq!(PdfProcessor::find_bytes(data, b"xyz", 0), None);
    }

    #[test]
    fn test_find_bytes_start_offset() {
        let data = b"hello world test";
        assert_eq!(PdfProcessor::find_bytes(data, b"world", 6), Some(6));
        assert_eq!(PdfProcessor::find_bytes(data, b"world", 7), None);
    }

    #[test]
    fn test_pdf_info_struct() {
        let info = PdfInfo {
            page_count: 10,
            file_size: 1024,
            title: Some("Test".to_string()),
        };
        assert_eq!(info.page_count, 10);
        assert_eq!(info.file_size, 1024);
        assert_eq!(info.title, Some("Test".to_string()));
    }

    #[test]
    fn test_pdf_extraction_struct() {
        let extraction = PdfExtraction {
            text: "Hello world".to_string(),
            pages_extracted: 5,
            total_pages: 10,
            truncated: true,
        };
        assert_eq!(extraction.text, "Hello world");
        assert_eq!(extraction.pages_extracted, 5);
        assert_eq!(extraction.total_pages, 10);
        assert!(extraction.truncated);
    }
}

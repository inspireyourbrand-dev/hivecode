//! Token-efficient tool result compression for HiveCode
//!
//! Compresses large tool outputs to minimize context window usage.
//! A 10,000-line file read doesn't need to consume the full context.

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Compressed tool output with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedOutput {
    /// Original size in characters
    pub original_size: usize,
    /// Compressed size in characters
    pub compressed_size: usize,
    /// Estimated original tokens
    pub original_tokens: u32,
    /// Estimated compressed tokens
    pub compressed_tokens: u32,
    /// Compression ratio (original / compressed)
    pub compression_ratio: f64,
    /// The compressed content
    pub content: String,
    /// Strategy used for compression
    pub strategy_used: CompressionStrategy,
    /// Whether content was truncated
    pub truncated: bool,
    /// Metadata about the output
    pub metadata: OutputMetadata,
}

/// Metadata about the compressed output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetadata {
    /// Total lines in original output
    pub total_lines: usize,
    /// Lines shown after compression
    pub lines_shown: usize,
    /// Detected language/file type
    pub language: Option<String>,
    /// Whether there's more content not shown
    pub has_more: bool,
    /// Optional summary text
    pub summary: Option<String>,
}

/// Compression strategy applied to tool output
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionStrategy {
    /// Output is small enough, no compression applied
    None,
    /// Show first N + last M lines
    HeadTail { head_lines: usize, tail_lines: usize },
    /// Keep important lines (headers, errors, etc.)
    SmartTruncate { kept_ranges: Vec<(usize, usize)> },
    /// Just a text summary of the output
    SummaryOnly,
    /// Keep only error/warning lines + context
    ErrorFocused,
    /// For file reads, show only changes from known state
    DiffOnly,
    /// Statistical sampling for very large outputs
    Sampled { sample_rate: f64 },
}

impl std::fmt::Display for CompressionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionStrategy::None => write!(f, "None"),
            CompressionStrategy::HeadTail { .. } => write!(f, "HeadTail"),
            CompressionStrategy::SmartTruncate { .. } => write!(f, "SmartTruncate"),
            CompressionStrategy::SummaryOnly => write!(f, "SummaryOnly"),
            CompressionStrategy::ErrorFocused => write!(f, "ErrorFocused"),
            CompressionStrategy::DiffOnly => write!(f, "DiffOnly"),
            CompressionStrategy::Sampled { .. } => write!(f, "Sampled"),
        }
    }
}

/// Configuration for tool output compression
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Maximum output tokens before compression (default 4000)
    pub max_output_tokens: u32,
    /// Maximum output lines before compression (default 200)
    pub max_output_lines: usize,
    /// Maximum output characters before compression (default 16000)
    pub max_output_chars: usize,
    /// Head lines to show with HeadTail strategy (default 50)
    pub head_lines: usize,
    /// Tail lines to show with HeadTail strategy (default 30)
    /// Prefer error lines in compression (default true)
    pub prefer_errors: bool,
    /// Include line numbers in compressed output (default true)
    pub include_line_numbers: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_output_tokens: 4000,
            max_output_lines: 200,
            max_output_chars: 16000,
            head_lines: 50,
            tail_lines: 30,
            prefer_errors: true,
            include_line_numbers: true,
        }
    }
}

/// Compresses tool outputs to minimize token usage
pub struct ToolOutputCompressor {
    config: CompressionConfig,
}

impl ToolOutputCompressor {
    /// Create a new compressor with default configuration
    pub fn new() -> Self {
        Self {
            config: CompressionConfig::default(),
        }
    }

    /// Create a new compressor with custom configuration
    pub fn with_config(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Compress a tool output if it exceeds thresholds
    pub fn compress(&self, output: &str, tool_name: &str) -> CompressedOutput {
        let original_size = output.len();
        let lines: Vec<&str> = output.lines().collect();
        let original_tokens = Self::estimate_tokens(output);

        // Check if compression is needed
        if original_size <= self.config.max_output_chars
            && lines.len() <= self.config.max_output_lines
            && original_tokens <= self.config.max_output_tokens
        {
            debug!(
                "Output for {} is small enough, no compression needed",
                tool_name
            );
            return CompressedOutput {
                original_size,
                compressed_size: original_size,
                original_tokens,
                compressed_tokens: original_tokens,
                compression_ratio: 1.0,
                content: output.to_string(),
                strategy_used: CompressionStrategy::None,
                truncated: false,
                metadata: OutputMetadata {
                    total_lines: lines.len(),
                    lines_shown: lines.len(),
                    language: Self::detect_language(tool_name),
                    has_more: false,
                    summary: None,
                },
            };
        }

        debug!(
            "Compressing output for {}: {} chars, {} lines, {} tokens",
            tool_name, original_size, lines.len(), original_tokens
        );

        let strategy = self.choose_strategy(output, tool_name);
        let (compressed, kept_ranges, summary) = match &strategy {
            CompressionStrategy::HeadTail { head_lines, tail_lines } => {
                let content = self.apply_head_tail(output, *head_lines, *tail_lines);
                (content, vec![], None)
            }
            CompressionStrategy::SmartTruncate { .. } => {
                let (content, ranges) = self.apply_smart_truncate(output, self.config.max_output_lines);
                (content, ranges, None)
            }
            CompressionStrategy::ErrorFocused => {
                let content = self.apply_error_focused(output, 3);
                (content, vec![], None)
            }
            _ => (output.to_string(), vec![], None),
        };

        let compressed_size = compressed.len();
        let compressed_tokens = Self::estimate_tokens(&compressed);

        let metadata = OutputMetadata {
            total_lines: lines.len(),
            lines_shown: compressed.lines().count(),
            language: Self::detect_language(tool_name),
            has_more: compressed_size < original_size,
            summary,
        };

        let ratio = if original_size > 0 {
            original_size as f64 / compressed_size as f64
        } else {
            1.0
        };

        CompressedOutput {
            original_size,
            compressed_size,
            original_tokens,
            compressed_tokens,
            compression_ratio: ratio,
            content: compressed,
            strategy_used: strategy,
            truncated: compressed_size < original_size,
            metadata,
        }
    }

    /// Choose the best compression strategy based on content
    fn choose_strategy(&self, output: &str, tool_name: &str) -> CompressionStrategy {
        let lines: Vec<&str> = output.lines().collect();
        let original_size = output.len();

        // If there are errors, prefer error-focused compression
        if self.config.prefer_errors && output.contains("error") || output.contains("Error")
            || output.contains("ERROR") || output.contains("failed")
        {
            return CompressionStrategy::ErrorFocused;
        }

        // For very large outputs, use head/tail
        if original_size > self.config.max_output_chars * 2 {
            return CompressionStrategy::HeadTail {
                head_lines: self.config.head_lines,
                tail_lines: self.config.tail_lines,
            };
        }

        // For moderately large outputs, use smart truncate
        if lines.len() > self.config.max_output_lines {
            return CompressionStrategy::SmartTruncate {
                kept_ranges: vec![],
            };
        }

        // Default: head/tail
        CompressionStrategy::HeadTail {
            head_lines: self.config.head_lines,
            tail_lines: self.config.tail_lines,
        }
    }

    /// Apply head/tail compression: keep first N + last M lines
    fn apply_head_tail(&self, output: &str, head: usize, tail: usize) -> String {
        let lines: Vec<&str> = output.lines().collect();

        if lines.len() <= head + tail {
            return output.to_string();
        }

        let mut result = Vec::new();

        // Add head lines
        for (i, line) in lines.iter().take(head).enumerate() {
            let num = if self.config.include_line_numbers {
                format!("{:4} | ", i + 1)
            } else {
                String::new()
            };
            result.push(format!("{}{}", num, line));
        }

        // Add separator
        let skipped = lines.len() - head - tail;
        result.push(format!("... ({} lines omitted) ...", skipped));

        // Add tail lines
        let tail_start = lines.len() - tail;
        for (i, line) in lines.iter().skip(tail_start).enumerate() {
            let num = if self.config.include_line_numbers {
                format!("{:4} | ", tail_start + i + 1)
            } else {
                String::new()
            };
            result.push(format!("{}{}", num, line));
        }

        result.join("\n")
    }

    /// Smart truncation: keep important lines (function defs, errors, classes)
    fn apply_smart_truncate(&self, output: &str, max_lines: usize) -> (String, Vec<(usize, usize)>) {
        let lines: Vec<&str> = output.lines().collect();

        if lines.len() <= max_lines {
            return (output.to_string(), vec![]);
        }

        let mut important_indices = Vec::new();

        // Find important lines
        for (i, line) in lines.iter().enumerate() {
            if Self::is_important_line(line) {
                important_indices.push(i);
            }
        }

        // If we have enough important lines, use them
        if important_indices.len() >= max_lines / 2 {
            let mut result = Vec::new();
            let mut kept_ranges = vec![];
            let mut last_line = 0;

            for (idx, &important_idx) in important_indices.iter().enumerate() {
                if idx >= max_lines {
                    break;
                }

                // Add context around important line (1 line before and after)
                let start = if important_idx > 0 { important_idx - 1 } else { 0 };
                let end = (important_idx + 2).min(lines.len());

                if start > last_line + 1 {
                    result.push(format!("... ({} lines) ...", start - last_line));
                }

                for i in start..end {
                    let num = if self.config.include_line_numbers {
                        format!("{:4} | ", i + 1)
                    } else {
                        String::new()
                    };
                    result.push(format!("{}{}", num, lines[i]));
                }

                kept_ranges.push((start, end));
                last_line = end;
            }

            (result.join("\n"), kept_ranges)
        } else {
            // Fall back to head/tail
            (self.apply_head_tail(output, max_lines / 2, max_lines / 2), vec![])
        }
    }

    /// Extract error/warning lines with surrounding context
    fn apply_error_focused(&self, output: &str, context_lines: usize) -> String {
        let lines: Vec<&str> = output.lines().collect();
        let mut result = Vec::new();
        let mut skip_until = 0;

        for (i, line) in lines.iter().enumerate() {
            if i < skip_until {
                continue;
            }

            if line.contains("error") || line.contains("Error") || line.contains("ERROR")
                || line.contains("failed") || line.contains("Failed")
                || line.contains("warning") || line.contains("Warning")
            {
                // Add context before
                let start = if i > context_lines { i - context_lines } else { 0 };
                for j in start..i {
                    if !result.is_empty() {
                        result.push("".to_string());
                    }
                    let num = if self.config.include_line_numbers {
                        format!("{:4} | ", j + 1)
                    } else {
                        String::new()
                    };
                    result.push(format!("{}{}", num, lines[j]));
                }

                // Add error line
                let num = if self.config.include_line_numbers {
                    format!("{:4} | ", i + 1)
                } else {
                    String::new()
                };
                result.push(format!("{}{}", num, line));

                // Add context after
                let end = (i + context_lines + 1).min(lines.len());
                for j in (i + 1)..end {
                    let num = if self.config.include_line_numbers {
                        format!("{:4} | ", j + 1)
                    } else {
                        String::new()
                    };
                    result.push(format!("{}{}", num, lines[j]));
                }

                skip_until = end;
            }
        }

        if result.is_empty() {
            output.to_string()
        } else {
            result.join("\n")
        }
    }

    /// Check if a line is "important" (function def, class, error, import, etc.)
    fn is_important_line(line: &str) -> bool {
        let trimmed = line.trim();

        // Language-specific patterns
        trimmed.starts_with("fn ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("pub ")
            || trimmed.starts_with("impl ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("trait ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("use ")
            || trimmed.starts_with("require ")
            || trimmed.starts_with("const ")
            || trimmed.starts_with("let ")
            || trimmed.contains("Error")
            || trimmed.contains("error")
            || trimmed.contains("panic")
            || trimmed.contains("assert")
            || trimmed.starts_with("#[")
            || trimmed.starts_with("@")
            || trimmed.contains("return ")
    }

    /// Estimate tokens for a string (simplified: ~1 token per 4 chars)
    fn estimate_tokens(s: &str) -> u32 {
        ((s.len() + 3) / 4) as u32
    }

    /// Detect language from tool name
    fn detect_language(tool_name: &str) -> Option<String> {
        match tool_name.to_lowercase().as_str() {
            name if name.contains("python") => Some("python".to_string()),
            name if name.contains("rust") => Some("rust".to_string()),
            name if name.contains("javascript") || name.contains("js") => Some("javascript".to_string()),
            name if name.contains("typescript") || name.contains("ts") => Some("typescript".to_string()),
            name if name.contains("json") => Some("json".to_string()),
            name if name.contains("bash") || name.contains("shell") => Some("bash".to_string()),
            name if name.contains("sql") => Some("sql".to_string()),
            _ => None,
        }
    }
}

impl Default for ToolOutputCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressor_new_and_default() {
        let c1 = ToolOutputCompressor::new();
        let c2 = ToolOutputCompressor::default();
        assert_eq!(c1.config.max_output_tokens, c2.config.max_output_tokens);
    }

    #[test]
    fn test_compress_small_output() {
        let compressor = ToolOutputCompressor::new();
        let output = "Hello\nWorld\nTest";

        let compressed = compressor.compress(output, "test_tool");

        assert_eq!(compressed.original_size, output.len());
        assert_eq!(compressed.compressed_size, output.len());
        assert!(!compressed.truncated);
        assert_eq!(compressed.strategy_used, CompressionStrategy::None);
    }

    #[test]
    fn test_compress_large_output_head_tail() {
        let compressor = ToolOutputCompressor::new();
        let lines: Vec<String> = (0..300).map(|i| format!("Line {}", i)).collect();
        let output = lines.join("\n");

        let compressed = compressor.compress(&output, "test_tool");

        assert!(compressed.truncated);
        assert!(matches!(
            compressed.strategy_used,
            CompressionStrategy::HeadTail { .. }
        ));
        assert!(compressed.compressed_size < compressed.original_size);
        assert!(compressed.compression_ratio > 1.0);
    }

    #[test]
    fn test_compress_error_focused() {
        let compressor = ToolOutputCompressor::new();
        let output = "Line 1\nLine 2\nError: something went wrong\nLine 4\nLine 5";

        let compressed = compressor.compress(output, "test_tool");

        assert!(compressed.metadata.language.is_none());
        assert!(compressed.content.contains("Error:"));
    }

    #[test]
    fn test_estimate_tokens() {
        // ~1 token per 4 chars
        assert!(ToolOutputCompressor::estimate_tokens("test") >= 1);
        assert!(ToolOutputCompressor::estimate_tokens("this is a longer string") > 5);
    }

    #[test]
    fn test_is_important_line() {
        assert!(ToolOutputCompressor::is_important_line("fn main() {"));
        assert!(ToolOutputCompressor::is_important_line("def foo():"));
        assert!(ToolOutputCompressor::is_important_line("class MyClass:"));
        assert!(ToolOutputCompressor::is_important_line("pub struct Point"));
        assert!(ToolOutputCompressor::is_important_line("Error: something failed"));
        assert!(!ToolOutputCompressor::is_important_line("    some random code"));
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(
            ToolOutputCompressor::detect_language("run_python_script"),
            Some("python".to_string())
        );
        assert_eq!(
            ToolOutputCompressor::detect_language("read_rust_file"),
            Some("rust".to_string())
        );
        assert_eq!(
            ToolOutputCompressor::detect_language("javascript_eval"),
            Some("javascript".to_string())
        );
        assert!(ToolOutputCompressor::detect_language("unknown_tool").is_none());
    }

    #[test]
    fn test_apply_head_tail_respects_line_count() {
        let compressor = ToolOutputCompressor::new();
        let lines: Vec<String> = (0..100).map(|i| format!("Line {}", i)).collect();
        let output = lines.join("\n");

        let compressed = compressor.apply_head_tail(&output, 10, 10);
        let compressed_lines: Vec<&str> = compressed.lines().collect();

        // Should have head (10) + separator (1) + tail (10) + line numbers
        assert!(compressed_lines.len() < 100);
    }

    #[test]
    fn test_apply_smart_truncate() {
        let compressor = ToolOutputCompressor::new();
        let lines: Vec<String> = (0..300)
            .map(|i| {
                if i % 20 == 0 {
                    format!("fn important_func_{}", i)
                } else {
                    format!("Line {}", i)
                }
            })
            .collect();
        let output = lines.join("\n");

        let (compressed, ranges) = compressor.apply_smart_truncate(&output, 100);
        let compressed_lines: Vec<&str> = compressed.lines().collect();

        assert!(compressed_lines.len() <= 150); // Some overhead for context
        assert!(compressed_lines.iter().any(|l| l.contains("important_func")));
    }

    #[test]
    fn test_apply_error_focused() {
        let compressor = ToolOutputCompressor::new();
        let output = "Setup line 1\nSetup line 2\nError: critical issue\nCleanup line 1\nCleanup line 2";

        let compressed = compressor.apply_error_focused(output, 2);

        assert!(compressed.contains("Error: critical issue"));
        assert!(compressed.contains("Setup"));
        assert!(compressed.contains("Cleanup"));
    }

    #[test]
    fn test_compression_config_default() {
        let config = CompressionConfig::default();
        assert_eq!(config.max_output_tokens, 4000);
        assert_eq!(config.max_output_lines, 200);
        assert_eq!(config.max_output_chars, 16000);
        assert_eq!(config.head_lines, 50);
        assert_eq!(config.tail_lines, 30);
        assert!(config.prefer_errors);
        assert!(config.include_line_numbers);
    }

    #[test]
    fn test_compressed_output_serialization() {
        let output = CompressedOutput {
            original_size: 1000,
            compressed_size: 500,
            original_tokens: 250,
            compressed_tokens: 125,
            compression_ratio: 2.0,
            content: "compressed".to_string(),
            strategy_used: CompressionStrategy::HeadTail {
                head_lines: 10,
                tail_lines: 5,
            },
            truncated: true,
            metadata: OutputMetadata {
                total_lines: 100,
                lines_shown: 50,
                language: Some("rust".to_string()),
                has_more: true,
                summary: Some("Summary".to_string()),
            },
        };

        let json = serde_json::to_string(&output).unwrap();
        let restored: CompressedOutput = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.original_size, 1000);
        assert_eq!(restored.compressed_tokens, 125);
        assert!(restored.truncated);
    }

    #[test]
    fn test_strategy_display() {
        assert_eq!(
            CompressionStrategy::None.to_string(),
            "None"
        );
        assert_eq!(
            CompressionStrategy::ErrorFocused.to_string(),
            "ErrorFocused"
        );
    }
}

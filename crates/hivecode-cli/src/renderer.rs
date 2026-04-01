//! Terminal rendering utilities for HiveCode CLI
//!
//! Provides:
//! - Markdown rendering for terminal
//! - Syntax highlighting for code blocks
//! - Progress indicators
//! - Streaming text rendering with live cursor

use colored::*;
use std::io::Write;

/// Render markdown content for terminal display
pub fn render_markdown(content: &str) -> String {
    let mut output = String::new();
    let mut in_code_block = false;
    let mut code_fence = String::new();

    for line in content.lines() {
        // Check for code fence markers
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                code_fence = line[3..].to_string();
                output.push_str(&format!("{}\n", "─".repeat(60).bright_black()));
            } else {
                output.push_str(&format!("{}\n", "─".repeat(60).bright_black()));
            }
            continue;
        }

        if in_code_block {
            // Syntax highlight code blocks
            output.push_str(&highlight_code(line, &code_fence));
            output.push('\n');
        } else if line.starts_with("# ") {
            // Headers
            output.push_str(&format!("{}\n", line[2..].bold().bright_cyan()));
        } else if line.starts_with("## ") {
            output.push_str(&format!("{}\n", line[3..].bold().bright_blue()));
        } else if line.starts_with("### ") {
            output.push_str(&format!("{}\n", line[4..].bold().bright_white()));
        } else if line.starts_with("- ") {
            // Bullet points
            output.push_str(&format!("  {} {}\n", "•".bright_green(), &line[2..]));
        } else if line.starts_with("* ") {
            output.push_str(&format!("  {} {}\n", "•".bright_green(), &line[2..]));
        } else if line.starts_with("1. ") || line.starts_with("2. ") || line.starts_with("3. ") {
            // Numbered lists
            output.push_str(&format!("  {}\n", line));
        } else if line.starts_with("> ") {
            // Blockquotes
            output.push_str(&format!("  {}\n", line[2..].italic().bright_black()));
        } else if line.is_empty() {
            output.push('\n');
        } else {
            output.push_str(&format!("{}\n", line));
        }
    }

    output
}

/// Apply syntax highlighting to code
fn highlight_code(line: &str, language: &str) -> String {
    match language {
        "rust" => highlight_rust(line),
        "python" => highlight_python(line),
        "bash" | "sh" | "shell" => highlight_bash(line),
        "json" => highlight_json(line),
        "toml" => highlight_toml(line),
        "yaml" | "yml" => highlight_yaml(line),
        _ => line.to_string(),
    }
}

fn highlight_rust(line: &str) -> String {
    let mut result = String::new();
    let keywords = ["fn", "pub", "struct", "impl", "trait", "use", "mod", "let", "mut", "const"];

    let mut current = String::new();
    let mut in_string = false;
    let mut in_comment = false;

    for ch in line.chars() {
        match ch {
            '"' if !in_comment => {
                if !in_string {
                    if !current.is_empty() {
                        result.push_str(&format_word(&current));
                        current.clear();
                    }
                    in_string = true;
                    current.push(ch);
                } else {
                    current.push(ch);
                    result.push_str(&current.bright_green().to_string());
                    current.clear();
                    in_string = false;
                }
            }
            '/' if !in_string && line[result.len()..].starts_with("//") => {
                result.push_str(&current);
                result.push_str(&format!("//{}", &line[result.len() + 2..]).bright_black().to_string());
                return result;
            }
            ' ' | '\t' | '(' | ')' | '{' | '}' | '[' | ']' | ':' | ';' | ',' if !in_string => {
                if !current.is_empty() {
                    result.push_str(&format_word(&current));
                    current.clear();
                }
                result.push(ch);
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        if in_string {
            result.push_str(&current.bright_green().to_string());
        } else {
            result.push_str(&format_word(&current));
        }
    }

    result
}

fn highlight_python(line: &str) -> String {
    let keywords = ["def", "class", "import", "from", "if", "else", "for", "while", "return"];
    let mut result = line.to_string();

    for keyword in keywords {
        result = result.replace(
            &format!("{} ", keyword),
            &format!("{} ", keyword.bright_magenta()),
        );
    }

    result
}

fn highlight_bash(line: &str) -> String {
    if line.starts_with("$") || line.starts_with("#") {
        line.bright_yellow().to_string()
    } else {
        line.to_string()
    }
}

fn highlight_json(line: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape_next = false;

    for ch in line.chars() {
        if escape_next {
            result.push(ch);
            escape_next = false;
        } else if ch == '\\' && in_string {
            escape_next = true;
            result.push(ch);
        } else if ch == '"' {
            in_string = !in_string;
            result.push_str(&ch.to_string().bright_green());
        } else if in_string {
            result.push(ch);
        } else if ch == ':' || ch == ',' {
            result.push_str(&ch.to_string().bright_yellow());
        } else if ch == '{' || ch == '}' || ch == '[' || ch == ']' {
            result.push_str(&ch.to_string().bright_cyan());
        } else {
            result.push(ch);
        }
    }

    result
}

fn highlight_toml(line: &str) -> String {
    if line.starts_with("[") && line.ends_with("]") {
        line.bright_cyan().to_string()
    } else if line.contains("=") {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            format!(
                "{} = {}",
                parts[0].bright_magenta(),
                parts[1].bright_green()
            )
        } else {
            line.to_string()
        }
    } else {
        line.to_string()
    }
}

fn highlight_yaml(line: &str) -> String {
    if let Some(pos) = line.find(':') {
        let key = &line[..pos];
        let value = &line[pos..];
        format!("{}{}", key.bright_cyan(), value)
    } else {
        line.to_string()
    }
}

fn format_word(word: &str) -> String {
    let keywords = ["fn", "pub", "struct", "impl", "trait", "use", "mod", "let", "mut", "const", "return", "if", "else"];

    if keywords.contains(&word) {
        word.bright_magenta().to_string()
    } else if word.chars().all(|c| c.is_numeric()) {
        word.bright_cyan().to_string()
    } else {
        word.to_string()
    }
}

/// Display a progress indicator
pub fn show_progress(message: &str, spinner_char: &str) {
    print!("{} {} \r", spinner_char.bright_cyan(), message);
    let _ = std::io::stdout().flush();
}

/// Display streaming text with animation
pub async fn stream_text(text: &str, delay_ms: u64) {
    for ch in text.chars() {
        print!("{}", ch);
        let _ = std::io::stdout().flush();
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }
    println!();
}

/// Format token usage statistics
pub fn format_token_stats(input_tokens: u32, output_tokens: u32) -> String {
    let total = input_tokens + output_tokens;
    format!(
        "{}  Input: {} | Output: {} | Total: {}",
        "Tokens:".bright_black(),
        input_tokens.to_string().bright_cyan(),
        output_tokens.to_string().bright_green(),
        total.to_string().bright_yellow()
    )
}

/// Format tool execution result
pub fn format_tool_result(tool_name: &str, success: bool, content: &str) -> String {
    let status = if success {
        format!("{}", "✓".bright_green())
    } else {
        format!("{}", "✗".bright_red())
    };

    format!(
        "{} {}:\n{}",
        status,
        tool_name.bright_white(),
        content
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_header_rendering() {
        let result = render_markdown("# Hello");
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_bullet_list_rendering() {
        let result = render_markdown("- Item 1\n- Item 2");
        assert!(result.contains("Item 1"));
        assert!(result.contains("Item 2"));
    }

    #[test]
    fn test_code_block_detection() {
        let result = render_markdown("```rust\nfn main() {}\n```");
        assert!(result.contains("main"));
    }

    #[test]
    fn test_token_stats_format() {
        let stats = format_token_stats(100, 50);
        assert!(stats.contains("100"));
        assert!(stats.contains("50"));
        assert!(stats.contains("150"));
    }
}

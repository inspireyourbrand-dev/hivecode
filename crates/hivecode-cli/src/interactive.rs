//! Interactive REPL mode for HiveCode CLI

use crate::{banner, renderer, Args};
use anyhow::Result;
use hivecode_core::AppState;
use rustyline::DefaultEditor;
use rustyline::history::FileHistory;
use colored::*;
use std::path::PathBuf;

/// Context for the interactive session
pub struct InteractiveContext {
    /// Current working directory
    pub cwd: PathBuf,

    /// Current model being used
    pub model: String,

    /// Current provider being used
    pub provider: String,

    /// Whether streaming is enabled
    pub stream_enabled: bool,

    /// Whether to output JSON
    pub json_output: bool,

    /// Max tokens per response
    pub max_tokens: Option<u32>,

    /// Command history file
    pub history_file: PathBuf,
}

impl InteractiveContext {
    /// Create a new interactive context
    pub fn new(args: &Args) -> Result<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let history_file = home.join(".hivecode").join("cli_history");

        Ok(Self {
            cwd: args.project.clone().unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            model: args.model.clone().unwrap_or_else(|| "claude-sonnet-4".to_string()),
            provider: args.provider.clone().unwrap_or_else(|| "anthropic".to_string()),
            stream_enabled: !args.no_stream,
            json_output: args.json,
            max_tokens: args.max_tokens,
            history_file,
        })
    }
}

/// Run the interactive REPL
pub async fn run_interactive(app_state: AppState, args: Args) -> Result<()> {
    let mut context = InteractiveContext::new(&args)?;

    banner::print_banner();
    banner::print_header("Interactive Mode");
    println!("{} {}", "Model:".bright_cyan(), context.model.bright_white());
    println!("{} {}", "Provider:".bright_cyan(), context.provider.bright_white());
    println!("{} {}", "Working Directory:".bright_cyan(), context.cwd.display().to_string().bright_white());
    println!("{} Type {} for help", "Ready!".bright_green(), "'help'".bright_yellow());
    println!();

    let mut rl = DefaultEditor::new()?;

    // Load history if it exists
    let _ = rl.load_history(context.history_file.to_str().unwrap_or(""));

    loop {
        let prompt = format!("hc {} > ", context.model.split('-').next().unwrap_or("claude"))
            .bright_cyan()
            .to_string();

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(trimmed);

                // Handle special commands
                if trimmed.starts_with('/') {
                    handle_command(trimmed, &mut context, &app_state).await?;
                } else {
                    // Send as prompt to LLM
                    send_prompt(trimmed, &context, &app_state).await?;
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("{}", "^C".bright_yellow());
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("{}", "Goodbye!".bright_green());
                break;
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".bright_red(), e);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(context.history_file.to_str().unwrap_or(""));

    Ok(())
}

/// Handle special commands (those starting with /)
async fn handle_command(command: &str, context: &mut InteractiveContext, app_state: &AppState) -> Result<()> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    match parts.get(0).copied() {
        Some("/help") | Some("/h") => show_help(),
        Some("/exit") | Some("/quit") | Some("/q") => {
            println!("{}", "Exiting HiveCode...".bright_green());
            std::process::exit(0);
        }
        Some("/clear") | Some("/cls") => {
            // Try to clear screen
            let _ = std::process::Command::new("clear")
                .status()
                .or_else(|_| std::process::Command::new("cls").status());
        }
        Some("/model") => {
            if let Some(model) = parts.get(1) {
                context.model = model.to_string();
                println!("{} Model changed to: {}", "✓".bright_green(), model.bright_cyan());
            } else {
                println!("{}", context.model.bright_cyan());
            }
        }
        Some("/provider") => {
            if let Some(provider) = parts.get(1) {
                context.provider = provider.to_string();
                println!("{} Provider changed to: {}", "✓".bright_green(), provider.bright_cyan());
            } else {
                println!("{}", context.provider.bright_cyan());
            }
        }
        Some("/cd") => {
            if let Some(path) = parts.get(1) {
                let new_path = PathBuf::from(path);
                if new_path.exists() {
                    context.cwd = new_path.canonicalize()?;
                    println!("{} {}", "✓".bright_green(), context.cwd.display().to_string().bright_cyan());
                } else {
                    println!("{} Path does not exist", "✗".bright_red());
                }
            } else {
                println!("{}", context.cwd.display().to_string().bright_cyan());
            }
        }
        Some("/stream") => {
            context.stream_enabled = !context.stream_enabled;
            let status = if context.stream_enabled { "enabled" } else { "disabled" };
            println!("{} Streaming {}",
                "✓".bright_green(),
                status.bright_cyan()
            );
        }
        Some("/json") => {
            context.json_output = !context.json_output;
            let status = if context.json_output { "enabled" } else { "disabled" };
            println!("{} JSON output {}",
                "✓".bright_green(),
                status.bright_cyan()
            );
        }
        Some("/plan") => {
            println!("{}", "Command plan not yet implemented".bright_yellow());
        }
        Some("/history") => {
            println!("{}", "Conversation history not yet implemented".bright_yellow());
        }
        _ => {
            println!("{} Unknown command: {}", "⚠".bright_yellow(), command.bright_red());
            println!("Type {} for help", "/help".bright_yellow());
        }
    }

    Ok(())
}

/// Send a prompt to the LLM
async fn send_prompt(prompt: &str, context: &InteractiveContext, _app_state: &AppState) -> Result<()> {
    // Show thinking indicator
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_index = 0;

    // Simulate sending to LLM (this would be replaced with actual API call)
    println!();
    println!(
        "{} {} {} {}",
        "Model:".bright_black(),
        context.model.bright_cyan(),
        "| Provider:".bright_black(),
        context.provider.bright_cyan()
    );

    // Mock response for demonstration
    let response = format!(
        "Processing prompt: {}\n\n[This is where the LLM response would appear]",
        prompt
    );

    println!();
    println!("{}", renderer::render_markdown(&response));
    println!();
    println!("{}", renderer::format_token_stats(142, 89).bright_black());
    println!();

    Ok(())
}

/// Show help text
fn show_help() {
    let help_text = r#"
╔════════════════════════════════════════════════════════════╗
║                   HiveCode CLI Commands                    ║
╚════════════════════════════════════════════════════════════╝

NAVIGATION & MODE:
  /exit, /quit, /q      Exit HiveCode
  /clear, /cls          Clear the screen
  /help, /h             Show this help text

CONFIGURATION:
  /model [NAME]         Set or show current model
  /provider [NAME]      Set or show current provider
  /cd [PATH]            Change working directory
  /stream               Toggle streaming (default: on)
  /json                 Toggle JSON output (default: off)

CONVERSATION:
  /plan                 Show conversation plan
  /history              Show conversation history

EXAMPLES:
  > Explain Rust ownership
  > Write a Python function that...
  > /model claude-opus-4
  > /provider openai
  > /stream

TIPS:
  - Use Control+C to interrupt any operation
  - Use Control+D (Unix) or Control+Z (Windows) to exit
  - Multi-line input: use Shift+Enter to continue on next line
  - All responses are streamed live as they arrive
    "#;

    println!("{}", help_text);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_context_creation() {
        let args = Args {
            prompt: None,
            command: None,
            model: Some("test-model".to_string()),
            provider: Some("test-provider".to_string()),
            project: None,
            no_stream: false,
            json: false,
            max_tokens: Some(100),
            continue_conv: false,
            log_level: None,
            config: None,
        };

        let context = InteractiveContext::new(&args).unwrap();
        assert_eq!(context.model, "test-model");
        assert_eq!(context.provider, "test-provider");
        assert_eq!(context.max_tokens, Some(100));
    }
}

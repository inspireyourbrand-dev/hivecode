//! HiveCode CLI - Command-line interface for HiveCode
//!
//! Provides interactive and batch modes for interacting with AI models
//! with full tool execution, streaming responses, and project management.

mod cli;
mod renderer;
mod banner;
mod interactive;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand};
use hivecode_core::{HiveConfig, AppState};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

/// HiveCode CLI - AI-powered development assistant
#[derive(Parser, Debug)]
#[command(name = "hivecode")]
#[command(about = "HiveCode: AI-powered development assistant", long_about = None)]
#[command(version)]
#[command(author = "HivePowered")]
struct Args {
    /// Optional prompt to send (if not provided, enters interactive mode)
    #[arg(value_name = "PROMPT")]
    prompt: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Model to use (e.g., claude-sonnet-4, gpt-4o)
    #[arg(short, long)]
    model: Option<String>,

    /// Provider to use (anthropic, openai, ollama, etc.)
    #[arg(short, long)]
    provider: Option<String>,

    /// Project directory to use
    #[arg(long)]
    project: Option<PathBuf>,

    /// Disable streaming output
    #[arg(long)]
    no_stream: bool,

    /// Output in JSON format
    #[arg(long)]
    json: bool,

    /// Maximum response tokens
    #[arg(long)]
    max_tokens: Option<u32>,

    /// Continue last conversation
    #[arg(long)]
    continue_conv: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,

    /// Config file to use
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Interactive chat mode
    Chat,

    /// Initialize a new HiveCode project
    Init {
        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },

    /// View or edit configuration
    Config {
        #[command(subcommand)]
        action: ConfigActions,
    },

    /// Manage authentication profiles
    Auth {
        #[command(subcommand)]
        action: AuthActions,
    },

    /// Manage plugins
    Plugins {
        #[command(subcommand)]
        action: PluginActions,
    },

    /// Run diagnostics
    Doctor,

    /// Check for and install updates
    Update {
        #[command(subcommand)]
        action: Option<UpdateActions>,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigActions {
    /// Show current configuration
    Show,

    /// Show configuration path
    Path,

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "app.log_level")
        #[arg(value_name = "KEY")]
        key: String,

        /// Configuration value
        #[arg(value_name = "VALUE")]
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        #[arg(value_name = "KEY")]
        key: String,
    },
}

#[derive(Subcommand, Debug)]
enum AuthActions {
    /// List authentication profiles
    List,

    /// Add a new authentication profile
    Add {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: String,

        /// Provider (anthropic, openai, etc.)
        #[arg(value_name = "PROVIDER")]
        provider: String,
    },

    /// Remove an authentication profile
    Remove {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Set default profile
    Default {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Test authentication
    Test {
        /// Profile name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum PluginActions {
    /// List installed plugins
    List,

    /// Install a plugin
    Install {
        /// Plugin ID or URL
        #[arg(value_name = "ID_OR_URL")]
        id_or_url: String,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Enable a plugin
    Enable {
        /// Plugin ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin ID
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Search for plugins
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
    },
}

#[derive(Subcommand, Debug)]
enum UpdateActions {
    /// Check for available updates
    Check,

    /// Set update channel (stable, beta, nightly)
    Channel {
        /// Channel to use
        #[arg(value_name = "CHANNEL")]
        channel: String,
    },

    /// Download and apply update
    Now,

    /// Show update preferences
    Preferences,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = args.log_level.clone().unwrap_or_else(|| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&log_level))
        .with_writer(std::io::stderr)
        .init();

    // Print banner
    banner::print_banner();

    // Load configuration
    let config = if let Some(config_path) = args.config {
        HiveConfig::load_from(&config_path)?
    } else {
        HiveConfig::load().unwrap_or_default()
    };

    // Create app state
    let app_state = AppState::from_config(config).await?;

    // Handle commands
    match args.command {
        Some(Commands::Chat) => {
            interactive::run_interactive(app_state, args).await?;
        }
        Some(Commands::Init { name }) => {
            cli::init::handle_init(name).await?;
        }
        Some(Commands::Config { action }) => {
            cli::config::handle_config_command(action).await?;
        }
        Some(Commands::Auth { action }) => {
            cli::auth::handle_auth_command(action, &app_state).await?;
        }
        Some(Commands::Plugins { action }) => {
            cli::plugins::handle_plugin_command(action, &app_state).await?;
        }
        Some(Commands::Doctor) => {
            cli::doctor::run_diagnostics(&app_state).await?;
        }
        Some(Commands::Update { action }) => {
            cli::updater::handle_update_command(action, &app_state).await?;
        }
        None => {
            // No subcommand - check if we have a prompt
            if let Some(prompt) = args.prompt {
                // Single prompt mode - send and exit
                cli::single_prompt::run_single_prompt(app_state, args, &prompt).await?;
            } else {
                // Enter interactive mode
                interactive::run_interactive(app_state, args).await?;
            }
        }
    }

    Ok(())
}

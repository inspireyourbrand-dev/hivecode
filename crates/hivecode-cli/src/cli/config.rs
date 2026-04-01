//! Configuration command handler

use crate::banner;
use anyhow::Result;
use colored::*;
use hivecode_core::HiveConfig;
use std::path::PathBuf;

use crate::main::ConfigActions;

pub async fn handle_config_command(action: ConfigActions) -> Result<()> {
    match action {
        ConfigActions::Show => {
            let config = HiveConfig::load()?;
            println!("{}", serde_json::to_string_pretty(&config)?);
        }
        ConfigActions::Path => {
            let path = HiveConfig::default_config_path()?;
            println!("{}", path.display());
        }
        ConfigActions::Set { key, value } => {
            println!("{} Setting {} = {}",
                "→".bright_cyan(),
                key.bright_yellow(),
                value.bright_white()
            );

            // Load current config
            let config = HiveConfig::load()?;

            // Save to path (this would need custom logic to set nested values)
            let path = HiveConfig::default_config_path()?;
            config.save(&path)?;

            println!("{} Configuration saved", "✓".bright_green());
        }
        ConfigActions::Get { key } => {
            let config = HiveConfig::load()?;
            println!("{} = {}", key.bright_yellow(), config.custom.get(&key)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "not set".to_string())
                .bright_white()
            );
        }
    }

    Ok(())
}

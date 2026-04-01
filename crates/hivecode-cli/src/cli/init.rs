//! Project initialization command

use anyhow::Result;
use colored::*;
use std::fs;
use std::path::PathBuf;

pub async fn handle_init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or_else(|| "my-hivecode-project".to_string());
    let project_path = PathBuf::from(&project_name);

    println!("{} Initializing HiveCode project: {}",
        "→".bright_cyan(),
        project_name.bright_white()
    );

    // Create project directory
    fs::create_dir_all(&project_path)?;
    println!("  {} Created directory", "✓".bright_green());

    // Create .hivecode directory
    let hivecode_dir = project_path.join(".hivecode");
    fs::create_dir_all(&hivecode_dir)?;
    println!("  {} Created .hivecode directory", "✓".bright_green());

    // Create project config
    let config_content = format!(
        r#"[app]
name = "{}"
version = "0.1.0"

[providers]
default = "anthropic"

[ui]
theme = "auto"
"#,
        project_name
    );
    fs::write(hivecode_dir.join("config.toml"), config_content)?;
    println!("  {} Created config.toml", "✓".bright_green());

    // Create README
    let readme_content = format!(
        r#"# {}

HiveCode project initialized.

## Getting Started

Use HiveCode CLI to interact with AI models:

```bash
hivecode "Your prompt here"
```

## Configuration

Edit `.hivecode/config.toml` to customize settings.
"#,
        project_name
    );
    fs::write(project_path.join("README.md"), readme_content)?;
    println!("  {} Created README.md", "✓".bright_green());

    // Create .gitignore
    let gitignore_content = r#".hivecode/
.env
*.log
"#;
    fs::write(project_path.join(".gitignore"), gitignore_content)?;
    println!("  {} Created .gitignore", "✓".bright_green());

    println!();
    println!("{} Project ready!", "✓".bright_green());
    println!("{} cd {} && hivecode", "Next:".bright_cyan(), project_name);

    Ok(())
}

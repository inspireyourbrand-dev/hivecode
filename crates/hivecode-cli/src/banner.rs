//! ASCII art banner and greeting for HiveCode CLI

use colored::*;

/// Print the HiveCode welcome banner
pub fn print_banner() {
    let banner = r#"
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║                     🐝 HiveCode                          ║
    ║         AI-Powered Development Assistant                 ║
    ║                                                           ║
    ║              Empowering Developer Productivity            ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝
    "#;

    println!("{}", banner.bright_cyan());
    println!("{}  {}",
        "Version:".bright_yellow(),
        env!("CARGO_PKG_VERSION").bright_white()
    );
    println!("{}", "Type 'help' for commands, 'exit' to quit".bright_black());
    println!();
}

/// Print a command tip
pub fn print_tip(tip: &str) {
    println!("{} {}", "Tip:".bright_green(), tip.white());
}

/// Print a status message
pub fn print_status(status: &str, message: &str) {
    match status {
        "success" => println!("{} {}", "✓".bright_green(), message.white()),
        "error" => println!("{} {}", "✗".bright_red(), message.bright_red()),
        "info" => println!("{} {}", "ℹ".bright_blue(), message.white()),
        "warning" => println!("{} {}", "⚠".bright_yellow(), message.bright_yellow()),
        _ => println!("{}", message.white()),
    }
}

/// Print a divider line
pub fn print_divider() {
    println!("{}", "─".repeat(60).bright_black());
}

/// Print section header
pub fn print_header(title: &str) {
    println!();
    println!("{} {}", "═".repeat(3), title.bright_cyan().bold());
    println!("{}", "═".repeat(title.len() + 5).bright_black());
}

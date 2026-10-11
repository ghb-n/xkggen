mod config;
mod generator;

use anyhow::Result;
use clap::Parser;
use config::LayoutConfig;
use generator::{generate_rules, generate_symbols};

#[derive(Parser)]
#[command(name = "xkbgen")]
#[command(about = "Generate custom XKB keyboard layouts")]
struct Cli {
    #[arg(short, long, default_value = "layout.toml")]
    config: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("🔨 xkbgen");
    println!("   Config: {}\n", cli.config);

    let config = LayoutConfig::from_file(&cli.config)?;
    println!("✅ Loaded: {}", config.layout.name);

    let symbols = generate_symbols(&config)?;
    println!("✅ Symbols: {}", symbols.display());

    let rules = generate_rules(&config)?;
    println!("✅ Rules:   {}", rules.display());

    println!("\n💡 Apply:");
    println!("   gsettings set org.gnome.desktop.input-sources sources \"[('xkb', 'custom')]\"");
    println!("   Then log out and log back in.");

    Ok(())
}
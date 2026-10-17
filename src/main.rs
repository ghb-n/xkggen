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

    print_apply_instructions();

    Ok(())
}

/// Print how to activate the generated layout for the current desktop.
///
/// The generated files land in `~/.config/xkb`, which only the *libxkbcommon*
/// keymap compiler reads. On X11 (including KDE's `kwin_x11`) the layout is
/// applied via `setxkbmap`, which has hard-coded system paths and ignores
/// `~/.config/xkb` — so there the files must also be installed under
/// `/usr/share/X11/xkb`. See the README for details.
fn print_apply_instructions() {
    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("XDG_SESSION_DESKTOP"))
        .unwrap_or_default()
        .to_lowercase();

    let is_kde = desktop.contains("kde");
    let is_gnome = desktop.contains("gnome") || desktop.contains("unity");

    println!("\n💡 Apply:");

    if is_kde {
        println!("   KDE (Plasma) — the layout name to select is: custom");
        println!();
        println!("   1. Make the layout visible to setxkbmap/KWin (X11 needs system paths):");
        println!("      sudo cp ~/.config/xkb/symbols/custom /usr/share/X11/xkb/symbols/custom");
        println!("      # then register 'custom' in the system rules so it appears in");
        println!("      # System Settings → Keyboard → Layouts (see README for the snippet).");
        println!();
        println!("   2. Activate it either via the GUI or the config file:");
        println!("      GUI:   System Settings → Keyboard → Layouts → Add Layout → \"custom\"");
        println!("      File:  set LayoutList=custom in ~/.config/kxkbrc, then:");
        println!("             qdbus6 org.kde.KWin /KWin reconfigure");
        println!();
        println!("   3. Verify:");
        println!("      setxkbmap -query      # should show layout: custom");
    } else if is_gnome {
        println!("   GNOME — register the layout as an input source:");
        println!("      gsettings set org.gnome.desktop.input-sources sources \"[('xkb', 'custom')]\"");
        println!("   Then log out and log back in.");
    } else {
        println!("   Select the layout named 'custom' in your desktop's keyboard settings.");
        println!("   Verify with: setxkbmap -layout custom && setxkbmap -query");
    }
}
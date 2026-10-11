use crate::config::LayoutConfig;
use anyhow::{Context, Result};
use dirs::config_dir;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn generate_symbols(config: &LayoutConfig) -> Result<PathBuf> {
    let dir = config_dir()
        .context("No config dir")?
        .join("xkb")
        .join("symbols");
    fs::create_dir_all(&dir)?;

    let path = dir.join("custom");

    let mut by_key: HashMap<String, Vec<&crate::config::Mapping>> = HashMap::new();
    for m in &config.mappings {
        by_key.entry(m.key.to_uppercase()).or_default().push(m);
    }

    let mut content = String::new();
    content.push_str("default partial alphanumeric_keys\n");
    content.push_str("xkb_symbols \"basic\" {\n");
    content.push_str(&format!("    include \"{}(basic)\"\n", config.layout.base));
    content.push_str(&format!(
        "    name[Group1]= \"{}\";\n\n",
        config.layout.description
    ));

    for (key, mappings) in &by_key {
        let key_code = key_to_code(key)?;
        
        let mut levels = [
            key.to_lowercase(),
            key.to_uppercase(),
            key.to_lowercase(),
            key.to_uppercase(),
        ];

        for m in mappings {
            let level = modifier_to_level(&m.modifier);
            if level == 3 {
                levels[2] = m.produces.clone();
            } else if level == 4 {
                levels[3] = m.produces.clone();
            }
        }

        content.push_str(&format!(
            "    key <{}> {{ [ {}, {}, {}, {} ] }};\n",
            key_code, levels[0], levels[1], levels[2], levels[3]
        ));
    }

    // CRITICAL: Force AltGr (ISO_Level3_Shift) to map correctly
    content.push_str("\n    include \"level3(ralt_switch)\"\n");
    content.push_str("};\n");

    fs::write(&path, content)?;
    Ok(path)
}
pub fn generate_rules(config: &LayoutConfig) -> Result<PathBuf> {
    let dir = config_dir()
        .context("No config dir")?
        .join("xkb")
        .join("rules");
    fs::create_dir_all(&dir)?;

    let path = dir.join("evdev.xml");
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE xkbConfigRegistry SYSTEM "xkb.dtd">
<xkbConfigRegistry version="1.1">
  <layoutList>
    <layout>
      <configItem>
        <name>custom</name>
        <shortDescription>Custom</shortDescription>
        <description>{}</description>
      </configItem>
    </layout>
  </layoutList>
</xkbConfigRegistry>
"#,
        config.layout.description
    );

    fs::write(&path, xml)?;
    Ok(path)
}

fn key_to_code(key: &str) -> Result<String> {
    let code = match key.to_uppercase().as_str() {
        // Letters
        "Q" => "AD01", "W" => "AD02", "E" => "AD03", "R" => "AD04",
        "T" => "AD05", "Y" => "AD06", "U" => "AD07", "I" => "AD08",
        "O" => "AD09", "P" => "AD10",
        "A" => "AC01", "S" => "AC02", "D" => "AC03", "F" => "AC04",
        "G" => "AC05", "H" => "AC06", "J" => "AC07", "K" => "AC08",
        "L" => "AC09",
        "Z" => "AB01", "X" => "AB02", "C" => "AB03", "V" => "AB04",
        "B" => "AB05", "N" => "AB06", "M" => "AB07",
        _ => anyhow::bail!("Unknown key: {}", key),
    };
    Ok(code.to_string())
}

fn modifier_to_level(modifier: &str) -> u8 {
    match modifier.to_lowercase().as_str() {
        "altgr" | "rightalt" => 3,
        "shift+altgr" | "altgr+shift" => 4,
        _ => 3,
    }
}
// ```[cite: 2]

// ---

// ### Testing Your Layout

// 1. Save your desired layout into `layout.toml`.
// 2. Compile and run your Rust tool:
//    ```bash
//    cargo run -- --config layout.toml
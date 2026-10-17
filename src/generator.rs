use crate::config::LayoutConfig;
use anyhow::{Context, Result};
use dirs::config_dir;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub fn generate_symbols(config: &LayoutConfig) -> Result<PathBuf> {
    let dir = config_dir()
        .context("No config dir")?
        .join("xkb")
        .join("symbols");
    fs::create_dir_all(&dir)?;

    let path = dir.join("custom");

    // BTreeMap: emit keys in alphabetical order so the generated file is stable.
    let mut by_key: BTreeMap<String, Vec<&crate::config::Mapping>> = BTreeMap::new();
    for m in &config.mappings {
        by_key.entry(m.key.to_uppercase()).or_default().push(m);
    }

    let mut content = String::new();
    content.push_str("default partial alphanumeric_keys\n");
    content.push_str("xkb_symbols \"basic\" {\n");
    // `base` is used verbatim: it may be a bare layout ("us", "ru") or an
    // explicit variant ("ru(phonetic)"). Do NOT append "(basic)" — not every
    // layout has a "basic" variant (e.g. ru), and the include would fail,
    // leaving the whole layout unapplied.
    content.push_str(&format!("    include \"{}\"\n", config.layout.base));
    content.push_str(&format!(
        "    name[Group1]= \"{}\";\n\n",
        config.layout.description
    ));

    for (key, mappings) in &by_key {
        let key_code = key_to_code(key)?;

        // Every level defaults to NoSymbol: the keymap compiler then keeps
        // whatever the base layout defines on that level, and only levels
        // explicitly mapped in the config are overridden. (Filling the gaps
        // with the Latin key name instead would clobber base letters, e.g.
        // Russian "о" on <AC07> would become Latin "j".)
        let mut levels = ["NoSymbol"; 4];

        for m in mappings {
            if m.produces.chars().any(|c| !c.is_ascii()) {
                anyhow::bail!(
                    "mapping for key {}: produces = \"{}\" must be an ASCII XKB keysym name \
                     (write \"Cyrillic_o\" instead of \"о\")",
                    m.key,
                    m.produces
                );
            }
            let level = modifier_to_level(&m.modifier)?;
            levels[level - 1] = m.produces.as_str();
        }

        // The explicit FOUR_LEVEL type is essential: without it the compiler
        // resolves the type from the first keysym (ALPHABETIC, 2 levels) and
        // pushes levels 3-4 into a second group, where AltGr can never reach
        // them — so the mapped symbol would be unreachable.
        content.push_str(&format!(
            "    key <{}> {{ type[Group1]= \"FOUR_LEVEL\", symbols[Group1]= [ {}, {}, {}, {} ] }};\n",
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

/// Map a config modifier string to its XKB level (1-based).
///
/// Levels: 1 = plain key, 2 = Shift, 3 = AltGr (Right Alt), 4 = Shift+AltGr.
/// Unknown modifiers are an error — previously they silently became level 3,
/// which put base-level mappings (e.g. plain `J → O`) on the AltGr level so
/// the key still produced its original letter.
fn modifier_to_level(modifier: &str) -> Result<usize> {
    let normalized = modifier.trim().to_lowercase().replace(' ', "");
    match normalized.as_str() {
        "" | "none" | "base" | "normal" => Ok(1),
        "shift" => Ok(2),
        "altgr" | "rightalt" => Ok(3),
        "shift+altgr" | "altgr+shift" | "shift+rightalt" | "rightalt+shift" => Ok(4),
        other => anyhow::bail!(
            "unknown modifier \"{}\" — expected: none, shift, altgr or shift+altgr",
            other
        ),
    }
}
// ```[cite: 2]

// ---

// ### Testing Your Layout

// 1. Save your desired layout into `layout.toml`.
// 2. Compile and run your Rust tool:
//    ```bash
//    cargo run -- --config layout.toml
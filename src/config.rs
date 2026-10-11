use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct LayoutConfig {
    pub layout: Layout,
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Deserialize)]
pub struct Layout {
    pub name: String,
    pub description: String,
    pub base: String,
}

#[derive(Debug, Deserialize)]
pub struct Mapping {
    pub key: String,
    pub modifier: String,
    pub produces: String,
}

impl LayoutConfig {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path))?;
        let config: LayoutConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse TOML")?;
        Ok(config)
    }
}
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Project-level configuration loaded from `.claude-agent-team.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,
    #[serde(default = "default_team")]
    pub default_team: String,
    #[serde(default = "default_base_branch")]
    pub base_branch: String,
}

fn default_specs_dir() -> String {
    "docs/specs".to_string()
}

fn default_team() -> String {
    "feature-dev".to_string()
}

fn default_base_branch() -> String {
    "main".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            specs_dir: default_specs_dir(),
            default_team: default_team(),
            base_branch: default_base_branch(),
        }
    }
}

impl Config {
    /// Load config from a `.claude-agent-team.toml` file in the given directory.
    /// Returns defaults if the file does not exist.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(".claude-agent-team.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents =
            std::fs::read_to_string(&path).context("Failed to read .claude-agent-team.toml")?;
        let config: Config = toml::from_str(&contents).context("Failed to parse config TOML")?;
        Ok(config)
    }
}

/// Discover spec files (`.md` only, no subdirectories) in the given specs directory.
pub fn discover_specs(specs_dir: &Path) -> Result<Vec<String>> {
    let entries = std::fs::read_dir(specs_dir).context("Failed to read specs directory")?;
    let mut specs = Vec::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".md") {
                    specs.push(name.to_string());
                }
            }
        }
    }
    specs.sort();
    Ok(specs)
}

/// Discover team files from `prompts/teams/` directory, returning names without `.md` extension.
pub fn discover_teams(teams_dir: &Path) -> Result<Vec<String>> {
    let entries = std::fs::read_dir(teams_dir).context("Failed to read teams directory")?;
    let mut teams = Vec::new();
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(stem) = name.strip_suffix(".md") {
                    teams.push(stem.to_string());
                }
            }
        }
    }
    teams.sort();
    Ok(teams)
}

#[cfg(test)]
mod tests;

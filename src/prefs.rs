use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default)]
    pub headless: bool,
    #[serde(default = "default_true")]
    pub show_complete: bool,
    #[serde(default = "default_true")]
    pub show_blocked: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            headless: false,
            show_complete: true,
            show_blocked: true,
        }
    }
}

impl Prefs {
    fn default_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|h| {
            PathBuf::from(h)
                .join(".claude")
                .join("claude-agent-team-prefs.toml")
        })
    }

    /// Load prefs from the default path. Returns defaults on any error.
    pub fn load() -> Self {
        Self::default_path()
            .and_then(|p| Self::load_from_path(&p).ok())
            .unwrap_or_default()
    }

    /// Save prefs to the default path. Non-fatal: logs a warning on failure.
    pub fn save(&self) {
        if let Some(path) = Self::default_path() {
            if let Err(e) = self.save_to_path(&path) {
                eprintln!("Warning: Failed to save prefs: {e:#}");
            }
        }
    }

    pub(crate) fn load_from_path(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        toml::from_str(&text).context("Failed to parse prefs TOML")
    }

    pub(crate) fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self).context("Failed to serialize prefs")?;
        std::fs::write(path, text).with_context(|| format!("Failed to write {}", path.display()))
    }
}

#[cfg(test)]
mod tests;

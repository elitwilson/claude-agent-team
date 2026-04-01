use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AccountEntry {
    pub label: String,
}

#[derive(Debug, Deserialize)]
struct AccountsFile {
    #[serde(default)]
    accounts: Vec<AccountEntry>,
}

/// Load accounts from `~/.claude/claude-agent-team-accounts.toml`.
/// Returns an empty vec if the file does not exist or has no entries.
pub fn load_accounts() -> Vec<AccountEntry> {
    default_accounts_path()
        .and_then(|p| load_accounts_from_path(&p).ok())
        .unwrap_or_default()
}

/// Testable variant: load accounts from an arbitrary path.
pub fn load_accounts_from_path(path: &Path) -> Result<Vec<AccountEntry>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let file: AccountsFile = toml::from_str(&text).context("Failed to parse accounts TOML")?;
    Ok(file.accounts)
}

/// Load an OAuth token from the macOS Keychain for the given account label.
/// Service: `com.claude-agent-team`, account: `label`.
/// Returns `None` on any error (missing entry, Keychain failure, etc.).
pub fn load_token_for_account(label: &str) -> Option<String> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-w",
            "-s",
            "com.claude-agent-team",
            "-a",
            label,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() { None } else { Some(token) }
}

fn default_accounts_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| {
        PathBuf::from(h)
            .join(".claude")
            .join("claude-agent-team-accounts.toml")
    })
}

#[cfg(test)]
mod tests;

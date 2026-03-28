use std::fs::{self, File};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

/// Build the argument list for the `claude` command.
pub fn build_claude_args(rendered_prompt: &str, headless: bool) -> Vec<String> {
    let mut args = Vec::new();
    if headless {
        args.push("--print".to_string());
    }
    args.extend([
        "--max-turns".to_string(),
        "200".to_string(),
        "--dangerously-skip-permissions".to_string(),
        "--teammate-mode".to_string(),
        "in-process".to_string(),
        rendered_prompt.to_string(),
    ]);
    args
}

/// Build the log file path for a run.
pub fn build_log_path(feature_slug: &str, date: &str) -> String {
    format!("logs/agent-runs/{feature_slug}-{date}.log")
}

/// Load OAuth token from macOS Keychain.
pub fn load_oauth_token() -> Option<String> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-w",
            "-s",
            "claude-token-1",
            "-a",
            "claude",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() { None } else { Some(token) }
}

/// Spawn the claude process and wait for it to exit. Returns the exit code.
pub fn run_claude(
    rendered_prompt: &str,
    headless: bool,
    log_path: &str,
    oauth_token: Option<&str>,
) -> Result<i32> {
    // Ensure log directory exists
    if let Some(parent) = std::path::Path::new(log_path).parent() {
        fs::create_dir_all(parent).context("Failed to create log directory")?;
    }

    let args = build_claude_args(rendered_prompt, headless);
    let mut cmd = Command::new("claude");
    cmd.args(&args);
    cmd.env("CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS", "1");

    if let Some(token) = oauth_token {
        cmd.env("CLAUDE_OAUTH_TOKEN", token);
    }

    if headless {
        // Headless: redirect stdout/stderr to log, stdin from /dev/null
        let log_file = File::create(log_path).context("Failed to create log file")?;
        let log_err = log_file
            .try_clone()
            .context("Failed to clone log file handle")?;
        cmd.stdin(Stdio::null());
        cmd.stdout(log_file);
        cmd.stderr(log_err);
    } else {
        // Interactive: inherit stdio directly so claude gets a real TTY.
        // No log file in interactive mode — tee breaks the interactive UI.
        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
    }

    let mut child = cmd.spawn().context("Failed to spawn claude process")?;
    let status = child.wait().context("Failed to wait for claude process")?;

    Ok(status.code().unwrap_or(-1))
}

#[cfg(test)]
mod tests;

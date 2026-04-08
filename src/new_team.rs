use std::io::{self, BufRead, Write as IoWrite};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const TEAM_SCAFFOLD: &str = "\
This is a scaffolded team prompt that has not been configured.\n\
Replace this file with your own prompt engineering before running.\n\
Exiting.\n";

const AGENT_SCAFFOLD: &str = "\
This is a scaffolded agent definition that has not been configured.\n\
Replace this file with your own prompt engineering before running.\n\
Exiting.\n";

/// Validate that a team name matches `^[a-z0-9-]+$`.
pub(crate) fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Team name must not be empty");
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        anyhow::bail!(
            "Invalid team name '{}': only lowercase letters, digits, and hyphens are allowed",
            name
        );
    }
    Ok(())
}

/// Resolve the target root directory for the given level.
///
/// - `user` → `<workflow_dir>/user`
/// - `project` → `<cwd>/<custom_dir>`; fails if `custom_dir` is `None`
pub(crate) fn resolve_target_root(
    level: &str,
    workflow_dir: &str,
    custom_dir: Option<&str>,
    cwd: &Path,
) -> Result<PathBuf> {
    if level != "user" && level != "project" {
        anyhow::bail!("Level must be 'user' or 'project', got '{level}'");
    }
    match level {
        "project" => {
            let dir = custom_dir.ok_or_else(|| {
                anyhow::anyhow!(
                    "custom_dir is not set in .claude-launch.toml — add it before creating a project-level team"
                )
            })?;
            Ok(cwd.join(dir))
        }
        _ => Ok(PathBuf::from(format!("{}/user", workflow_dir))),
    }
}

/// Scaffold team and agent files under `root`.
///
/// Checks that neither output file exists before writing anything (no partial writes).
/// Returns `(team_path, agent_path)` on success.
pub(crate) fn scaffold_team(name: &str, root: &Path) -> Result<(PathBuf, PathBuf)> {
    let team_path = root.join("teams").join(format!("{name}.md"));
    let agent_path = root.join("agents").join(name).join("agent.md");

    if team_path.exists() {
        anyhow::bail!("'{}' already exists", team_path.display());
    }
    if agent_path.exists() {
        anyhow::bail!("'{}' already exists", agent_path.display());
    }

    // Create directories
    if let Some(parent) = team_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }
    if let Some(parent) = agent_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    std::fs::write(&team_path, TEAM_SCAFFOLD)
        .with_context(|| format!("Failed to write {}", team_path.display()))?;
    std::fs::write(&agent_path, AGENT_SCAFFOLD)
        .with_context(|| format!("Failed to write {}", agent_path.display()))?;

    Ok((team_path, agent_path))
}

/// Entry point called from `main.rs`.
pub fn run(args: &[String]) -> Result<()> {
    use crate::{config, prompt};

    let stdin = io::stdin();
    let stdout = io::stdout();

    // Step 1: resolve name — from positional arg or prompt
    let name = if args.first().map(|s| !s.starts_with('-')).unwrap_or(false) {
        args[0].clone()
    } else {
        let mut out = stdout.lock();
        write!(out, "Team name: ")?;
        out.flush()?;
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        line.trim().to_string()
    };

    // Step 2: validate name
    validate_name(&name)?;

    // Step 3: prompt for level
    {
        let mut out = stdout.lock();
        write!(out, "Level (user/project) [user]: ")?;
        out.flush()?;
    }
    let mut level_line = String::new();
    stdin.lock().read_line(&mut level_line)?;
    let level = {
        let trimmed = level_line.trim();
        if trimmed.is_empty() { "user" } else { trimmed }.to_string()
    };

    // Step 4: resolve workflow dir and config
    let workflow_dir = prompt::resolve_workflow_dir()?;
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let config = config::Config::load(&cwd)?;

    let target_root = resolve_target_root(&level, &workflow_dir, config.custom_dir.as_deref(), &cwd)?;

    // If project level and root doesn't exist, create it
    if level == "project" && !target_root.exists() {
        std::fs::create_dir_all(&target_root)
            .with_context(|| format!("Failed to create project dir: {}", target_root.display()))?;
    }

    // Step 5: collision check via discover_teams
    let builtin_teams_dir = std::path::Path::new(&workflow_dir).join("prompts").join("teams");
    let user_dir = format!("{}/user", workflow_dir);
    let user_teams_dir = std::path::Path::new(&user_dir).join("teams");
    let project_teams_dir = config.custom_dir.as_ref().map(|d| cwd.join(d).join("teams"));
    let project_teams_dir_ref = project_teams_dir.as_deref();

    // For collision check we only pass project_teams_dir if it actually exists,
    // otherwise discover_teams would bail on a configured but nonexistent dir.
    let effective_project = project_teams_dir_ref.filter(|p| p.exists());
    let team_entries = config::discover_teams(&builtin_teams_dir, &user_teams_dir, effective_project)
        .context("Failed to discover existing teams")?;

    if team_entries.iter().any(|e| e.name == name) {
        anyhow::bail!("A team named '{}' already exists", name);
    }

    // Step 6 & 7: check paths and scaffold
    let (team_path, agent_path) = scaffold_team(&name, &target_root)?;

    // Step 8: print results
    println!("Created:");
    println!("  {}", team_path.display());
    println!("  {}", agent_path.display());
    println!();
    println!("Edit these files to define your team, then run claude-launch to use it.");

    Ok(())
}

#[cfg(test)]
mod tests;

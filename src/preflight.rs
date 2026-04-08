use std::process::Command;

use anyhow::{Context, Result, bail};

/// Build the feature branch name from the slug.
pub fn build_branch_name(feature_slug: &str) -> String {
    format!("feature/{feature_slug}")
}

/// Run preflight checks: ensure clean working tree, validate branches, create feature branch.
pub fn run_preflight(base_branch: &str, feature_slug: &str) -> Result<String> {
    check_clean_working_tree()?;

    let branch_name = build_branch_name(feature_slug);

    check_branch_exists(base_branch)?;
    check_branch_absent(&branch_name)?;

    run_git(&["checkout", base_branch])
        .with_context(|| format!("Failed to checkout {base_branch}"))?;

    run_git(&["checkout", "-b", &branch_name])
        .with_context(|| format!("Failed to create branch {branch_name}"))?;

    Ok(branch_name)
}

/// Check if the working tree is clean (no uncommitted changes).
pub fn check_clean_working_tree() -> Result<()> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to run git status")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_changes = stdout.lines().any(|l| !l.starts_with("??"));
    if has_changes {
        bail!("Working tree is not clean. Please commit or stash your changes first.");
    }
    Ok(())
}

fn check_branch_exists(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .output()
        .context("Failed to check branch existence")?;
    if !output.status.success() {
        bail!("Base branch '{branch}' does not exist locally.");
    }
    Ok(())
}

fn check_branch_absent(branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", branch])
        .output()
        .context("Failed to check branch existence")?;
    if output.status.success() {
        bail!("Branch '{branch}' already exists. Did a previous run leave it behind?");
    }
    Ok(())
}

fn run_git(args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("Failed to run git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(())
}

#[cfg(test)]
mod tests;

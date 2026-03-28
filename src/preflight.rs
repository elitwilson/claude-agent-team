use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::Local;

/// Build the feature branch name from the slug and today's date (YYYYMMDD).
pub fn build_branch_name(feature_slug: &str, date: &str) -> String {
    format!("feature/{feature_slug}-{date}")
}

/// Run preflight checks: ensure clean working tree, checkout base branch, pull, create feature branch.
pub fn run_preflight(base_branch: &str, feature_slug: &str) -> Result<String> {
    check_clean_working_tree()?;

    let date = Local::now().format("%Y%m%d").to_string();
    let branch_name = build_branch_name(feature_slug, &date);

    // Checkout base branch
    run_git(&["checkout", base_branch])
        .with_context(|| format!("Failed to checkout {base_branch}"))?;

    // Pull latest
    run_git(&["pull"]).context("Failed to pull latest changes")?;

    // Create and checkout feature branch
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
    if !stdout.trim().is_empty() {
        bail!("Working tree is not clean. Please commit or stash your changes first.");
    }
    Ok(())
}

/// Create a feature branch from the base branch.
pub fn create_feature_branch(base_branch: &str, feature_slug: &str) -> Result<String> {
    let date = Local::now().format("%Y%m%d").to_string();
    let branch_name = build_branch_name(feature_slug, &date);
    run_git(&["checkout", base_branch])?;
    run_git(&["checkout", "-b", &branch_name])?;
    Ok(branch_name)
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

use std::process::Command;

use anyhow::{bail, Context, Result};

/// Build the MR title, prefixing with "INCOMPLETE:" if exit code is non-zero.
pub fn build_mr_title(feature_slug: &str, exit_code: i32) -> String {
    if exit_code != 0 {
        format!("INCOMPLETE: {feature_slug}")
    } else {
        feature_slug.to_string()
    }
}

/// Build the MR description. Includes a warning if exit code is non-zero.
pub fn build_mr_description(feature_slug: &str, exit_code: i32) -> String {
    if exit_code != 0 {
        format!(
            "Automated MR for {feature_slug}\n\n\
             **WARNING:** This run exited with code {exit_code}. \
             The implementation may be incomplete."
        )
    } else {
        format!("Automated MR for {feature_slug}")
    }
}

/// Build the git push arguments including GitLab push options for MR creation.
pub fn build_push_args(
    branch: &str,
    base_branch: &str,
    title: &str,
    description: &str,
) -> Vec<String> {
    vec![
        "push".to_string(),
        "-u".to_string(),
        "origin".to_string(),
        branch.to_string(),
        "-o".to_string(),
        "merge_request.create".to_string(),
        "-o".to_string(),
        format!("merge_request.target={base_branch}"),
        "-o".to_string(),
        format!("merge_request.title={title}"),
        "-o".to_string(),
        format!("merge_request.description={description}"),
    ]
}

/// Create a GitLab MR via git push with push options.
pub fn create_mr(feature_slug: &str, base_branch: &str, exit_code: i32) -> Result<()> {
    // Get current branch name
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to get current branch")?;

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    let title = build_mr_title(feature_slug, exit_code);
    let description = build_mr_description(feature_slug, exit_code);
    let args = build_push_args(&branch, base_branch, &title, &description);

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to push and create MR")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git push failed: {}", stderr.trim());
    }

    Ok(())
}

/// Format a post-run summary string.
pub fn format_summary(branch: &str, mr_created: bool, metrics_written: bool) -> String {
    let mr_status = if mr_created {
        "MR created"
    } else {
        "MR creation failed"
    };
    let metrics_status = if metrics_written {
        "metrics written"
    } else {
        "metrics collection failed"
    };
    format!("Branch: {branch} | {mr_status} | {metrics_status}")
}

#[cfg(test)]
mod tests;

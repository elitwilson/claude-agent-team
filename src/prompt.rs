use std::path::Path;

use anyhow::{bail, Context, Result};

/// Load a prompt template and substitute variables.
pub fn render_prompt(
    template_path: &Path,
    spec_file: &str,
    feature_slug: &str,
    workflow_dir: &str,
    team: &str,
) -> Result<String> {
    let template = std::fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read template at {}", template_path.display()))?;

    let rendered = template
        .replace("${SPEC_FILE}", spec_file)
        .replace("${FEATURE_SLUG}", feature_slug)
        .replace("${WORKFLOW_DIR}", workflow_dir)
        .replace("${TEAM}", team);

    Ok(rendered)
}

/// Resolve the workflow directory (repo root containing `prompts/`).
/// Checks `CLAUDE_AGENT_TEAM_DIR` env var first, then walks up from the binary location.
pub fn resolve_workflow_dir() -> Result<String> {
    // Check env var first
    if let Ok(dir) = std::env::var("CLAUDE_AGENT_TEAM_DIR") {
        if Path::new(&dir).join("prompts").exists() {
            return Ok(dir);
        }
    }

    // Walk up from the binary's location
    let exe = std::env::current_exe().context("Failed to determine binary location")?;
    let mut dir = exe
        .parent()
        .map(Path::to_path_buf)
        .context("Binary has no parent directory")?;

    loop {
        if dir.join("prompts").exists() {
            return dir
                .to_str()
                .map(String::from)
                .context("Workflow dir path is not valid UTF-8");
        }
        if !dir.pop() {
            break;
        }
    }

    bail!("Could not find workflow directory (no ancestor contains a prompts/ subdirectory)")
}

#[cfg(test)]
mod tests;

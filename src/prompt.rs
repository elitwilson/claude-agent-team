use std::path::Path;

use anyhow::{Context, Result};
use include_dir::{Dir, include_dir};

static WORKFLOW_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/prompts");
static RULES_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/rules");
static HOOKS_FILES: Dir = include_dir!("$CARGO_MANIFEST_DIR/hooks");

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

/// Load the drafter prompt template and substitute variables.
pub fn render_drafter_prompt(
    template_path: &Path,
    input_file: &str,
    specs_dir: &str,
    workflow_dir: &str,
) -> Result<String> {
    let template = std::fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read template at {}", template_path.display()))?;

    let rendered = template
        .replace("${INPUT_FILE}", input_file)
        .replace("${SPECS_DIR}", specs_dir)
        .replace("${WORKFLOW_DIR}", workflow_dir);

    Ok(rendered)
}

/// Resolve the workflow directory. Checks `CLAUDE_LAUNCH_DIR` env var first,
/// then extracts embedded files to `~/.claude-launch/` and returns that path.
pub fn resolve_workflow_dir() -> Result<String> {
    // Check env var first (allows override for development)
    if let Ok(dir) = std::env::var("CLAUDE_LAUNCH_DIR") {
        if Path::new(&dir).join("prompts").exists() {
            return Ok(dir);
        }
    }

    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let workflow_dir = Path::new(&home).join(".claude-launch");
    std::fs::create_dir_all(&workflow_dir).context("Failed to create ~/.claude-launch")?;

    extract_dir(&WORKFLOW_FILES, &workflow_dir.join("prompts"))?;
    extract_dir(&RULES_FILES, &workflow_dir.join("rules"))?;
    extract_dir(&HOOKS_FILES, &workflow_dir.join("hooks"))?;

    workflow_dir
        .to_str()
        .map(String::from)
        .context("Workflow dir path is not valid UTF-8")
}

/// Recursively extract an embedded Dir to a filesystem path.
fn extract_dir(dir: &Dir, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for file in dir.files() {
        let target = dest.join(file.path());
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&target, file.contents())?;
    }
    for subdir in dir.dirs() {
        extract_dir(subdir, dest)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;

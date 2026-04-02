use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Returns true if claude-launch has already been installed (hooks registered + rules symlinked).
pub fn is_installed() -> bool {
    let Ok(home) = std::env::var("HOME") else {
        return false;
    };
    let link = Path::new(&home)
        .join(".claude")
        .join("rules")
        .join("agent-workflow");
    link.is_symlink() || link.is_dir()
}

/// Symlink <workflow_dir>/rules into <claude_dir>/rules/agent-workflow.
/// Idempotent: skips if the symlink already exists.
pub fn link_rules(workflow_dir: &Path, claude_dir: &Path) -> Result<()> {
    let rules_dir = claude_dir.join("rules");
    std::fs::create_dir_all(&rules_dir).context("Failed to create rules directory")?;

    let link = rules_dir.join("agent-workflow");
    let target = workflow_dir.join("rules");

    if link.is_symlink() {
        println!("  rules symlink already exists, skipping");
        return Ok(());
    }
    if link.exists() {
        bail!(
            "{} exists but is not a symlink. Remove it manually and retry.",
            link.display()
        );
    }

    std::os::unix::fs::symlink(&target, &link).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            link.display(),
            target.display()
        )
    })?;
    println!("  linked rules/ -> {}", link.display());
    Ok(())
}

/// Register TaskCompleted, TaskCreated, and TeammateIdle hooks in settings.json.
/// Idempotent: skips hooks that are already registered.
pub fn register_hooks(workflow_dir: &Path, settings_path: &Path) -> Result<()> {
    let mut settings: serde_json::Value = if settings_path.exists() {
        let text =
            std::fs::read_to_string(settings_path).context("Failed to read settings.json")?;
        serde_json::from_str(&text).context("Failed to parse settings.json")?
    } else {
        serde_json::json!({})
    };

    let hooks_map = settings
        .as_object_mut()
        .context("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hook_scripts = [
        ("TaskCompleted", "task-completed.sh"),
        ("TaskCreated", "task-created.sh"),
        ("TeammateIdle", "teammate-idle.sh"),
    ];

    let mut changed = false;
    for (event, script_name) in &hook_scripts {
        let script_path = workflow_dir
            .join("hooks")
            .join(script_name)
            .to_string_lossy()
            .to_string();

        let entries = hooks_map
            .as_object_mut()
            .context("hooks is not an object")?
            .entry(*event)
            .or_insert_with(|| serde_json::json!([]));

        let already_registered = entries
            .as_array()
            .map(|arr| {
                arr.iter().any(|entry| {
                    entry
                        .get("hooks")
                        .and_then(|h| h.as_array())
                        .map(|hooks| {
                            hooks.iter().any(|h| {
                                h.get("command").and_then(|c| c.as_str()) == Some(&script_path)
                            })
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !already_registered {
            entries
                .as_array_mut()
                .context("hook entries is not an array")?
                .push(serde_json::json!({
                    "hooks": [{"type": "command", "command": script_path}]
                }));
            changed = true;
        }
    }

    if changed {
        if settings_path.exists() {
            let backup = settings_path.with_extension("json.bak");
            std::fs::copy(settings_path, &backup).with_context(|| {
                format!("Failed to back up settings.json to {}", backup.display())
            })?;
        }
        let text =
            serde_json::to_string_pretty(&settings).context("Failed to serialize settings.json")?;
        std::fs::write(settings_path, text).context("Failed to write settings.json")?;
        println!("  registered hooks in settings.json");
    } else {
        println!("  hooks already registered, skipping");
    }

    Ok(())
}

/// Run the full install sequence.
pub fn run_install() -> Result<()> {
    let workflow_dir_str = crate::prompt::resolve_workflow_dir()?;
    let workflow_dir = PathBuf::from(&workflow_dir_str);
    println!("Installing from: {}", workflow_dir.display());

    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let claude_dir = Path::new(&home).join(".claude");
    let settings_path = claude_dir.join("settings.json");

    link_rules(&workflow_dir, &claude_dir)?;
    register_hooks(&workflow_dir, &settings_path)?;
    println!("\nInstall complete.");
    Ok(())
}

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Status of a spec file, parsed from YAML frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecStatus {
    Ready,
    Complete,
    Blocked,
    /// File has no frontmatter — treated as raw requirements input for the Drafter.
    Raw,
}

/// A discovered spec file with its parsed status and optional block reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecEntry {
    pub name: String,
    pub status: SpecStatus,
    pub block_reason: Option<String>,
}

/// Parsed YAML frontmatter from a spec file.
pub struct SpecFrontmatter {
    pub status: SpecStatus,
    pub block_reason: Option<String>,
    pub base_branch: Option<String>,
}

/// Parse the YAML frontmatter from a spec file's content, returning status,
/// block reason, and base_branch.
///
/// Rules:
/// - No frontmatter → `status: Raw`, everything else `None`
/// - Has frontmatter, has `status: blocked` or `needs_attention` → `status: Blocked`,
///   `block_reason: Some("Spec is marked blocked — requires human review before running.")`
/// - Has frontmatter, missing `base_branch` → `status: Blocked`,
///   `block_reason: Some("Missing required frontmatter field: base_branch")`
/// - Has frontmatter, has valid `status`, has `base_branch` → normal status, no block reason
pub fn parse_spec_frontmatter(content: &str) -> SpecFrontmatter {
    let Some(fm) = extract_frontmatter(content) else {
        return SpecFrontmatter {
            status: SpecStatus::Raw,
            block_reason: None,
            base_branch: None,
        };
    };

    let mut raw_status: Option<&str> = None;
    let mut base_branch: Option<String> = None;

    for line in fm.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("status:") {
            raw_status = Some(value.trim());
        } else if let Some(value) = line.strip_prefix("base_branch:") {
            base_branch = Some(value.trim().to_string());
        }
    }

    // Check for explicitly blocked status first
    match raw_status {
        Some("needs_attention") | Some("blocked") => {
            return SpecFrontmatter {
                status: SpecStatus::Blocked,
                block_reason: Some(
                    "Spec is marked blocked — requires human review before running.".to_string(),
                ),
                base_branch,
            };
        }
        _ => {}
    }

    // Require base_branch for all specs with frontmatter, except complete specs —
    // they're done and non-interactable, so missing base_branch doesn't block them.
    if base_branch.is_none() && raw_status != Some("complete") {
        return SpecFrontmatter {
            status: SpecStatus::Blocked,
            block_reason: Some("Missing required frontmatter field: base_branch".to_string()),
            base_branch: None,
        };
    }

    let status = match raw_status {
        Some("ready") => SpecStatus::Ready,
        Some("complete") => SpecStatus::Complete,
        _ => SpecStatus::Ready,
    };

    SpecFrontmatter {
        status,
        block_reason: None,
        base_branch,
    }
}

/// Extract the YAML frontmatter block (content between `---` delimiters).
fn extract_frontmatter(content: &str) -> Option<&str> {
    let content = content.trim_start();
    let rest = content.strip_prefix("---")?;
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    if rest.starts_with("---") {
        return Some("");
    }
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Project-level configuration loaded from `.claude-launch.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_specs_dir")]
    pub specs_dir: String,
    #[serde(default = "default_team")]
    pub default_team: String,
    #[serde(default)]
    pub custom_dir: Option<String>,
}

fn default_specs_dir() -> String {
    "docs/specs".to_string()
}

fn default_team() -> String {
    "feature-dev".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            specs_dir: default_specs_dir(),
            default_team: default_team(),
            custom_dir: None,
        }
    }
}

impl Config {
    /// Load config from a `.claude-launch.toml` file in the given directory.
    /// Returns defaults if the file does not exist.
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(".claude-launch.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents =
            std::fs::read_to_string(&path).context("Failed to read .claude-launch.toml")?;
        let config: Config = toml::from_str(&contents).context("Failed to parse config TOML")?;
        Ok(config)
    }
}

/// Discover spec and requirements files (no subdirectories) in the given specs directory.
/// Reads all text files; skips binaries (non-UTF-8).
pub fn discover_specs(specs_dir: &Path) -> Result<Vec<SpecEntry>> {
    let entries = std::fs::read_dir(specs_dir).context("Failed to read specs directory")?;
    let mut specs = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue; // skip binaries
        };
        let fm = parse_spec_frontmatter(&content);
        specs.push(SpecEntry {
            name,
            status: fm.status,
            block_reason: fm.block_reason,
        });
    }
    specs.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(specs)
}

/// Read `base_branch` from a spec file's frontmatter. Returns an error if the
/// file cannot be read or the field is missing.
pub fn read_base_branch(spec_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path.display()))?;
    let fm = parse_spec_frontmatter(&content);
    fm.base_branch.ok_or_else(|| {
        anyhow::anyhow!(
            "Spec '{}' is missing required frontmatter field: base_branch",
            spec_path.file_name().unwrap_or_default().to_string_lossy()
        )
    })
}

/// Which source a team was loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamSource {
    BuiltIn,
    User,
    Project,
}

/// A discovered team with its name, absolute path, and source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamEntry {
    pub name: String,
    pub path: PathBuf,
    pub source: TeamSource,
}

/// Discover team files from multiple sources, merging them with collision detection.
///
/// - `builtin_teams_dir`: must exist; error if missing.
/// - `user_teams_dir`: silently skipped if missing (user may have no custom teams).
/// - `project_teams_dir`: if `Some` but missing on disk, returns an error (misconfiguration).
///
/// If any team name appears in more than one source, returns an error listing all conflicts.
/// Returns entries sorted by name.
pub fn discover_teams(
    builtin_teams_dir: &Path,
    user_teams_dir: &Path,
    project_teams_dir: Option<&Path>,
) -> Result<Vec<TeamEntry>> {
    // Fail fast if a configured project dir is missing.
    if let Some(proj_dir) = project_teams_dir {
        if !proj_dir.exists() {
            anyhow::bail!(
                "custom_dir project teams directory does not exist: {}",
                proj_dir.display()
            );
        }
    }

    let mut entries: Vec<TeamEntry> = Vec::new();

    // Built-in dir: required — propagate error if missing.
    let builtin_entries =
        std::fs::read_dir(builtin_teams_dir).context("Failed to read built-in teams directory")?;
    for entry in builtin_entries {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(stem) = md_stem(&entry) {
                entries.push(TeamEntry {
                    name: stem,
                    path: entry.path(),
                    source: TeamSource::BuiltIn,
                });
            }
        }
    }

    // User dir: silently skip if missing.
    if user_teams_dir.exists() {
        let user_entries = std::fs::read_dir(user_teams_dir)
            .context("Failed to read user teams directory")?;
        for entry in user_entries {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(stem) = md_stem(&entry) {
                    entries.push(TeamEntry {
                        name: stem,
                        path: entry.path(),
                        source: TeamSource::User,
                    });
                }
            }
        }
    }

    // Project dir: already verified to exist above if Some.
    if let Some(proj_dir) = project_teams_dir {
        let proj_entries =
            std::fs::read_dir(proj_dir).context("Failed to read project teams directory")?;
        for entry in proj_entries {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(stem) = md_stem(&entry) {
                    entries.push(TeamEntry {
                        name: stem,
                        path: entry.path(),
                        source: TeamSource::Project,
                    });
                }
            }
        }
    }

    // Collision detection: collect all names that appear more than once.
    let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for e in &entries {
        *seen.entry(e.name.as_str()).or_insert(0) += 1;
    }
    let mut conflicts: Vec<&str> = seen
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect();
    if !conflicts.is_empty() {
        conflicts.sort();
        anyhow::bail!(
            "Team name collision — the following team names appear in more than one source: {}",
            conflicts.join(", ")
        );
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

fn md_stem(entry: &std::fs::DirEntry) -> Option<String> {
    entry
        .file_name()
        .to_str()
        .and_then(|n| n.strip_suffix(".md"))
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests;

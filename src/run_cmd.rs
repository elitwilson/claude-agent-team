use std::path::PathBuf;

use anyhow::Result;

/// Parsed arguments for the `run` subcommand.
#[derive(Debug, PartialEq)]
pub struct RunArgs {
    pub spec: Option<String>,
    pub team: Option<String>,
    pub headless: bool,
    pub cleanup_plist: Option<PathBuf>,
    pub account: Option<String>,
    pub spec_hash: Option<String>,
    pub mode: Option<String>,
}

/// Resolve the spec file path from a slug (with or without `.md` extension).
pub fn spec_path_for_slug(slug: &str, specs_dir: &std::path::Path) -> std::path::PathBuf {
    let slug = slug.strip_suffix(".md").unwrap_or(slug);
    specs_dir.join(format!("{slug}.md"))
}

/// Parse `run` subcommand flags from an arg iterator.
///
/// Expects args AFTER the `run` keyword has been consumed, e.g.:
///   `--spec foo.md --team dev --headless --cleanup-plist /tmp/a.plist`
pub fn parse_run_args(args: &[String]) -> Result<RunArgs> {
    let mut spec: Option<String> = None;
    let mut team: Option<String> = None;
    let mut headless = false;
    let mut cleanup_plist: Option<PathBuf> = None;
    let mut account: Option<String> = None;
    let mut spec_hash: Option<String> = None;
    let mut mode: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--spec" => {
                i += 1;
                let val = args.get(i).filter(|v| !v.starts_with("--"));
                spec = Some(val.ok_or_else(|| anyhow::anyhow!("--spec requires a value"))?.clone());
            }
            "--team" => {
                i += 1;
                let val = args.get(i).filter(|v| !v.starts_with("--"));
                team = Some(val.ok_or_else(|| anyhow::anyhow!("--team requires a value"))?.clone());
            }
            "--headless" => {
                headless = true;
            }
            "--cleanup-plist" => {
                i += 1;
                let val = args.get(i).filter(|v| !v.starts_with("--"));
                cleanup_plist = Some(PathBuf::from(
                    val.ok_or_else(|| anyhow::anyhow!("--cleanup-plist requires a value"))?,
                ));
            }
            "--account" => {
                i += 1;
                let val = args.get(i).filter(|v| !v.starts_with("--"));
                account = Some(val.ok_or_else(|| anyhow::anyhow!("--account requires a value"))?.clone());
            }
            "--spec-hash" => {
                i += 1;
                let val = args.get(i).filter(|v| !v.starts_with("--"));
                spec_hash = Some(val.ok_or_else(|| anyhow::anyhow!("--spec-hash requires a value"))?.clone());
            }
            "--mode" => {
                i += 1;
                let val = args.get(i).filter(|v| !v.starts_with("--"));
                mode = Some(val.ok_or_else(|| anyhow::anyhow!("--mode requires a value"))?.clone());
            }
            other => {
                anyhow::bail!("Unknown flag: {other}");
            }
        }
        i += 1;
    }

    let is_auto_plan = mode.as_deref() == Some("auto-plan");

    if !is_auto_plan {
        if spec.is_none() {
            anyhow::bail!("--spec is required");
        }
        if team.is_none() {
            anyhow::bail!("--team is required");
        }
    }

    Ok(RunArgs {
        spec,
        team,
        headless,
        cleanup_plist,
        account,
        spec_hash,
        mode,
    })
}

#[cfg(test)]
mod tests;

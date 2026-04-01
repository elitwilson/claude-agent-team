use std::path::PathBuf;

use anyhow::Result;

/// Parsed arguments for the `run` subcommand.
#[derive(Debug, PartialEq)]
pub struct RunArgs {
    pub spec: String,
    pub team: String,
    pub headless: bool,
    pub cleanup_plist: Option<PathBuf>,
    pub account: Option<String>,
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
            other => {
                anyhow::bail!("Unknown flag: {other}");
            }
        }
        i += 1;
    }

    Ok(RunArgs {
        spec: spec.ok_or_else(|| anyhow::anyhow!("--spec is required"))?,
        team: team.ok_or_else(|| anyhow::anyhow!("--team is required"))?,
        headless,
        cleanup_plist,
        account,
    })
}

#[cfg(test)]
mod tests;

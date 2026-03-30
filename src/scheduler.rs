use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, Timelike};

/// A scheduled agent run backed by a launchd plist.
#[derive(Debug, PartialEq)]
pub struct ScheduledRun {
    pub spec: String,
    pub team: String,
    pub headless: bool,
    pub scheduled_at: DateTime<Local>,
    pub plist_path: PathBuf,
}

/// Generate the plist XML content for a scheduled run.
///
/// This is separated from `schedule_run` so it can be unit tested without
/// touching the filesystem or launchd.
pub fn generate_plist_xml(
    spec: &str,
    team: &str,
    headless: bool,
    working_dir: &Path,
    scheduled_at: DateTime<Local>,
    binary_path: &Path,
    plist_path: &Path,
) -> Result<String> {
    let label = format!("{PLIST_PREFIX}.{spec}");
    let binary = binary_path.to_str().context("Binary path is not valid UTF-8")?;
    let working = working_dir.to_str().context("Working dir is not valid UTF-8")?;
    let plist = plist_path.to_str().context("Plist path is not valid UTF-8")?;

    let mut program_args = vec![
        "        <string>/usr/bin/caffeinate</string>".to_string(),
        "        <string>-i</string>".to_string(),
        format!("        <string>{binary}</string>"),
        "        <string>run</string>".to_string(),
        "        <string>--spec</string>".to_string(),
        format!("        <string>{spec}</string>"),
        "        <string>--team</string>".to_string(),
        format!("        <string>{team}</string>"),
    ];
    if headless {
        program_args.push("        <string>--headless</string>".to_string());
    }
    program_args.push("        <string>--cleanup-plist</string>".to_string());
    program_args.push(format!("        <string>{plist}</string>"));

    let args_xml = program_args.join("\n");

    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
{args_xml}
    </array>
    <key>WorkingDirectory</key>
    <string>{working}</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Month</key>
        <integer>{month}</integer>
        <key>Day</key>
        <integer>{day}</integer>
        <key>Hour</key>
        <integer>{hour}</integer>
        <key>Minute</key>
        <integer>{minute}</integer>
    </dict>
</dict>
</plist>
"#,
        month = scheduled_at.month(),
        day = scheduled_at.day(),
        hour = scheduled_at.hour(),
        minute = scheduled_at.minute(),
    );

    Ok(xml)
}

/// Build the plist file path for a given spec slug.
pub fn plist_path_for_spec(spec: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let filename = format!("{PLIST_PREFIX}.{spec}.plist");
    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(filename))
}

/// Validate that the scheduled time is at least 1 minute in the future.
pub fn validate_schedule_time(scheduled_at: DateTime<Local>) -> Result<()> {
    let now = Local::now();
    let diff = scheduled_at - now;
    if diff.num_seconds() < 60 {
        anyhow::bail!(
            "Scheduled time must be at least 1 minute in the future (got {}s from now)",
            diff.num_seconds()
        );
    }
    Ok(())
}

/// Schedule a run by writing a plist and loading it with launchctl.
pub fn schedule_run(
    spec: &str,
    team: &str,
    headless: bool,
    working_dir: &Path,
    scheduled_at: DateTime<Local>,
) -> Result<ScheduledRun> {
    validate_schedule_time(scheduled_at)?;

    let plist_path = plist_path_for_spec(spec)?;
    let binary_path = std::env::current_exe().context("Failed to resolve binary path")?;

    let xml = generate_plist_xml(spec, team, headless, working_dir, scheduled_at, &binary_path, &plist_path)?;

    std::fs::write(&plist_path, &xml)
        .with_context(|| format!("Failed to write plist to {}", plist_path.display()))?;

    let status = Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .status()
        .context("Failed to run launchctl load")?;

    if !status.success() {
        // Clean up the written file if launchctl load fails
        let _ = std::fs::remove_file(&plist_path);
        anyhow::bail!("launchctl load failed with exit code {:?}", status.code());
    }

    Ok(ScheduledRun {
        spec: spec.to_string(),
        team: team.to_string(),
        headless,
        scheduled_at,
        plist_path,
    })
}

/// List all pending scheduled runs by scanning ~/Library/LaunchAgents/.
pub fn list_pending() -> Result<Vec<ScheduledRun>> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;
    let launch_agents = PathBuf::from(home).join("Library").join("LaunchAgents");
    list_pending_in(&launch_agents)
}

/// List pending scheduled runs in a given directory. Testable variant of `list_pending`.
pub fn list_pending_in(_dir: &Path) -> Result<Vec<ScheduledRun>> {
    todo!()
}

/// Parse a plist file into a ScheduledRun.
pub fn parse_plist(_path: &Path) -> Result<ScheduledRun> {
    todo!()
}

/// Clean up a plist after a scheduled run completes.
///
/// Calls `launchctl unload <path>` then removes the file. Both steps are
/// fatal — a missed cleanup turns a one-shot run into a recurring annual job.
pub fn cleanup_plist(_path: &Path) -> Result<()> {
    todo!()
}

/// The plist label prefix used by this tool.
pub const PLIST_PREFIX: &str = "com.claude-agent-team";

#[cfg(test)]
mod tests;

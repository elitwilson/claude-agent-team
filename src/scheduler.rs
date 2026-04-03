use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};

/// A scheduled agent run backed by a launchd plist.
#[derive(Debug, PartialEq)]
pub struct ScheduledRun {
    pub spec: String,
    pub team: String,
    pub headless: bool,
    pub scheduled_at: DateTime<Local>,
    pub plist_path: PathBuf,
    pub account: Option<String>,
    pub spec_hash: Option<String>,
}

/// Compute a SHA-256 hash of a spec file's raw bytes, returning a lowercase hex string.
pub fn hash_spec_file(path: &Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read spec file for hashing: {}", path.display()))?;
    let hash = Sha256::digest(&bytes);
    Ok(format!("{hash:x}"))
}

/// Generate the plist XML content for a scheduled run.
///
/// This is separated from `schedule_run` so it can be unit tested without
/// touching the filesystem or launchd.
pub fn generate_plist_xml(
    spec: &str,
    team: &str,
    headless: bool,
    account: Option<&str>,
    spec_hash: Option<&str>,
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
    if let Some(label) = account {
        program_args.push("        <string>--account</string>".to_string());
        program_args.push(format!("        <string>{label}</string>"));
    }
    if let Some(hash) = spec_hash {
        program_args.push("        <string>--spec-hash</string>".to_string());
        program_args.push(format!("        <string>{hash}</string>"));
    }
    program_args.push("        <string>--cleanup-plist</string>".to_string());
    program_args.push(format!("        <string>{plist}</string>"));

    let args_xml = program_args.join("\n");

    let log_dir = format!("{working}/logs/agent-runs");
    let stdout_path = format!("{log_dir}/{spec}-launchd.log");
    let stderr_path = format!("{log_dir}/{spec}-launchd.err");

    // Capture current PATH so launchd (which runs with a minimal environment)
    // can find binaries like `claude` that live in user-specific directories.
    let path_env = std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string());

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
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>{path_env}</string>
    </dict>
    <key>WorkingDirectory</key>
    <string>{working}</string>
    <key>StandardOutPath</key>
    <string>{stdout_path}</string>
    <key>StandardErrorPath</key>
    <string>{stderr_path}</string>
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

    // Unload and remove any existing registration for this spec before re-scheduling.
    if plist_path.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", plist_path.to_str().unwrap_or("")])
            .status();
        let _ = std::fs::remove_file(&plist_path);
    }

    let binary_path = std::env::current_exe().context("Failed to resolve binary path")?;

    // Compute spec hash for scheduled-run integrity check
    let spec_hash = hash_spec_file(&working_dir.join("docs/specs").join(format!("{spec}.md")))
        .or_else(|_| hash_spec_file(&working_dir.join("docs/specs").join(spec)))
        .ok();

    let xml = generate_plist_xml(spec, team, headless, None, spec_hash.as_deref(), working_dir, scheduled_at, &binary_path, &plist_path)?;

    std::fs::write(&plist_path, &xml)
        .with_context(|| format!("Failed to write plist to {}", plist_path.display()))?;

    let status = Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .status()
        .context("Failed to run launchctl load")?;

    if !status.success() {
        let _ = std::fs::remove_file(&plist_path);
        anyhow::bail!("launchctl load failed with exit code {:?}", status.code());
    }

    Ok(ScheduledRun {
        spec: spec.to_string(),
        team: team.to_string(),
        headless,
        scheduled_at,
        plist_path,
        account: None,
        spec_hash,
    })
}

/// List all pending scheduled runs by scanning ~/Library/LaunchAgents/.
pub fn list_pending() -> Result<Vec<ScheduledRun>> {
    let home = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME not set"))?;
    let launch_agents = PathBuf::from(home).join("Library").join("LaunchAgents");
    list_pending_in(&launch_agents)
}

/// List pending scheduled runs in a given directory. Testable variant of `list_pending`.
pub fn list_pending_in(dir: &Path) -> Result<Vec<ScheduledRun>> {
    let mut runs = Vec::new();

    if !dir.exists() {
        return Ok(runs);
    }

    for entry in std::fs::read_dir(dir).context("Failed to read LaunchAgents directory")? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(PLIST_PREFIX) && name.ends_with(".plist") {
            match parse_plist(&entry.path()) {
                Ok(run) => runs.push(run),
                Err(_) => continue, // skip malformed plists
            }
        }
    }

    Ok(runs)
}

/// Parse a plist file into a ScheduledRun.
pub fn parse_plist(path: &Path) -> Result<ScheduledRun> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read plist: {}", path.display()))?;

    // Extract ProgramArguments strings
    let args = extract_program_arguments(&content)?;

    // Parse spec, team, headless, account, spec_hash from ProgramArguments
    let mut spec: Option<String> = None;
    let mut team: Option<String> = None;
    let mut headless = false;
    let mut account: Option<String> = None;
    let mut spec_hash: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--spec" => {
                i += 1;
                spec = args.get(i).cloned();
            }
            "--team" => {
                i += 1;
                team = args.get(i).cloned();
            }
            "--headless" => {
                headless = true;
            }
            "--account" => {
                i += 1;
                account = args.get(i).cloned();
            }
            "--spec-hash" => {
                i += 1;
                spec_hash = args.get(i).cloned();
            }
            _ => {}
        }
        i += 1;
    }

    let spec = spec.context("No --spec found in ProgramArguments")?;
    let team = team.context("No --team found in ProgramArguments")?;

    // Parse StartCalendarInterval
    let (month, day, hour, minute) = extract_calendar_interval(&content)?;

    let year = Local::now().year();
    let scheduled_at = Local
        .with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .context("Invalid calendar interval date/time")?;

    Ok(ScheduledRun {
        spec,
        team,
        headless,
        scheduled_at,
        plist_path: path.to_path_buf(),
        account,
        spec_hash,
    })
}

/// Extract <string> values from the ProgramArguments <array> in plist XML.
fn extract_program_arguments(xml: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut in_array = false;

    for line in xml.lines() {
        let trimmed = line.trim();
        if trimmed == "<key>ProgramArguments</key>" {
            // Next non-whitespace should be <array>
            continue;
        }
        if trimmed == "<array>" && !in_array {
            in_array = true;
            continue;
        }
        if trimmed == "</array>" && in_array {
            break;
        }
        if in_array {
            if let Some(val) = extract_string_value(trimmed) {
                args.push(val);
            }
        }
    }

    Ok(args)
}

/// Extract Month, Day, Hour, Minute from StartCalendarInterval dict.
fn extract_calendar_interval(xml: &str) -> Result<(u32, u32, u32, u32)> {
    let mut month: Option<u32> = None;
    let mut day: Option<u32> = None;
    let mut hour: Option<u32> = None;
    let mut minute: Option<u32> = None;

    let mut in_interval = false;
    let mut current_key: Option<String> = None;

    for line in xml.lines() {
        let trimmed = line.trim();
        if trimmed == "<key>StartCalendarInterval</key>" {
            in_interval = true;
            continue;
        }
        if !in_interval {
            continue;
        }
        // End of the interval dict
        if trimmed == "</dict>" {
            break;
        }
        if trimmed == "<dict>" {
            continue;
        }
        if let Some(key) = extract_key_value(trimmed) {
            current_key = Some(key);
        } else if let Some(val) = extract_integer_value(trimmed) {
            if let Some(ref key) = current_key {
                match key.as_str() {
                    "Month" => month = Some(val),
                    "Day" => day = Some(val),
                    "Hour" => hour = Some(val),
                    "Minute" => minute = Some(val),
                    _ => {}
                }
                current_key = None;
            }
        }
    }

    Ok((
        month.context("Missing Month in StartCalendarInterval")?,
        day.context("Missing Day in StartCalendarInterval")?,
        hour.context("Missing Hour in StartCalendarInterval")?,
        minute.context("Missing Minute in StartCalendarInterval")?,
    ))
}

/// Extract value from `<string>VALUE</string>`.
fn extract_string_value(s: &str) -> Option<String> {
    s.strip_prefix("<string>")
        .and_then(|rest| rest.strip_suffix("</string>"))
        .map(|v| v.to_string())
}

/// Extract value from `<key>VALUE</key>`.
fn extract_key_value(s: &str) -> Option<String> {
    s.strip_prefix("<key>")
        .and_then(|rest| rest.strip_suffix("</key>"))
        .map(|v| v.to_string())
}

/// Extract value from `<integer>VALUE</integer>`.
fn extract_integer_value(s: &str) -> Option<u32> {
    s.strip_prefix("<integer>")
        .and_then(|rest| rest.strip_suffix("</integer>"))
        .and_then(|v| v.parse().ok())
}

/// Clean up a plist after a scheduled run completes.
///
/// Calls `launchctl unload <path>` then removes the file. Both steps are
/// fatal — a missed cleanup turns a one-shot run into a recurring annual job.
pub fn cleanup_plist(path: &Path) -> Result<()> {
    let path_str = path.to_str().context("Plist path is not valid UTF-8")?;

    let output = Command::new("launchctl")
        .args(["unload", path_str])
        .output()
        .context("Failed to run launchctl unload")?;

    if !output.status.success() {
        anyhow::bail!(
            "launchctl unload failed for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    std::fs::remove_file(path)
        .with_context(|| format!("Failed to remove plist file: {}", path.display()))?;

    Ok(())
}

/// The plist label prefix used by this tool.
pub const PLIST_PREFIX: &str = "com.claude-launch";

#[cfg(test)]
mod tests;

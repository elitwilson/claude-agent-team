use super::*;
use chrono::{Datelike, TimeZone, Timelike};
use std::path::PathBuf;
use std::io::Write;

// --- plist_path_for_spec ---

#[test]
fn test_plist_path_uses_launch_agents_dir() {
    let path = plist_path_for_spec("my-feature").unwrap();
    let home = std::env::var("HOME").unwrap();
    let expected = PathBuf::from(format!(
        "{}/Library/LaunchAgents/com.claude-agent-team.my-feature.plist",
        home
    ));
    assert_eq!(path, expected);
}

#[test]
fn test_plist_path_slug_with_dots() {
    let path = plist_path_for_spec("005-scheduled-runs.md").unwrap();
    let home = std::env::var("HOME").unwrap();
    assert!(path.to_str().unwrap().contains("com.claude-agent-team.005-scheduled-runs.md.plist"));
    assert!(path.starts_with(format!("{}/Library/LaunchAgents", home)));
}

// --- validate_schedule_time ---

#[test]
fn test_validate_schedule_time_rejects_past() {
    let past = Local::now() - chrono::Duration::minutes(5);
    assert!(validate_schedule_time(past).is_err());
}

#[test]
fn test_validate_schedule_time_rejects_less_than_one_minute() {
    let soon = Local::now() + chrono::Duration::seconds(30);
    assert!(validate_schedule_time(soon).is_err());
}

#[test]
fn test_validate_schedule_time_accepts_future() {
    let future = Local::now() + chrono::Duration::minutes(5);
    assert!(validate_schedule_time(future).is_ok());
}

// --- generate_plist_xml ---

#[test]
fn test_plist_contains_label() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("com.claude-agent-team.my-feature"));
}

#[test]
fn test_plist_wraps_in_caffeinate() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        true,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    // caffeinate -i must be the first program argument
    assert!(xml.contains("caffeinate"));
    assert!(xml.contains("-i"));
}

#[test]
fn test_plist_includes_run_args() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        true,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("run"));
    assert!(xml.contains("--spec"));
    assert!(xml.contains("my-feature"));
    assert!(xml.contains("--team"));
    assert!(xml.contains("dev-team"));
    assert!(xml.contains("--headless"));
}

#[test]
fn test_plist_excludes_headless_when_false() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(!xml.contains("--headless"));
}

#[test]
fn test_plist_includes_cleanup_plist_flag() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let plist_path = Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist");
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        plist_path,
    )
    .unwrap();
    assert!(xml.contains("--cleanup-plist"));
    assert!(xml.contains(plist_path.to_str().unwrap()));
}

#[test]
fn test_plist_includes_working_directory() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("WorkingDirectory"));
    assert!(xml.contains("/Users/test/project"));
}

#[test]
fn test_plist_includes_calendar_interval() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("StartCalendarInterval"));
    // Month=4, Day=15, Hour=14, Minute=30
    assert!(xml.contains("Month"));
    assert!(xml.contains("Day"));
    assert!(xml.contains("Hour"));
    assert!(xml.contains("Minute"));
}

#[test]
fn test_plist_includes_binary_path() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("/usr/local/bin/claude-bros"));
}

#[test]
fn test_plist_is_valid_xml() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("<?xml version="));
    assert!(xml.contains("<!DOCTYPE plist"));
    assert!(xml.contains("<plist version="));
    assert!(xml.contains("</plist>"));
}

// --- list_pending_in / parse_plist ---

const FIXTURE_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.claude-agent-team.my-feature</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/caffeinate</string>
        <string>-i</string>
        <string>/usr/local/bin/claude-bros</string>
        <string>run</string>
        <string>--spec</string>
        <string>my-feature</string>
        <string>--team</string>
        <string>dev-team</string>
        <string>--headless</string>
        <string>--cleanup-plist</string>
        <string>/tmp/com.claude-agent-team.my-feature.plist</string>
    </array>
    <key>WorkingDirectory</key>
    <string>/Users/test/project</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Month</key>
        <integer>4</integer>
        <key>Day</key>
        <integer>15</integer>
        <key>Hour</key>
        <integer>14</integer>
        <key>Minute</key>
        <integer>30</integer>
    </dict>
</dict>
</plist>
"#;

#[test]
fn test_parse_plist_extracts_spec() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert_eq!(run.spec, "my-feature");
}

#[test]
fn test_parse_plist_extracts_team() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert_eq!(run.team, "dev-team");
}

#[test]
fn test_parse_plist_extracts_headless() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert!(run.headless);
}

#[test]
fn test_parse_plist_extracts_scheduled_at() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert_eq!(run.scheduled_at.month(), 4);
    assert_eq!(run.scheduled_at.day(), 15);
    assert_eq!(run.scheduled_at.hour(), 14);
    assert_eq!(run.scheduled_at.minute(), 30);
}

#[test]
fn test_parse_plist_stores_plist_path() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert_eq!(run.plist_path, plist_file);
}

#[test]
fn test_list_pending_in_empty_dir() {
    let dir = tempfile::tempdir().unwrap();
    let runs = list_pending_in(dir.path()).unwrap();
    assert!(runs.is_empty());
}

#[test]
fn test_list_pending_in_finds_matching_plists() {
    let dir = tempfile::tempdir().unwrap();

    // Write a matching plist
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let runs = list_pending_in(dir.path()).unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].spec, "my-feature");
}

#[test]
fn test_list_pending_in_ignores_non_matching_files() {
    let dir = tempfile::tempdir().unwrap();

    // Write a matching plist
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    // Write a non-matching file
    let other_file = dir.path().join("com.other-tool.something.plist");
    std::fs::write(&other_file, "not our plist").unwrap();

    let runs = list_pending_in(dir.path()).unwrap();
    assert_eq!(runs.len(), 1);
}

// --- generate_plist_xml: account encoding ---

#[test]
fn test_plist_includes_account_flag_when_provided() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        Some("work"),
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(xml.contains("--account"));
    assert!(xml.contains("work"));
}

#[test]
fn test_plist_excludes_account_flag_when_none() {
    let scheduled_at = Local.with_ymd_and_hms(2026, 4, 15, 14, 30, 0).unwrap();
    let xml = generate_plist_xml(
        "my-feature",
        "dev-team",
        false,
        None,
        Path::new("/Users/test/project"),
        scheduled_at,
        Path::new("/usr/local/bin/claude-bros"),
        Path::new("/Users/test/Library/LaunchAgents/com.claude-agent-team.my-feature.plist"),
    )
    .unwrap();
    assert!(!xml.contains("--account"));
}

// --- parse_plist: --account extraction ---

const FIXTURE_PLIST_WITH_ACCOUNT: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.claude-agent-team.my-feature</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/bin/caffeinate</string>
        <string>-i</string>
        <string>/usr/local/bin/claude-bros</string>
        <string>run</string>
        <string>--spec</string>
        <string>my-feature</string>
        <string>--team</string>
        <string>dev-team</string>
        <string>--account</string>
        <string>work</string>
        <string>--cleanup-plist</string>
        <string>/tmp/com.claude-agent-team.my-feature.plist</string>
    </array>
    <key>WorkingDirectory</key>
    <string>/Users/test/project</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Month</key>
        <integer>4</integer>
        <key>Day</key>
        <integer>15</integer>
        <key>Hour</key>
        <integer>14</integer>
        <key>Minute</key>
        <integer>30</integer>
    </dict>
</dict>
</plist>
"#;

#[test]
fn test_parse_plist_extracts_account_when_present() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST_WITH_ACCOUNT.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert_eq!(run.account, Some("work".to_string()));
}

#[test]
fn test_parse_plist_account_is_none_for_legacy_plist_without_account() {
    // FIXTURE_PLIST has no --account arg — backward compat: account = None
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.my-feature.plist");
    let mut f = std::fs::File::create(&plist_file).unwrap();
    f.write_all(FIXTURE_PLIST.as_bytes()).unwrap();

    let run = parse_plist(&plist_file).unwrap();
    assert!(run.account.is_none());
}

// --- cleanup_plist ---

#[test]
fn test_cleanup_plist_removes_file() {
    let dir = tempfile::tempdir().unwrap();
    let plist_file = dir.path().join("com.claude-agent-team.test.plist");
    std::fs::write(&plist_file, "dummy").unwrap();
    assert!(plist_file.exists());

    // launchctl unload exits 0 silently for plists that aren't registered,
    // so cleanup_plist succeeds and removes the file.
    let result = cleanup_plist(&plist_file);
    assert!(result.is_ok());
    assert!(!plist_file.exists());
}

#[test]
fn test_cleanup_plist_errors_on_nonexistent_file() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does-not-exist.plist");
    let result = cleanup_plist(&missing);
    assert!(result.is_err());
}

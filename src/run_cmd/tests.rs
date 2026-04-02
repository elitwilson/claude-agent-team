use super::*;
use std::path::PathBuf;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

// --- parse_run_args: required flags ---

#[test]
fn test_parse_all_flags() {
    let input = args(&[
        "--spec", "005-scheduled-runs.md",
        "--team", "feature-dev",
        "--headless",
        "--cleanup-plist", "/tmp/com.claude-launch.test.plist",
    ]);
    let result = parse_run_args(&input).unwrap();
    assert_eq!(result, RunArgs {
        spec: "005-scheduled-runs.md".to_string(),
        team: "feature-dev".to_string(),
        headless: true,
        cleanup_plist: Some(PathBuf::from("/tmp/com.claude-launch.test.plist")),
        account: None,
    });
}

#[test]
fn test_parse_required_flags_only() {
    let input = args(&[
        "--spec", "my-spec.md",
        "--team", "my-team",
    ]);
    let result = parse_run_args(&input).unwrap();
    assert_eq!(result, RunArgs {
        spec: "my-spec.md".to_string(),
        team: "my-team".to_string(),
        headless: false,
        cleanup_plist: None,
        account: None,
    });
}

#[test]
fn test_parse_flags_in_any_order() {
    let input = args(&[
        "--headless",
        "--team", "dev",
        "--cleanup-plist", "/tmp/test.plist",
        "--spec", "foo.md",
    ]);
    let result = parse_run_args(&input).unwrap();
    assert_eq!(result, RunArgs {
        spec: "foo.md".to_string(),
        team: "dev".to_string(),
        headless: true,
        cleanup_plist: Some(PathBuf::from("/tmp/test.plist")),
        account: None,
    });
}

// --- parse_run_args: missing required flags ---

#[test]
fn test_missing_spec_returns_error() {
    let input = args(&["--team", "dev"]);
    assert!(parse_run_args(&input).is_err());
}

#[test]
fn test_missing_team_returns_error() {
    let input = args(&["--spec", "foo.md"]);
    assert!(parse_run_args(&input).is_err());
}

#[test]
fn test_empty_args_returns_error() {
    let input = args(&[]);
    assert!(parse_run_args(&input).is_err());
}

// --- parse_run_args: edge cases ---

#[test]
fn test_unknown_flag_returns_error() {
    let input = args(&[
        "--spec", "foo.md",
        "--team", "dev",
        "--unknown", "value",
    ]);
    assert!(parse_run_args(&input).is_err());
}

#[test]
fn test_spec_flag_missing_value_returns_error() {
    let input = args(&["--spec", "--team", "dev"]);
    assert!(parse_run_args(&input).is_err());
}

// --- parse_run_args: --account flag ---

#[test]
fn test_parse_account_flag() {
    let input = args(&[
        "--spec", "foo.md",
        "--team", "dev",
        "--account", "work",
    ]);
    let result = parse_run_args(&input).unwrap();
    assert_eq!(result.account, Some("work".to_string()));
}

#[test]
fn test_parse_account_flag_with_all_flags() {
    let input = args(&[
        "--spec", "foo.md",
        "--team", "dev",
        "--headless",
        "--account", "personal",
        "--cleanup-plist", "/tmp/test.plist",
    ]);
    let result = parse_run_args(&input).unwrap();
    assert_eq!(result, RunArgs {
        spec: "foo.md".to_string(),
        team: "dev".to_string(),
        headless: true,
        cleanup_plist: Some(PathBuf::from("/tmp/test.plist")),
        account: Some("personal".to_string()),
    });
}

#[test]
fn test_account_flag_defaults_to_none() {
    let input = args(&["--spec", "foo.md", "--team", "dev"]);
    let result = parse_run_args(&input).unwrap();
    assert!(result.account.is_none());
}

#[test]
fn test_account_flag_missing_value_returns_error() {
    let input = args(&["--spec", "foo.md", "--team", "dev", "--account"]);
    assert!(parse_run_args(&input).is_err());
}

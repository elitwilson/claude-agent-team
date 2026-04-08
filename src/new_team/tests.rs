use super::*;
use std::fs;
use tempfile::TempDir;

fn make_temp() -> TempDir {
    TempDir::new().expect("failed to create temp dir")
}

// --- validate_name tests ---

#[test]
fn test_validate_name_accepts_lowercase_letters() {
    assert!(validate_name("myteam").is_ok());
}

#[test]
fn test_validate_name_accepts_hyphens() {
    assert!(validate_name("my-team").is_ok());
}

#[test]
fn test_validate_name_accepts_digits() {
    assert!(validate_name("team1").is_ok());
}

#[test]
fn test_validate_name_accepts_mixed_lowercase_digits_hyphens() {
    assert!(validate_name("my-team-2").is_ok());
}

#[test]
fn test_validate_name_rejects_uppercase() {
    let result = validate_name("MyTeam");
    assert!(result.is_err(), "expected error for uppercase name");
}

#[test]
fn test_validate_name_rejects_spaces() {
    let result = validate_name("my team");
    assert!(result.is_err(), "expected error for name with space");
}

#[test]
fn test_validate_name_rejects_underscores() {
    let result = validate_name("my_team");
    assert!(result.is_err(), "expected error for underscore");
}

#[test]
fn test_validate_name_rejects_special_chars() {
    let result = validate_name("foo!bar");
    assert!(result.is_err(), "expected error for special char");
}

#[test]
fn test_validate_name_rejects_empty_string() {
    let result = validate_name("");
    assert!(result.is_err(), "expected error for empty name");
}

// --- resolve_target_root tests ---

#[test]
fn test_resolve_target_root_user_level_returns_user_dir() {
    let cwd = make_temp();
    let root = resolve_target_root("user", "/fake/workflow", None, cwd.path()).unwrap();
    assert_eq!(root.to_string_lossy(), "/fake/workflow/user");
}

#[test]
fn test_resolve_target_root_project_level_with_custom_dir() {
    let cwd = make_temp();
    let root =
        resolve_target_root("project", "/fake/workflow", Some("my-custom"), cwd.path()).unwrap();
    assert_eq!(root, cwd.path().join("my-custom"));
}

#[test]
fn test_resolve_target_root_project_level_no_custom_dir_errors() {
    let cwd = make_temp();
    let result = resolve_target_root("project", "/fake/workflow", None, cwd.path());
    assert!(result.is_err(), "expected error when custom_dir is missing for project level");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("custom_dir"),
        "error should mention custom_dir, got: {msg}"
    );
    assert!(
        msg.contains(".claude-launch.toml"),
        "error should mention .claude-launch.toml, got: {msg}"
    );
}

// --- scaffold_team tests ---

#[test]
fn test_scaffold_team_creates_team_file_with_noop_content() {
    let root = make_temp();
    let (team_path, _) = scaffold_team("my-team", root.path()).unwrap();
    let content = fs::read_to_string(&team_path).unwrap();
    assert!(
        content.contains("scaffolded team prompt"),
        "team file should contain no-op message, got: {content}"
    );
    assert!(
        content.contains("Replace this file"),
        "team file should tell user to replace it, got: {content}"
    );
}

#[test]
fn test_scaffold_team_creates_agent_file_with_noop_content() {
    let root = make_temp();
    let (_, agent_path) = scaffold_team("my-team", root.path()).unwrap();
    let content = fs::read_to_string(&agent_path).unwrap();
    assert!(
        content.contains("scaffolded agent definition"),
        "agent file should contain no-op message, got: {content}"
    );
    assert!(
        content.contains("Replace this file"),
        "agent file should tell user to replace it, got: {content}"
    );
}

#[test]
fn test_scaffold_team_creates_files_at_correct_paths() {
    let root = make_temp();
    let (team_path, agent_path) = scaffold_team("my-team", root.path()).unwrap();
    assert_eq!(team_path, root.path().join("teams").join("my-team.md"));
    assert_eq!(
        agent_path,
        root.path().join("agents").join("my-team").join("agent.md")
    );
}

#[test]
fn test_scaffold_team_creates_intermediate_directories() {
    let root = make_temp();
    let (team_path, agent_path) = scaffold_team("alpha", root.path()).unwrap();
    assert!(team_path.exists(), "teams/alpha.md should be created");
    assert!(agent_path.exists(), "agents/alpha/agent.md should be created");
}

#[test]
fn test_scaffold_team_fails_if_team_file_already_exists() {
    let root = make_temp();
    let teams_dir = root.path().join("teams");
    fs::create_dir_all(&teams_dir).unwrap();
    fs::write(teams_dir.join("existing.md"), "already here").unwrap();

    let result = scaffold_team("existing", root.path());
    assert!(result.is_err(), "should fail when team file already exists");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("already exists"),
        "error should say 'already exists', got: {msg}"
    );
}

#[test]
fn test_scaffold_team_fails_if_agent_file_already_exists() {
    let root = make_temp();
    let agent_dir = root.path().join("agents").join("existing");
    fs::create_dir_all(&agent_dir).unwrap();
    fs::write(agent_dir.join("agent.md"), "already here").unwrap();

    let result = scaffold_team("existing", root.path());
    assert!(result.is_err(), "should fail when agent file already exists");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("already exists"),
        "error should say 'already exists', got: {msg}"
    );
}

#[test]
fn test_scaffold_team_no_partial_writes_when_agent_exists() {
    // If agent file exists, team file must NOT be created (no partial writes)
    let root = make_temp();
    let agent_dir = root.path().join("agents").join("beta");
    fs::create_dir_all(&agent_dir).unwrap();
    fs::write(agent_dir.join("agent.md"), "already here").unwrap();

    let result = scaffold_team("beta", root.path());
    assert!(result.is_err());
    // The team file should not have been written
    assert!(
        !root.path().join("teams").join("beta.md").exists(),
        "team file should not be created when scaffolding fails"
    );
}

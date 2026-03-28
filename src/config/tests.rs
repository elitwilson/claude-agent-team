use super::*;
use std::fs;
use tempfile::TempDir;

fn create_temp_dir() -> TempDir {
    TempDir::new().expect("failed to create temp dir")
}

// --- Config loading tests ---

#[test]
fn test_load_returns_defaults_when_no_config_file() {
    let dir = create_temp_dir();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "docs/specs");
    assert_eq!(config.default_team, "feature-dev");
    assert_eq!(config.base_branch, "main");
}

#[test]
fn test_load_parses_toml_with_all_fields() {
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "custom/specs"
default_team = "my-team"
base_branch = "develop"
"#;
    fs::write(dir.path().join(".claude-agent-team.toml"), toml_content).unwrap();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "custom/specs");
    assert_eq!(config.default_team, "my-team");
    assert_eq!(config.base_branch, "develop");
}

#[test]
fn test_load_uses_defaults_for_missing_fields() {
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "other/specs"
"#;
    fs::write(dir.path().join(".claude-agent-team.toml"), toml_content).unwrap();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "other/specs");
    assert_eq!(config.default_team, "feature-dev");
    assert_eq!(config.base_branch, "main");
}

#[test]
fn test_load_ignores_unknown_keys() {
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "docs/specs"
unknown_key = "should be ignored"
another_unknown = 42
"#;
    fs::write(dir.path().join(".claude-agent-team.toml"), toml_content).unwrap();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "docs/specs");
}

// --- Spec discovery tests ---

#[test]
fn test_discover_specs_returns_md_files_only() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("feature-a.md"), "# Feature A").unwrap();
    fs::write(dir.path().join("feature-b.md"), "# Feature B").unwrap();
    fs::write(dir.path().join("notes.txt"), "not a spec").unwrap();
    fs::write(dir.path().join("readme.rst"), "not a spec").unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 2);
    assert!(specs.contains(&"feature-a.md".to_string()));
    assert!(specs.contains(&"feature-b.md".to_string()));
}

#[test]
fn test_discover_specs_skips_subdirectories() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("top-level.md"), "# Top").unwrap();
    let sub = dir.path().join("subdir");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("nested.md"), "# Nested").unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0], "top-level.md");
}

#[test]
fn test_discover_specs_returns_empty_for_empty_dir() {
    let dir = create_temp_dir();
    let specs = discover_specs(dir.path()).unwrap();
    assert!(specs.is_empty());
}

#[test]
fn test_discover_specs_errors_on_nonexistent_dir() {
    let result = discover_specs(Path::new("/nonexistent/path/surely"));
    assert!(result.is_err());
}

// --- Team discovery tests ---

#[test]
fn test_discover_teams_returns_names_without_extension() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("feature-dev.md"), "# Team").unwrap();
    fs::write(dir.path().join("review-only.md"), "# Team").unwrap();

    let teams = discover_teams(dir.path()).unwrap();
    assert_eq!(teams.len(), 2);
    assert!(teams.contains(&"feature-dev".to_string()));
    assert!(teams.contains(&"review-only".to_string()));
}

#[test]
fn test_discover_teams_skips_non_md_files() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("feature-dev.md"), "# Team").unwrap();
    fs::write(dir.path().join("notes.txt"), "not a team").unwrap();

    let teams = discover_teams(dir.path()).unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0], "feature-dev");
}

#[test]
fn test_discover_teams_returns_empty_for_empty_dir() {
    let dir = create_temp_dir();
    let teams = discover_teams(dir.path()).unwrap();
    assert!(teams.is_empty());
}

#[test]
fn test_discover_teams_errors_on_nonexistent_dir() {
    let result = discover_teams(Path::new("/nonexistent/path/surely"));
    assert!(result.is_err());
}

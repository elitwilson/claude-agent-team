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
fn test_discover_specs_includes_all_readable_text_files() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("feature-a.md"), "# Feature A").unwrap();
    fs::write(dir.path().join("feature-b.md"), "# Feature B").unwrap();
    fs::write(dir.path().join("notes.txt"), "not a spec").unwrap();
    fs::write(dir.path().join("readme.rst"), "not a spec").unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 4);
    let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"feature-a.md"));
    assert!(names.contains(&"feature-b.md"));
    assert!(names.contains(&"notes.txt"));
    assert!(names.contains(&"readme.rst"));
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
    assert_eq!(specs[0].name, "top-level.md");
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

// --- Frontmatter parsing tests ---

#[test]
fn test_parse_frontmatter_status_ready() {
    let content = "---\nstatus: ready\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Ready);
}

#[test]
fn test_parse_frontmatter_status_complete() {
    let content = "---\nstatus: complete\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Complete);
}

#[test]
fn test_parse_frontmatter_status_needs_attention_maps_to_blocked() {
    // needs_attention is a legacy alias — treated as Blocked for backwards compatibility
    let content = "---\nstatus: needs_attention\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Blocked);
}

#[test]
fn test_parse_frontmatter_status_missing_frontmatter() {
    let content = "# My Spec\n\nNo frontmatter here.";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Raw);
}

#[test]
fn test_parse_frontmatter_status_unrecognized_value() {
    let content = "---\nstatus: banana\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Ready);
}

#[test]
fn test_parse_frontmatter_status_missing_status_field() {
    let content = "---\nnumber: 4\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Ready);
}

#[test]
fn test_parse_frontmatter_status_empty_frontmatter() {
    let content = "---\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Ready);
}

// --- Spec discovery with status filtering tests ---

#[test]
fn test_discover_specs_includes_complete() {
    // Complete specs are no longer filtered at discovery — filtering is the TUI's job
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-done.md"),
        "---\nstatus: complete\n---\n# Done",
    )
    .unwrap();
    fs::write(
        dir.path().join("002-active.md"),
        "---\nstatus: ready\n---\n# Active",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 2);
    let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"001-done.md"));
    assert!(names.contains(&"002-active.md"));
}

#[test]
fn test_discover_specs_includes_needs_attention_as_blocked() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-broken.md"),
        "---\nstatus: needs_attention\n---\n# Broken",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "001-broken.md");
    assert_eq!(specs[0].status, SpecStatus::Blocked);
}

#[test]
fn test_discover_specs_treats_no_frontmatter_as_raw() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("no-front.md"), "# No Frontmatter").unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "no-front.md");
    assert_eq!(specs[0].status, SpecStatus::Raw);
}

// --- Blocked status tests ---

#[test]
fn test_parse_frontmatter_status_blocked() {
    let content = "---\nstatus: blocked\n---\n# My Spec";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Blocked);
}

#[test]
fn test_discover_specs_includes_blocked() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-blocked.md"),
        "---\nstatus: blocked\n---\n# Blocked",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].status, SpecStatus::Blocked);
}

// --- Raw requirements file tests ---

#[test]
fn test_parse_frontmatter_status_no_frontmatter_returns_raw() {
    let content = "# My Spec\n\nNo frontmatter here.";
    assert_eq!(parse_frontmatter_status(content), SpecStatus::Raw);
}

#[test]
fn test_discover_specs_includes_txt_file_as_raw() {
    let dir = create_temp_dir();
    fs::write(dir.path().join("email.txt"), "Can we add a date filter?").unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "email.txt");
    assert_eq!(specs[0].status, SpecStatus::Raw);
}

#[test]
fn test_discover_specs_includes_md_without_frontmatter_as_raw() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("notes.md"),
        "# Rough notes\n\nAdd filtering.",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "notes.md");
    assert_eq!(specs[0].status, SpecStatus::Raw);
}

#[test]
fn test_discover_specs_skips_binary_files() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("image.png"),
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
    )
    .unwrap();
    fs::write(
        dir.path().join("spec.md"),
        "---\nstatus: ready\n---\n# Real spec",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "spec.md");
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

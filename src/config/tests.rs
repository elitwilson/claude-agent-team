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
}

#[test]
fn test_load_parses_toml_with_all_fields() {
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "custom/specs"
default_team = "my-team"
"#;
    fs::write(dir.path().join(".claude-launch.toml"), toml_content).unwrap();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "custom/specs");
    assert_eq!(config.default_team, "my-team");
}

#[test]
fn test_load_uses_defaults_for_missing_fields() {
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "other/specs"
"#;
    fs::write(dir.path().join(".claude-launch.toml"), toml_content).unwrap();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "other/specs");
    assert_eq!(config.default_team, "feature-dev");
}

#[test]
fn test_load_ignores_unknown_keys() {
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "docs/specs"
unknown_key = "should be ignored"
another_unknown = 42
"#;
    fs::write(dir.path().join(".claude-launch.toml"), toml_content).unwrap();
    let config = Config::load(dir.path()).unwrap();
    assert_eq!(config.specs_dir, "docs/specs");
}

#[test]
fn test_load_ignores_base_branch_key() {
    // Old config files with base_branch must parse without error — serde ignores unknown fields
    let dir = create_temp_dir();
    let toml_content = r#"
specs_dir = "docs/specs"
base_branch = "develop"
"#;
    fs::write(dir.path().join(".claude-launch.toml"), toml_content).unwrap();
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

// --- parse_spec_frontmatter tests ---

#[test]
fn test_parse_spec_frontmatter_ready_with_base_branch() {
    let content = "---\nstatus: ready\nbase_branch: main\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Ready);
    assert!(fm.block_reason.is_none());
    assert_eq!(fm.base_branch, Some("main".to_string()));
}

#[test]
fn test_parse_spec_frontmatter_complete_with_base_branch() {
    let content = "---\nstatus: complete\nbase_branch: develop\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Complete);
    assert!(fm.block_reason.is_none());
    assert_eq!(fm.base_branch, Some("develop".to_string()));
}

#[test]
fn test_parse_spec_frontmatter_missing_base_branch_is_blocked() {
    let content = "---\nstatus: ready\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Blocked);
    assert_eq!(
        fm.block_reason,
        Some("Missing required frontmatter field: base_branch".to_string())
    );
    assert!(fm.base_branch.is_none());
}

#[test]
fn test_parse_spec_frontmatter_explicit_blocked_status() {
    let content = "---\nstatus: blocked\nbase_branch: main\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Blocked);
    assert_eq!(
        fm.block_reason,
        Some("Spec is marked blocked — requires human review before running.".to_string())
    );
    assert_eq!(fm.base_branch, Some("main".to_string()));
}

#[test]
fn test_parse_spec_frontmatter_needs_attention_is_blocked() {
    let content = "---\nstatus: needs_attention\nbase_branch: main\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Blocked);
    assert!(fm.block_reason.is_some());
}

#[test]
fn test_parse_spec_frontmatter_no_frontmatter_is_raw() {
    let content = "# My Spec\n\nNo frontmatter here.";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Raw);
    assert!(fm.block_reason.is_none());
    assert!(fm.base_branch.is_none());
}

#[test]
fn test_parse_spec_frontmatter_complete_missing_base_branch_is_complete() {
    // Complete specs take priority over missing base_branch — they're done and non-interactable.
    let content = "---\nstatus: complete\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Complete);
    assert!(fm.block_reason.is_none());
}

#[test]
fn test_discover_specs_complete_without_base_branch_is_not_blocked() {
    // Old completed specs that predate the base_branch requirement should show as complete.
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-old-done.md"),
        "---\nstatus: complete\n---\n# Old completed spec",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].status, SpecStatus::Complete);
    assert!(specs[0].block_reason.is_none());
}

#[test]
fn test_parse_spec_frontmatter_unrecognized_status_defaults_to_ready() {
    let content = "---\nstatus: banana\nbase_branch: main\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Ready);
    assert!(fm.block_reason.is_none());
}

#[test]
fn test_parse_spec_frontmatter_missing_status_defaults_to_ready() {
    let content = "---\nnumber: 4\nbase_branch: main\n---\n# Spec";
    let fm = parse_spec_frontmatter(content);
    assert_eq!(fm.status, SpecStatus::Ready);
    assert!(fm.block_reason.is_none());
}

// --- discover_specs populates block_reason ---

#[test]
fn test_discover_specs_sets_block_reason_for_missing_base_branch() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-spec.md"),
        "---\nstatus: ready\n---\n# Spec",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].status, SpecStatus::Blocked);
    assert_eq!(
        specs[0].block_reason,
        Some("Missing required frontmatter field: base_branch".to_string())
    );
}

#[test]
fn test_discover_specs_no_block_reason_for_valid_spec() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-spec.md"),
        "---\nstatus: ready\nbase_branch: main\n---\n# Spec",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].status, SpecStatus::Ready);
    assert!(specs[0].block_reason.is_none());
}

#[test]
fn test_discover_specs_block_reason_for_explicit_blocked() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-blocked.md"),
        "---\nstatus: blocked\nbase_branch: main\n---\n# Blocked Spec",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].status, SpecStatus::Blocked);
    assert!(specs[0].block_reason.is_some());
}

// --- read_base_branch ---

#[test]
fn test_read_base_branch_returns_value_from_frontmatter() {
    let dir = create_temp_dir();
    let spec_path = dir.path().join("spec.md");
    fs::write(&spec_path, "---\nstatus: ready\nbase_branch: develop\n---\n# Spec").unwrap();

    let result = read_base_branch(&spec_path).unwrap();
    assert_eq!(result, "develop");
}

#[test]
fn test_read_base_branch_errors_when_missing() {
    let dir = create_temp_dir();
    let spec_path = dir.path().join("spec.md");
    fs::write(&spec_path, "---\nstatus: ready\n---\n# Spec").unwrap();

    let result = read_base_branch(&spec_path);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("base_branch"), "error should mention base_branch, got: {msg}");
}

#[test]
fn test_read_base_branch_errors_on_missing_file() {
    let result = read_base_branch(Path::new("/nonexistent/spec.md"));
    assert!(result.is_err());
}

// --- Spec discovery with status filtering tests (updated) ---

#[test]
fn test_discover_specs_includes_complete() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-done.md"),
        "---\nstatus: complete\nbase_branch: main\n---\n# Done",
    )
    .unwrap();
    fs::write(
        dir.path().join("002-active.md"),
        "---\nstatus: ready\nbase_branch: main\n---\n# Active",
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
        "---\nstatus: needs_attention\nbase_branch: main\n---\n# Broken",
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
fn test_discover_specs_includes_blocked() {
    let dir = create_temp_dir();
    fs::write(
        dir.path().join("001-blocked.md"),
        "---\nstatus: blocked\nbase_branch: main\n---\n# Blocked",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].status, SpecStatus::Blocked);
}

// --- Raw requirements file tests ---

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
        "---\nstatus: ready\nbase_branch: main\n---\n# Real spec",
    )
    .unwrap();

    let specs = discover_specs(dir.path()).unwrap();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "spec.md");
}

// --- Team discovery tests (new multi-source API) ---

fn make_teams_dir(root: &TempDir, subpath: &str, teams: &[&str]) {
    let dir = root.path().join(subpath);
    fs::create_dir_all(&dir).unwrap();
    for name in teams {
        fs::write(dir.join(format!("{}.md", name)), "# Team").unwrap();
    }
}

#[test]
fn test_discover_teams_builtin_only() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    fs::write(builtin.path().join("feature-dev.md"), "# Team").unwrap();

    let teams = discover_teams(builtin.path(), user.path(), None).unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].name, "feature-dev");
    assert!(matches!(teams[0].source, TeamSource::BuiltIn));
    assert_eq!(teams[0].path, builtin.path().join("feature-dev.md"));
}

#[test]
fn test_discover_teams_missing_user_dir_silently_skipped() {
    let builtin = create_temp_dir();
    fs::write(builtin.path().join("alpha.md"), "# Team").unwrap();
    let nonexistent_user = Path::new("/nonexistent/user/teams/surely");

    let teams = discover_teams(builtin.path(), nonexistent_user, None).unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].name, "alpha");
}

#[test]
fn test_discover_teams_missing_builtin_dir_errors() {
    let user = create_temp_dir();
    let result = discover_teams(Path::new("/nonexistent/builtin/surely"), user.path(), None);
    assert!(result.is_err());
}

#[test]
fn test_discover_teams_configured_project_dir_missing_errors() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    let missing_project = Path::new("/nonexistent/project/teams/surely");

    let result = discover_teams(builtin.path(), user.path(), Some(missing_project));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.to_lowercase().contains("custom_dir") || msg.to_lowercase().contains("project"),
        "error should mention project/custom_dir, got: {msg}"
    );
}

#[test]
fn test_discover_teams_collision_builtin_vs_user_errors() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    fs::write(builtin.path().join("clash.md"), "# Team").unwrap();
    fs::write(user.path().join("clash.md"), "# Team").unwrap();

    let result = discover_teams(builtin.path(), user.path(), None);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("clash"), "error should name the conflicting team, got: {msg}");
}

#[test]
fn test_discover_teams_collision_builtin_vs_project_errors() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    let project = create_temp_dir();
    fs::write(builtin.path().join("clash.md"), "# Team").unwrap();
    fs::write(project.path().join("clash.md"), "# Team").unwrap();

    let result = discover_teams(builtin.path(), user.path(), Some(project.path()));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("clash"), "error should name the conflicting team, got: {msg}");
}

#[test]
fn test_discover_teams_collision_user_vs_project_errors() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    let project = create_temp_dir();
    fs::write(user.path().join("clash.md"), "# Team").unwrap();
    fs::write(project.path().join("clash.md"), "# Team").unwrap();

    let result = discover_teams(builtin.path(), user.path(), Some(project.path()));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("clash"), "error should name the conflicting team, got: {msg}");
}

#[test]
fn test_discover_teams_collision_error_lists_all_conflicting_names() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    fs::write(builtin.path().join("alpha.md"), "# Team").unwrap();
    fs::write(builtin.path().join("beta.md"), "# Team").unwrap();
    fs::write(user.path().join("alpha.md"), "# Team").unwrap();
    fs::write(user.path().join("beta.md"), "# Team").unwrap();

    let result = discover_teams(builtin.path(), user.path(), None);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("alpha"), "error should mention 'alpha', got: {msg}");
    assert!(msg.contains("beta"), "error should mention 'beta', got: {msg}");
}

#[test]
fn test_discover_teams_clean_merge_all_sources_sorted() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    let project = create_temp_dir();
    fs::write(builtin.path().join("charlie.md"), "# Team").unwrap();
    fs::write(user.path().join("alpha.md"), "# Team").unwrap();
    fs::write(project.path().join("beta.md"), "# Team").unwrap();

    let teams = discover_teams(builtin.path(), user.path(), Some(project.path())).unwrap();
    assert_eq!(teams.len(), 3);
    assert_eq!(teams[0].name, "alpha");
    assert_eq!(teams[1].name, "beta");
    assert_eq!(teams[2].name, "charlie");
    assert!(matches!(teams[0].source, TeamSource::User));
    assert!(matches!(teams[1].source, TeamSource::Project));
    assert!(matches!(teams[2].source, TeamSource::BuiltIn));
}

#[test]
fn test_discover_teams_skips_non_md_files() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    fs::write(builtin.path().join("feature-dev.md"), "# Team").unwrap();
    fs::write(builtin.path().join("notes.txt"), "not a team").unwrap();

    let teams = discover_teams(builtin.path(), user.path(), None).unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].name, "feature-dev");
}

#[test]
fn test_discover_teams_returns_empty_for_empty_dirs() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    let teams = discover_teams(builtin.path(), user.path(), None).unwrap();
    assert!(teams.is_empty());
}

#[test]
fn test_discover_teams_entry_path_is_absolute() {
    let builtin = create_temp_dir();
    let user = create_temp_dir();
    fs::write(builtin.path().join("my-team.md"), "# Team").unwrap();

    let teams = discover_teams(builtin.path(), user.path(), None).unwrap();
    assert_eq!(teams.len(), 1);
    assert!(teams[0].path.is_absolute());
}

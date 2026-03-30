use std::fs;

use tempfile::TempDir;

use super::*;

// --- link_rules ---

#[test]
fn link_rules_creates_symlink() {
    let workflow_dir = TempDir::new().unwrap();
    let claude_dir = TempDir::new().unwrap();
    fs::create_dir(workflow_dir.path().join("rules")).unwrap();

    link_rules(workflow_dir.path(), claude_dir.path()).unwrap();

    let link = claude_dir.path().join("rules").join("agent-workflow");
    assert!(link.is_symlink());
    assert_eq!(
        fs::read_link(&link).unwrap(),
        workflow_dir.path().join("rules")
    );
}

#[test]
fn link_rules_is_idempotent() {
    let workflow_dir = TempDir::new().unwrap();
    let claude_dir = TempDir::new().unwrap();
    fs::create_dir(workflow_dir.path().join("rules")).unwrap();

    link_rules(workflow_dir.path(), claude_dir.path()).unwrap();
    link_rules(workflow_dir.path(), claude_dir.path()).unwrap();
}

#[test]
fn link_rules_errors_if_path_exists_but_is_not_symlink() {
    let workflow_dir = TempDir::new().unwrap();
    let claude_dir = TempDir::new().unwrap();
    fs::create_dir(workflow_dir.path().join("rules")).unwrap();
    fs::create_dir_all(claude_dir.path().join("rules").join("agent-workflow")).unwrap();

    let result = link_rules(workflow_dir.path(), claude_dir.path());
    assert!(result.is_err());
}

// --- register_hooks ---

fn make_workflow_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("hooks")).unwrap();
    dir
}

#[test]
fn register_hooks_creates_settings_if_missing() {
    let workflow_dir = make_workflow_dir();
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");

    register_hooks(workflow_dir.path(), &settings_path).unwrap();

    assert!(settings_path.exists());
}

#[test]
fn register_hooks_adds_all_three_hooks() {
    let workflow_dir = make_workflow_dir();
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");

    register_hooks(workflow_dir.path(), &settings_path).unwrap();

    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    let hooks = settings["hooks"].as_object().unwrap();
    assert!(hooks.contains_key("TaskCompleted"));
    assert!(hooks.contains_key("TaskCreated"));
    assert!(hooks.contains_key("TeammateIdle"));
}

#[test]
fn register_hooks_is_idempotent() {
    let workflow_dir = make_workflow_dir();
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");

    register_hooks(workflow_dir.path(), &settings_path).unwrap();
    register_hooks(workflow_dir.path(), &settings_path).unwrap();

    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    let entries = settings["hooks"]["TaskCompleted"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn register_hooks_preserves_existing_settings() {
    let workflow_dir = make_workflow_dir();
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");
    fs::write(&settings_path, r#"{"autoUpdaterStatus": "disabled"}"#).unwrap();

    register_hooks(workflow_dir.path(), &settings_path).unwrap();

    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(settings["autoUpdaterStatus"], "disabled");
}

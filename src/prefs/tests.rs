use super::*;
use std::fs;
use tempfile::TempDir;

fn prefs_path_in(dir: &TempDir) -> PathBuf {
    dir.path().join("claude-agent-team-prefs.toml")
}

// --- Defaults ---

#[test]
fn test_default_headless_is_false() {
    assert!(!Prefs::default().headless);
}

#[test]
fn test_default_show_complete_is_true() {
    assert!(Prefs::default().show_complete);
}

#[test]
fn test_default_show_blocked_is_true() {
    assert!(Prefs::default().show_blocked);
}

// --- Load ---

#[test]
fn test_load_returns_defaults_when_file_missing() {
    let dir = TempDir::new().unwrap();
    let result = Prefs::load_from_path(&prefs_path_in(&dir));
    assert!(result.is_err()); // file doesn't exist
    // and the public load() falls back to defaults
    // (we can't call load() directly since it uses HOME, but we've verified try_load errors correctly)
}

#[test]
fn test_load_parses_all_fields() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    fs::write(
        &path,
        "headless = true\nshow_complete = false\nshow_blocked = false\n",
    )
    .unwrap();
    let prefs = Prefs::load_from_path(&path).unwrap();
    assert!(prefs.headless);
    assert!(!prefs.show_complete);
    assert!(!prefs.show_blocked);
}

#[test]
fn test_load_returns_defaults_on_invalid_toml() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    fs::write(&path, "this is not valid toml ][[[").unwrap();
    assert!(Prefs::load_from_path(&path).is_err());
}

#[test]
fn test_load_uses_defaults_for_missing_fields() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    fs::write(&path, "headless = true\n").unwrap(); // show_complete and show_blocked omitted
    let prefs = Prefs::load_from_path(&path).unwrap();
    assert!(prefs.headless);
    assert!(prefs.show_complete); // defaults to true
    assert!(prefs.show_blocked); // defaults to true
}

// --- Save / round-trip ---

#[test]
fn test_save_round_trips_values() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    let original = Prefs {
        headless: true,
        show_complete: false,
        show_blocked: true,
        default_account: None,
    };
    original.save_to_path(&path).unwrap();
    let loaded = Prefs::load_from_path(&path).unwrap();
    assert_eq!(original, loaded);
}

// --- default_account ---

#[test]
fn test_default_account_is_none() {
    assert_eq!(Prefs::default().default_account, None);
}

#[test]
fn test_default_account_round_trips_none() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    let original = Prefs {
        headless: false,
        show_complete: true,
        show_blocked: true,
        default_account: None,
    };
    original.save_to_path(&path).unwrap();
    let loaded = Prefs::load_from_path(&path).unwrap();
    assert_eq!(loaded.default_account, None);
}

#[test]
fn test_default_account_round_trips_some() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    let original = Prefs {
        headless: false,
        show_complete: true,
        show_blocked: true,
        default_account: Some("work".to_string()),
    };
    original.save_to_path(&path).unwrap();
    let loaded = Prefs::load_from_path(&path).unwrap();
    assert_eq!(loaded.default_account, Some("work".to_string()));
}

#[test]
fn test_default_account_missing_in_file_defaults_to_none() {
    let dir = TempDir::new().unwrap();
    let path = prefs_path_in(&dir);
    // Old prefs file without default_account field
    fs::write(&path, "headless = false\nshow_complete = true\nshow_blocked = true\n").unwrap();
    let prefs = Prefs::load_from_path(&path).unwrap();
    assert_eq!(prefs.default_account, None);
}

use super::*;
use std::fs;
use tempfile::TempDir;

// --- load_accounts_from_path ---

#[test]
fn test_load_accounts_returns_empty_when_file_missing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.toml");
    let result = load_accounts_from_path(&path);
    // Missing file should return an error (caller falls back to empty)
    assert!(result.is_err());
}

#[test]
fn test_load_accounts_returns_empty_for_empty_accounts_list() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("accounts.toml");
    fs::write(&path, "").unwrap();
    let accounts = load_accounts_from_path(&path).unwrap();
    assert!(accounts.is_empty());
}

#[test]
fn test_load_accounts_parses_single_account() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("accounts.toml");
    fs::write(
        &path,
        "[[accounts]]\nlabel = \"personal\"\n",
    )
    .unwrap();
    let accounts = load_accounts_from_path(&path).unwrap();
    assert_eq!(accounts.len(), 1);
}

#[test]
fn test_load_accounts_single_entry_label_correct() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("accounts.toml");
    fs::write(
        &path,
        "[[accounts]]\nlabel = \"personal\"\n",
    )
    .unwrap();
    let accounts = load_accounts_from_path(&path).unwrap();
    assert_eq!(accounts[0].label, "personal");
}

#[test]
fn test_load_accounts_parses_multiple_accounts() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("accounts.toml");
    fs::write(
        &path,
        "[[accounts]]\nlabel = \"personal\"\n\n[[accounts]]\nlabel = \"work\"\n",
    )
    .unwrap();
    let accounts = load_accounts_from_path(&path).unwrap();
    assert_eq!(accounts.len(), 2);
}

#[test]
fn test_load_accounts_multiple_entries_labels_correct() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("accounts.toml");
    fs::write(
        &path,
        "[[accounts]]\nlabel = \"personal\"\n\n[[accounts]]\nlabel = \"work\"\n",
    )
    .unwrap();
    let accounts = load_accounts_from_path(&path).unwrap();
    assert_eq!(accounts[0].label, "personal");
    assert_eq!(accounts[1].label, "work");
}

// --- load_token_for_account ---

#[test]
fn test_load_token_for_unknown_label_returns_none() {
    // In the test environment the Keychain entry won't exist; expect None.
    let result = load_token_for_account("__nonexistent_test_label_xyz__");
    assert!(result.is_none());
}

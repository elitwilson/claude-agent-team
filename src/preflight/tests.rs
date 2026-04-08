use std::process::Command;
use std::sync::Mutex;

use tempfile::TempDir;

use super::*;

// set_current_dir is process-global; serialize tests that use it.
static DIR_LOCK: Mutex<()> = Mutex::new(());

// --- build_branch_name tests ---

#[test]
fn test_build_branch_name_format() {
    let name = build_branch_name("my-feature");
    assert_eq!(name, "feature/my-feature");
}

#[test]
fn test_build_branch_name_with_different_slug() {
    let name = build_branch_name("claude-bros");
    assert_eq!(name, "feature/claude-bros");
}

// --- git validation tests ---

fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    Command::new("git").args(["init"]).current_dir(path).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(path).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(path).output().unwrap();

    // Need at least one commit for branches to exist
    std::fs::write(path.join("init.txt"), "init").unwrap();
    Command::new("git").args(["add", "."]).current_dir(path).output().unwrap();
    Command::new("git").args(["commit", "-m", "init"]).current_dir(path).output().unwrap();

    dir
}

fn git_in(dir: &TempDir, args: &[&str]) {
    Command::new("git").args(args).current_dir(dir.path()).output().unwrap();
}

#[test]
fn test_check_branch_exists_succeeds_for_existing_branch() {
    let _lock = DIR_LOCK.lock().unwrap();
    let dir = init_repo();
    std::env::set_current_dir(dir.path()).unwrap();

    assert!(check_branch_exists("main").is_ok() || check_branch_exists("master").is_ok());
}

#[test]
fn test_check_branch_exists_fails_for_missing_branch() {
    let _lock = DIR_LOCK.lock().unwrap();
    let dir = init_repo();
    std::env::set_current_dir(dir.path()).unwrap();

    let result = check_branch_exists("nonexistent-branch");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("does not exist locally"));
}

#[test]
fn test_check_branch_absent_succeeds_when_branch_missing() {
    let _lock = DIR_LOCK.lock().unwrap();
    let dir = init_repo();
    std::env::set_current_dir(dir.path()).unwrap();

    assert!(check_branch_absent("feature/not-yet-created").is_ok());
}

#[test]
fn test_check_branch_absent_fails_when_branch_exists() {
    let _lock = DIR_LOCK.lock().unwrap();
    let dir = init_repo();
    std::env::set_current_dir(dir.path()).unwrap();

    git_in(&dir, &["checkout", "-b", "feature/already-exists"]);

    let result = check_branch_absent("feature/already-exists");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

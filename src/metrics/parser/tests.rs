use super::*;
use serde_json::json;

// --- derive_project_dir tests ---

#[test]
fn test_derive_project_dir_standard_path() {
    assert_eq!(
        derive_project_dir("/Users/charlo/dev/myproject"),
        "-Users-charlo-dev-myproject"
    );
}

#[test]
fn test_derive_project_dir_root_path() {
    assert_eq!(derive_project_dir("/"), "-");
}

#[test]
fn test_derive_project_dir_no_leading_slash() {
    // Edge case: path without leading slash
    assert_eq!(derive_project_dir("Users/charlo"), "Users-charlo");
}

// --- is_agent_file tests ---

#[test]
fn test_is_agent_file_true() {
    assert!(is_agent_file("agent-abc123.jsonl"));
}

#[test]
fn test_is_agent_file_false_for_main_session() {
    assert!(!is_agent_file("session-abc123.jsonl"));
}

#[test]
fn test_is_agent_file_false_for_plain_jsonl() {
    assert!(!is_agent_file("data.jsonl"));
}

// --- attribute_role tests ---

#[test]
fn test_attribute_role_coder() {
    let msg = "You are the Coder on a TDD implementation team.";
    assert_eq!(attribute_role(msg), Some("coder".to_string()));
}

#[test]
fn test_attribute_role_reviewer() {
    let msg = "You are the Reviewer. Your job is to review tests.";
    assert_eq!(attribute_role(msg), Some("reviewer".to_string()));
}

#[test]
fn test_attribute_role_case_insensitive() {
    assert_eq!(
        attribute_role("the coder role"),
        Some("coder".to_string())
    );
    assert_eq!(
        attribute_role("the REVIEWER role"),
        Some("reviewer".to_string())
    );
}

#[test]
fn test_attribute_role_coder_takes_precedence() {
    // If both keywords appear, "Coder" check runs first per spec
    let msg = "You are the Coder and Reviewer.";
    assert_eq!(attribute_role(msg), Some("coder".to_string()));
}

#[test]
fn test_attribute_role_unknown() {
    let msg = "You are the Lead agent.";
    assert_eq!(attribute_role(msg), None);
}

// --- extract_tokens tests ---

#[test]
fn test_extract_tokens_all_fields() {
    let usage = json!({
        "input_tokens": 100,
        "output_tokens": 50,
        "cache_creation_input_tokens": 25,
        "cache_read_input_tokens": 10
    });
    assert_eq!(extract_tokens(&usage), (100, 50, 25, 10));
}

#[test]
fn test_extract_tokens_optional_fields_default_to_zero() {
    let usage = json!({
        "input_tokens": 100,
        "output_tokens": 50
    });
    assert_eq!(extract_tokens(&usage), (100, 50, 0, 0));
}

#[test]
fn test_extract_tokens_empty_object() {
    let usage = json!({});
    assert_eq!(extract_tokens(&usage), (0, 0, 0, 0));
}

use super::*;
use std::fs;
use tempfile::TempDir;

// --- render_prompt tests ---

#[test]
fn test_render_prompt_substitutes_all_variables() {
    let dir = TempDir::new().unwrap();
    let template = "Spec: ${SPEC_FILE}\nSlug: ${FEATURE_SLUG}\nDir: ${WORKFLOW_DIR}\nTeam: ${TEAM}";
    let template_path = dir.path().join("template.md");
    fs::write(&template_path, template).unwrap();

    let result = render_prompt(
        &template_path,
        "docs/specs/my-feature.md",
        "my-feature",
        "/home/user/repo",
        "feature-dev",
        "/home/user/.claude-launch/user",
        "",
    )
    .unwrap();

    assert_eq!(
        result,
        "Spec: docs/specs/my-feature.md\nSlug: my-feature\nDir: /home/user/repo\nTeam: feature-dev"
    );
}

#[test]
fn test_render_prompt_handles_repeated_variables() {
    let dir = TempDir::new().unwrap();
    let template = "${SPEC_FILE} and ${SPEC_FILE} again";
    let template_path = dir.path().join("template.md");
    fs::write(&template_path, template).unwrap();

    let result =
        render_prompt(&template_path, "spec.md", "slug", "/dir", "team", "/user", "").unwrap();
    assert_eq!(result, "spec.md and spec.md again");
}

#[test]
fn test_render_prompt_errors_on_missing_template() {
    let result = render_prompt(
        Path::new("/nonexistent/template.md"),
        "spec.md",
        "slug",
        "/dir",
        "team",
        "/user",
        "",
    );
    assert!(result.is_err());
}

#[test]
fn test_render_prompt_preserves_non_variable_text() {
    let dir = TempDir::new().unwrap();
    let template = "Plain text with no variables at all.";
    let template_path = dir.path().join("template.md");
    fs::write(&template_path, template).unwrap();

    let result =
        render_prompt(&template_path, "spec.md", "slug", "/dir", "team", "/user", "").unwrap();
    assert_eq!(result, "Plain text with no variables at all.");
}

#[test]
fn test_render_prompt_substitutes_user_dir() {
    let dir = TempDir::new().unwrap();
    let template = "Agent: ${USER_DIR}/agents/foo/coder.md";
    let template_path = dir.path().join("template.md");
    fs::write(&template_path, template).unwrap();

    let result = render_prompt(
        &template_path,
        "spec.md",
        "slug",
        "/workflow",
        "team",
        "/home/user/.claude-launch/user",
        "",
    )
    .unwrap();

    assert_eq!(
        result,
        "Agent: /home/user/.claude-launch/user/agents/foo/coder.md"
    );
}

#[test]
fn test_render_prompt_substitutes_project_dir() {
    let dir = TempDir::new().unwrap();
    let template = "Agent: ${PROJECT_DIR}/agents/foo/coder.md";
    let template_path = dir.path().join("template.md");
    fs::write(&template_path, template).unwrap();

    let result = render_prompt(
        &template_path,
        "spec.md",
        "slug",
        "/workflow",
        "team",
        "/user",
        "/home/user/project/custom-teams",
    )
    .unwrap();

    assert_eq!(
        result,
        "Agent: /home/user/project/custom-teams/agents/foo/coder.md"
    );
}

#[test]
fn test_render_prompt_project_dir_empty_string_substitutes_nothing() {
    let dir = TempDir::new().unwrap();
    let template = "Prefix: ${PROJECT_DIR}";
    let template_path = dir.path().join("template.md");
    fs::write(&template_path, template).unwrap();

    let result = render_prompt(
        &template_path,
        "spec.md",
        "slug",
        "/workflow",
        "team",
        "/user",
        "",
    )
    .unwrap();

    assert_eq!(result, "Prefix: ");
}

// --- user dir creation tests ---

#[test]
fn test_resolve_workflow_dir_creates_user_teams_dir() {
    let dir = TempDir::new().unwrap();
    let prompts_dir = dir.path().join("prompts");
    fs::create_dir(&prompts_dir).unwrap();

    unsafe {
        std::env::set_var("CLAUDE_LAUNCH_DIR", dir.path().to_str().unwrap());
    }
    let result = resolve_workflow_dir();
    unsafe {
        std::env::remove_var("CLAUDE_LAUNCH_DIR");
    }

    result.unwrap();
    assert!(dir.path().join("user").join("teams").exists());
    assert!(dir.path().join("user").join("agents").exists());
}

// --- resolve_workflow_dir tests ---

#[test]
fn test_resolve_workflow_dir_uses_env_var_if_set() {
    // This test verifies the function checks CLAUDE_AGENT_TEAM_DIR env var.
    // We set the env var temporarily and verify it's used.
    let dir = TempDir::new().unwrap();
    let prompts_dir = dir.path().join("prompts");
    fs::create_dir(&prompts_dir).unwrap();

    // SAFETY: This test runs single-threaded and restores the var immediately.
    unsafe {
        std::env::set_var("CLAUDE_LAUNCH_DIR", dir.path().to_str().unwrap());
    }
    let result = resolve_workflow_dir();
    unsafe {
        std::env::remove_var("CLAUDE_LAUNCH_DIR");
    }

    assert_eq!(result.unwrap(), dir.path().to_str().unwrap());
}

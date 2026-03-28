use super::*;

// --- build_claude_args tests ---

#[test]
fn test_build_claude_args_interactive_mode() {
    let args = build_claude_args("my prompt text", false);
    assert_eq!(
        args,
        vec![
            "--max-turns",
            "200",
            "--dangerously-skip-permissions",
            "--teammate-mode",
            "in-process",
            "my prompt text",
        ]
    );
}

#[test]
fn test_build_claude_args_headless_mode() {
    let args = build_claude_args("my prompt text", true);
    assert_eq!(
        args,
        vec![
            "--print",
            "--max-turns",
            "200",
            "--dangerously-skip-permissions",
            "--teammate-mode",
            "in-process",
            "my prompt text",
        ]
    );
}

// --- build_log_path tests ---

#[test]
fn test_build_log_path_format() {
    let path = build_log_path("my-feature", "20260327");
    assert_eq!(path, "logs/agent-runs/my-feature-20260327.log");
}

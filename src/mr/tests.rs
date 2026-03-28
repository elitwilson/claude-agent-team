use super::*;

// --- build_mr_title tests ---

#[test]
fn test_build_mr_title_success() {
    let title = build_mr_title("my-feature", 0);
    assert_eq!(title, "my-feature");
}

#[test]
fn test_build_mr_title_incomplete_on_nonzero_exit() {
    let title = build_mr_title("my-feature", 1);
    assert_eq!(title, "INCOMPLETE: my-feature");
}

#[test]
fn test_build_mr_title_incomplete_on_negative_exit() {
    let title = build_mr_title("my-feature", -1);
    assert_eq!(title, "INCOMPLETE: my-feature");
}

// --- build_mr_description tests ---

#[test]
fn test_build_mr_description_success() {
    let desc = build_mr_description("my-feature", 0);
    assert!(!desc.contains("INCOMPLETE"));
    assert!(!desc.contains("warning"));
}

#[test]
fn test_build_mr_description_includes_warning_on_failure() {
    let desc = build_mr_description("my-feature", 1);
    // Should contain some indication that the run was incomplete
    assert!(desc.to_lowercase().contains("incomplete") || desc.to_lowercase().contains("warning"));
}

// --- build_push_args tests ---

#[test]
fn test_build_push_args_includes_push_options() {
    let args = build_push_args(
        "feature/my-feature-20260327",
        "main",
        "my-feature",
        "Automated MR",
    );
    // Should push to origin with the branch
    assert!(args.contains(&"origin".to_string()));
    assert!(args.contains(&"feature/my-feature-20260327".to_string()));
    // Should include GitLab push options for MR creation
    let args_str = args.join(" ");
    assert!(args_str.contains("merge_request.create"));
    assert!(args_str.contains("merge_request.target=main"));
    assert!(args_str.contains("merge_request.title="));
}

// --- format_summary tests ---

#[test]
fn test_format_summary_all_success() {
    let summary = format_summary("feature/my-feature-20260327", true, true);
    assert!(summary.contains("feature/my-feature-20260327"));
    assert!(summary.to_lowercase().contains("mr"));
    assert!(summary.to_lowercase().contains("metrics"));
}

#[test]
fn test_format_summary_mr_failed() {
    let summary = format_summary("feature/my-feature-20260327", false, true);
    assert!(summary.contains("feature/my-feature-20260327"));
    // Should indicate MR was not created
    let lower = summary.to_lowercase();
    assert!(lower.contains("mr") || lower.contains("merge"));
}

#[test]
fn test_format_summary_metrics_failed() {
    let summary = format_summary("feature/my-feature-20260327", true, false);
    assert!(summary.contains("feature/my-feature-20260327"));
}

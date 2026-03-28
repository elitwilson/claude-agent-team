use super::*;

// --- build_branch_name tests ---

#[test]
fn test_build_branch_name_format() {
    let name = build_branch_name("my-feature", "20260327");
    assert_eq!(name, "feature/my-feature-20260327");
}

#[test]
fn test_build_branch_name_with_different_slug() {
    let name = build_branch_name("claude-bros", "20260101");
    assert_eq!(name, "feature/claude-bros-20260101");
}

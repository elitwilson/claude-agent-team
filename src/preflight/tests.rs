use super::*;

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

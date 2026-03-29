use std::path::PathBuf;

#[test]
fn hello_world_file_exists_with_correct_contents() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let file_path = repo_root.join("hello_world.txt");

    let contents = std::fs::read_to_string(&file_path)
        .expect("hello_world.txt should exist at the repository root");

    assert_eq!(contents, "Hello, World!");
}

use tempfile::TempDir;

#[test]
fn test_temp_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().join("test.txt");

    std::fs::write(&temp_path, "Hello, test!").unwrap();

    let contents = std::fs::read_to_string(&temp_path).unwrap();
    assert_eq!(contents, "Hello, test!");
}

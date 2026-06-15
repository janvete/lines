use std::path::PathBuf;

#[test]
fn test_example_files_exist() {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples");
    assert!(examples.join("Synology/dsmview.md").exists());
    assert!(examples.join("Scripts/update.md").exists());
}

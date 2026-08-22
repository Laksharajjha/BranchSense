use std::{fs, path::PathBuf};

use super::{DiscoveryOptions, SourceDiscovery};

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary root");
    fs::create_dir_all(root.path().join("src/z")).expect("source directory");
    fs::create_dir_all(root.path().join("target/classes")).expect("ignored directory");
    fs::write(root.path().join("src/z/Z.java"), "class Z {}").expect("Java file");
    fs::write(root.path().join("src/A.java"), "class A {}").expect("Java file");
    fs::write(root.path().join("target/classes/Generated.java"), "class Generated {}")
        .expect("generated file");
    root
}

#[test]
fn discovery_is_sorted_and_ignores_build_directories() {
    let root = fixture();
    let result =
        SourceDiscovery::new(DiscoveryOptions::default()).discover(root.path()).expect("discovery");
    let paths =
        result.files().iter().map(|file| file.relative_path().to_owned()).collect::<Vec<_>>();
    assert_eq!(paths, vec![PathBuf::from("src/A.java"), PathBuf::from("src/z/Z.java")]);
    assert_eq!(result.skipped(), 1);
}

#[test]
fn discovery_can_include_an_ignored_directory() {
    let root = fixture();
    let options = DiscoveryOptions::default().include_directory("target");
    let result = SourceDiscovery::new(options).discover(root.path()).expect("discovery");
    assert_eq!(result.files().len(), 3);
}

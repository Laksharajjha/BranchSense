use std::{fs, path::PathBuf};

use super::{DiscoveryOptions, IndexOptions, RepositoryIndex, SourceDiscovery};
use branchsense_graph::EdgeKind;
use branchsense_query::Query;
use branchsense_semantic::ResolutionState;

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

#[test]
fn index_builds_one_graph_from_multiple_java_files() {
    let root = repository_fixture();
    let index = RepositoryIndex::new(IndexOptions::default());
    let result = index.index(root.path(), None).expect("repository index");
    assert_eq!(result.report().discovered(), 3);
    assert_eq!(result.report().indexed(), 3);
    assert_eq!(result.snapshot().graph().statistics().documents(), 3);
    assert!(result.snapshot().graph().statistics().symbols() >= 6);
    assert!(result.snapshot().graph().edges().any(|edge| {
        edge.kind() == EdgeKind::Imports
            && matches!(edge.resolution(), Some(ResolutionState::Resolved(_)))
    }));
    let query = Query::new(result.snapshot().graph());
    assert_eq!(
        query
            .symbol_by_qualified_name(
                &branchsense_core::QualifiedName::new("billing.PaymentService.process()")
                    .expect("name")
            )
            .expect("service method")
            .name(),
        "process"
    );
}

#[test]
fn index_reuses_unchanged_documents_and_handles_changes_and_deletes() {
    let root = repository_fixture();
    let index = RepositoryIndex::new(IndexOptions::default());
    let first = index.index(root.path(), None).expect("initial index");
    let second = index.index(root.path(), Some(first.snapshot())).expect("unchanged index");
    assert_eq!(second.report().unchanged(), 3);
    assert_eq!(second.report().indexed(), 3);

    fs::write(
        root.path().join("src/PaymentService.java"),
        "package billing; public class PaymentService { public void changed() {} }",
    )
    .expect("changed source");
    let changed = index.index(root.path(), Some(second.snapshot())).expect("changed index");
    assert_eq!(changed.report().unchanged(), 2);
    assert_eq!(changed.snapshot().graph().statistics().documents(), 3);

    fs::remove_file(root.path().join("src/User.java")).expect("deleted source");
    let deleted = index.index(root.path(), Some(changed.snapshot())).expect("deleted index");
    assert_eq!(deleted.snapshot().graph().statistics().documents(), 2);
    assert_eq!(first.snapshot().graph().statistics().documents(), 3);
}

#[test]
fn malformed_file_does_not_abort_repository_indexing() {
    let root = repository_fixture();
    fs::write(root.path().join("src/Broken.java"), "class Broken {").expect("malformed source");
    let result = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("partial index");
    assert_eq!(result.report().discovered(), 4);
    assert_eq!(result.report().indexed(), 4);
    assert!(result.report().parse_diagnostics() > 0);
    assert_eq!(result.snapshot().graph().statistics().documents(), 4);
}

fn repository_fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary root");
    fs::create_dir_all(root.path().join("src")).expect("source directory");
    fs::write(root.path().join("src/User.java"), "package billing; public class User {}")
        .expect("user source");
    fs::write(
        root.path().join("src/PaymentValidator.java"),
        "package billing; public class PaymentValidator { public void validate() {} }",
    )
    .expect("validator source");
    fs::write(root.path().join("src/PaymentService.java"), "package billing; import billing.PaymentValidator; public class PaymentService { public void process() { validate(); } }").expect("service source");
    root
}

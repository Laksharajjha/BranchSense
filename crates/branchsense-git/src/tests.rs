use std::{fs, process::Command};

use super::{GitRefKind, GitRepository, GitSnapshotIndexer, MergeBaseResult};

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary directory");
    run(root.path(), &["init", "-b", "main"]);
    run(root.path(), &["config", "user.name", "BranchSense Test"]);
    run(root.path(), &["config", "user.email", "test@branchsense.invalid"]);
    fs::write(root.path().join("README.md"), "base\n").expect("base file");
    fs::create_dir_all(root.path().join("src")).expect("source directory");
    fs::write(root.path().join("src/Hello.java"), "class Hello { void greet() {} }\n")
        .expect("Java source");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-m", "base"]);
    run(root.path(), &["branch", "feature"]);
    fs::write(root.path().join("README.md"), "main\n").expect("main file");
    run(root.path(), &["commit", "-am", "main change"]);
    run(root.path(), &["checkout", "feature"]);
    fs::write(root.path().join("README.md"), "feature\n").expect("feature file");
    run(root.path(), &["commit", "-am", "feature change"]);
    run(root.path(), &["checkout", "main"]);
    root
}

fn run(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git available");
    assert!(status.success(), "git command failed: {args:?}");
}

#[test]
fn discovers_and_resolves_repository_state() {
    let root = repository();
    let repo = GitRepository::discover(root.path()).expect("repository");
    assert_eq!(repo.identity().worktree(), Some(root.path()));
    assert_eq!(repo.head().expect("head").message(), "main change\n");
    let branches = repo.local_branches().expect("branches");
    assert_eq!(branches.len(), 2);
    assert!(branches.iter().all(|reference| reference.kind() == GitRefKind::LocalBranch));
}

#[test]
fn computes_divergent_merge_base_without_writing_repository() {
    let root = repository();
    let before = fs::read(root.path().join(".git/HEAD")).expect("head before");
    let repo = GitRepository::discover(root.path()).expect("repository");
    let main = repo.resolve("main").expect("main");
    let feature = repo.resolve("feature").expect("feature");
    match repo.merge_bases(&main, &feature).expect("merge base") {
        MergeBaseResult::Single(base) => assert_eq!(base.message(), "base\n"),
        other => panic!("expected one merge base, got {other:?}"),
    }
    let after = fs::read(root.path().join(".git/HEAD")).expect("head after");
    assert_eq!(before, after);
}

#[test]
fn bounded_history_is_newest_first_and_read_only() {
    let repo = repository();
    let git = GitRepository::discover(repo.path()).expect("discover repository");
    let head = git.resolve("main").expect("resolve main");
    let history = git.history(&head, 2).expect("history");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].commit_id(), head.commit_id());
    assert_eq!(git.resolve("main").expect("resolve unchanged main"), head);
}

#[test]
fn rejects_missing_reference() {
    let root = repository();
    let repo = GitRepository::discover(root.path()).expect("repository");
    assert!(repo.reference("missing").is_err());
}

#[test]
fn indexes_a_commit_tree_without_checkout() {
    let root = repository();
    let head_before = fs::read(root.path().join(".git/HEAD")).expect("head before");
    let repository = GitRepository::discover(root.path()).expect("repository");
    let revision = repository.head().expect("head");
    let snapshot = GitSnapshotIndexer::default()
        .index_revision(&repository, &revision, None)
        .expect("Git semantic snapshot");
    assert_eq!(snapshot.revision().commit_id(), revision.commit_id());
    assert_eq!(snapshot.report().indexed(), 1);
    assert_eq!(snapshot.semantic().graph().statistics().documents(), 1);
    assert_eq!(fs::read(root.path().join(".git/HEAD")).expect("head after"), head_before);
    assert_eq!(
        fs::read_to_string(root.path().join("src/Hello.java")).expect("working-tree source"),
        "class Hello { void greet() {} }\n"
    );
}

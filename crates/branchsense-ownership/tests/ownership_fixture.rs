//! Controlled Git fixture for responsibility evidence.
#![allow(missing_docs)]

use std::process::Command;

use branchsense_git::GitRepository;
use branchsense_ownership::{ResponsibilityAnalyzer, ResponsibilityEntity, ResponsibilityOptions};
use tempfile::TempDir;

fn run(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git runs");
    assert!(status.success(), "git command failed: {args:?}");
}

fn commit(root: &std::path::Path, message: &str, name: &str, email: &str, date: &str) {
    let status = Command::new("git")
        .args(["commit", "-am", message])
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", name)
        .env("GIT_AUTHOR_EMAIL", email)
        .env("GIT_COMMITTER_NAME", name)
        .env("GIT_COMMITTER_EMAIL", email)
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .status()
        .expect("git runs");
    assert!(status.success(), "commit failed: {message}");
}

fn fixture() -> TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "-q", "-b", "main"]);
    run(root, &["config", "user.name", "Fixture"]);
    run(root, &["config", "user.email", "fixture@example.com"]);
    std::fs::create_dir(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/Payment.java"), "class Payment { void process() {} }\n")
        .expect("source");
    run(root, &["add", "."]);
    commit(root, "base", "Alice", "alice@example.com", "2020-01-01T00:00:00Z");
    std::fs::write(root.join("src/Payment.java"), "class Payment { void process(int value) {} }\n")
        .expect("source");
    commit(root, "change", "Bob", "bob@example.com", "2020-01-02T00:00:00Z");
    std::fs::write(
        root.join("src/Payment.java"),
        "class Payment { void process(long value) {} }\n",
    )
    .expect("source");
    commit(root, "change again", "Bob", "BOB@example.com", "2020-01-03T00:00:00Z");
    directory
}

#[test]
fn reports_conservative_symbol_contribution_evidence() {
    let directory = fixture();
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    let revision = repository.resolve("main").expect("resolve main");
    let signals = ResponsibilityAnalyzer::new()
        .analyze(&repository, &revision, ResponsibilityOptions::new(3))
        .expect("analyze responsibility");
    let evidence = signals.symbol_responsibility().iter().find(|item| {
        matches!(item.entity(), ResponsibilityEntity::Symbol(symbol) if symbol.qualified_name() == "Payment.process")
    }).expect("symbol evidence");
    assert_eq!(evidence.contributions().len(), 2);
    assert_eq!(evidence.contributions()[0].contributor().name(), "Bob");
    assert!((evidence.contributions()[0].share() - 2.0 / 3.0).abs() < f64::EPSILON);
    assert_eq!(evidence.concentration().active_contributors(), 2);
    assert_eq!(evidence.supporting_commits().len(), 3);
    assert_eq!(
        signals,
        serde_json::from_slice(&serde_json::to_vec(&signals).expect("serialize"))
            .expect("deserialize")
    );
    assert_eq!(repository.resolve("main").expect("unchanged revision"), revision);
}

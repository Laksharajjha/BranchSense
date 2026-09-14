#![allow(missing_docs)]
use std::process::Command;
use tempfile::TempDir;

use branchsense_evaluation::model::EvaluationDiagnostic;
use branchsense_semantic::{DatasetSchemaVersion, EvalOutcome, EvalRepositoryIdentity, EvalRevision};
use branchsense_evaluation::model::EvaluationCase;

fn run_git(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().unwrap();
    assert!(status.success());
}

fn commit(root: &std::path::Path, msg: &str) {
    Command::new("git")
        .args(["commit", "-am", msg])
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "Test Fixture")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test Fixture")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .status()
        .unwrap();
}

fn create_controlled_fixture() -> TempDir {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    run_git(root, &["init", "-q", "-b", "main"]);
    run_git(root, &["config", "user.name", "Test Fixture"]);
    run_git(root, &["config", "user.email", "test@example.com"]);
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::write(root.join("src/System.java"), "class System { void start() {} }\n").unwrap();
    run_git(root, &["add", "."]);
    commit(root, "base");

    run_git(root, &["branch", "feature_a"]);
    run_git(root, &["branch", "feature_b"]);

    run_git(root, &["checkout", "feature_a"]);
    std::fs::write(root.join("src/System.java"), "class System { void start(int x) {} }\n").unwrap();
    commit(root, "branch a changes");

    run_git(root, &["checkout", "feature_b"]);
    std::fs::write(root.join("src/System.java"), "class System { void start(String y) {} }\n").unwrap();
    commit(root, "branch b changes");

    directory
}

#[test]
fn test_harness_evaluates_controlled_fixture_deterministically() {
    let fixture = create_controlled_fixture();
    let root = fixture.path();
    
    // NOTE: This represents a TEST FIXTURE scenario, not a real historical integration.
    let outcome = EvalOutcome::new().with_semantic_integration_issue(true);
    let case = EvaluationCase::new(
        "synthetic-fixture-01",
        DatasetSchemaVersion::current(),
        EvalRepositoryIdentity::new("test", None::<String>),
        EvalRevision::new("main"),
        EvalRevision::new("feature_a"),
        EvalRevision::new("feature_b"),
        EvalRevision::new("main"),
        outcome,
    );

    // Ensure deterministic non-interference from ground truth
    let mut outcome2 = EvalOutcome::new();
    outcome2.build_failure = Some(false);
    
    let case2 = EvaluationCase::new(
        "synthetic-fixture-02",
        DatasetSchemaVersion::current(),
        EvalRepositoryIdentity::new("test", None::<String>),
        EvalRevision::new("main"),
        EvalRevision::new("feature_a"),
        EvalRevision::new("feature_b"),
        EvalRevision::new("main"),
        outcome2,
    );

    let results = branchsense_evaluation::runner::run_dataset(&[case, case2], root);
    assert_eq!(results.len(), 2);
    
    let res1 = &results[0];
    let res2 = &results[1];

    assert_eq!(res1.diagnostic(), &EvaluationDiagnostic::Abstained);
    assert_eq!(res2.diagnostic(), &EvaluationDiagnostic::Abstained);
    
    // Non-interference check: The BCS score MUST be perfectly identical regardless of the varying outcome labels.
    assert_eq!(res1.assessment().unwrap().ordinal_score(), res2.assessment().unwrap().ordinal_score());
    assert_eq!(res1.assessment().unwrap().band(), res2.assessment().unwrap().band());
    assert_eq!(res1.observed_outcome().semantic_integration_issue, Some(true));
    assert_eq!(res2.observed_outcome().build_failure, Some(false));
}

#[test]
fn test_harness_handles_unavailable_repository() {
    let outcome = EvalOutcome::new();
    let case = EvaluationCase::new(
        "synthetic-fixture-missing",
        DatasetSchemaVersion::current(),
        EvalRepositoryIdentity::new("test", None::<String>),
        EvalRevision::new("main"),
        EvalRevision::new("feature_a"),
        EvalRevision::new("feature_b"),
        EvalRevision::new("main"),
        outcome,
    );

    let results = branchsense_evaluation::runner::run_dataset(&[case], std::path::Path::new("/does/not/exist"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].diagnostic(), &EvaluationDiagnostic::UnavailableRepository);
}

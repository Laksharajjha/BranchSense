//! Controlled Git history coverage for historical semantic evidence.
#![allow(missing_docs)]

use std::process::Command;

use branchsense_git::GitRepository;
use branchsense_history::{HistoricalAnalyzer, HistoricalOptions};
use branchsense_semantic::EvidenceState;
use tempfile::{TempDir, tempdir};

fn run(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git runs");
    assert!(status.success(), "git command failed: {args:?}");
}

fn commit(root: &std::path::Path, message: &str, date: &str) {
    let status = Command::new("git")
        .args(["commit", "-am", message])
        .current_dir(root)
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .status()
        .expect("git runs");
    assert!(status.success(), "commit failed: {message}");
}

fn repository() -> TempDir {
    let directory = tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "-b", "main"]);
    run(root, &["config", "user.email", "tests@example.com"]);
    run(root, &["config", "user.name", "BranchSense Tests"]);
    std::fs::create_dir(root.join("src")).expect("source directory");
    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { public void process() {} }\n",
    )
    .expect("service source");
    std::fs::write(
        root.join("src/PaymentController.java"),
        "package sample; public class PaymentController { public void submit() {} }\n",
    )
    .expect("controller source");
    run(root, &["add", "."]);
    run(root, &["commit", "-m", "initial semantic area"]);

    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { public void process(String currency) {} }\n",
    )
    .expect("service change");
    commit(root, "change service", "2020-01-02T00:00:00Z");

    std::fs::write(
        root.join("src/PaymentController.java"),
        "package sample; public class PaymentController { public void submit(String request) {} }\n",
    )
    .expect("controller change");
    commit(root, "change controller", "2020-01-03T00:00:00Z");

    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { public void process(int currency) {} }\n",
    )
    .expect("service co-change");
    std::fs::write(
        root.join("src/PaymentController.java"),
        "package sample; public class PaymentController { public void submit(int request) {} }\n",
    )
    .expect("controller co-change");
    commit(root, "change payment area together", "2020-01-04T00:00:00Z");

    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { public void process(long currency) {} }\n",
    )
    .expect("service recent change");
    commit(root, "change service again", "2020-01-05T00:00:00Z");

    std::fs::write(
        root.join("src/Unrelated.java"),
        "package sample; public class Unrelated { public void export() {} }\n",
    )
    .expect("unrelated source");
    run(root, &["add", "."]);
    let status = Command::new("git")
        .args(["commit", "-m", "unrelated change"])
        .current_dir(root)
        .env("GIT_AUTHOR_DATE", "2020-01-06T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2020-01-06T00:00:00Z")
        .status()
        .expect("git runs");
    assert!(status.success());
    directory
}

#[test]
fn bounded_history_reports_frequency_recency_and_cochange() {
    let directory = repository();
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    let revision = repository.resolve("main").expect("resolve main");
    let analyzer = HistoricalAnalyzer::new();
    let signals = analyzer
        .analyze(&repository, &revision, HistoricalOptions::new(6))
        .expect("historical analysis");

    assert_eq!(signals.analysis_revision(), revision.commit_id());
    assert_eq!(signals.commits_analyzed(), 6);
    let service = signals
        .change_frequency()
        .iter()
        .find(|signal| signal.symbol().qualified_name() == "sample.PaymentService.process(long)")
        .expect("service frequency");
    assert_eq!(service.total_changes(), 1);
    let recent = signals
        .recency()
        .iter()
        .find(|signal| signal.symbol().qualified_name() == "sample.PaymentService.process(long)")
        .expect("service recency");
    assert_eq!(recent.age_in_commits(), 1);
    assert!(signals.symbol_co_change().iter().any(|signal| {
        signal.left().qualified_name().starts_with("sample.PaymentController.submit(")
            && signal.right().qualified_name().starts_with("sample.PaymentService.process(")
            && signal.co_change_count() >= 1
    }));
    assert!(signals.file_co_change().iter().any(|signal| {
        signal.left().ends_with("PaymentController.java")
            && signal.right().ends_with("PaymentService.java")
            && signal.co_change_count() >= 2
    }));
}

#[test]
fn bounded_windows_are_deterministic_and_read_only() {
    let directory = repository();
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    let revision = repository.resolve("main").expect("resolve main");
    let analyzer = HistoricalAnalyzer::new();
    let first = analyzer
        .analyze(&repository, &revision, HistoricalOptions::new(3))
        .expect("first analysis");
    let second = analyzer
        .analyze(&repository, &revision, HistoricalOptions::new(3))
        .expect("second analysis");
    assert_eq!(first, second);
    assert_eq!(first.commits_analyzed(), 3);
    assert_eq!(first.state(), EvidenceState::Truncated);
    assert_eq!(repository.resolve("main").expect("unchanged main"), revision);
    let json = serde_json::to_vec(&first).expect("serialize signals");
    let decoded: branchsense_history::HistoricalSignals =
        serde_json::from_slice(&json).expect("deserialize signals");
    assert_eq!(decoded, first);
}

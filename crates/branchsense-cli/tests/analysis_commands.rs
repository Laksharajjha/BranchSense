//! Offline integration coverage for analytical CLI commands.

use std::process::Command;

use tempfile::TempDir;

fn git(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git runs");
    assert!(status.success(), "git command failed: {args:?}");
}

fn commit(root: &std::path::Path, message: &str, email: &str) {
    let status = Command::new("git")
        .args(["commit", "-am", message])
        .current_dir(root)
        .env("GIT_AUTHOR_NAME", "CLI Fixture")
        .env("GIT_AUTHOR_EMAIL", email)
        .env("GIT_COMMITTER_NAME", "CLI Fixture")
        .env("GIT_COMMITTER_EMAIL", email)
        .status()
        .expect("git runs");
    assert!(status.success(), "commit failed: {message}");
}

fn fixture() -> TempDir {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "CLI Fixture"]);
    git(root, &["config", "user.email", "fixture@example.com"]);
    std::fs::create_dir(root.join("src")).expect("source directory");
    std::fs::write(root.join("src/Payment.java"), "class Payment { void process() {} }\n")
        .expect("base source");
    git(root, &["add", "."]);
    commit(root, "base", "base@example.com");
    git(root, &["branch", "feature/a"]);
    git(root, &["branch", "feature/b"]);
    std::fs::write(root.join("src/Payment.java"), "class Payment { void process(int value) {} }\n")
        .expect("main source");
    commit(root, "main change", "main@example.com");
    git(root, &["checkout", "feature/a"]);
    std::fs::write(
        root.join("src/Payment.java"),
        "class Payment { void process(String value) {} }\n",
    )
    .expect("branch A source");
    commit(root, "branch A change", "branch-a@example.com");
    git(root, &["checkout", "feature/b"]);
    std::fs::write(
        root.join("src/Payment.java"),
        "class Payment { void process(long value) {} }\n",
    )
    .expect("branch B source");
    commit(root, "branch B change", "branch-b@example.com");
    git(root, &["checkout", "main"]);
    directory
}

fn run_cli(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_branchsense"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("BranchSense CLI runs")
}

#[test]
fn branch_analysis_commands_run_without_network_or_checkout_mutation() {
    let directory = fixture();
    let root = directory.path();
    for args in [
        vec!["diff", "--repo", ".", "--before", "main", "--after", "feature/a"],
        vec!["impact", "--repo", ".", "--before", "main", "--after", "feature/a"],
        vec![
            "overlap",
            "--repo",
            ".",
            "--base",
            "HEAD~1",
            "--branch-a",
            "feature/a",
            "--branch-b",
            "feature/b",
        ],
        vec![
            "analyze",
            "--repo",
            ".",
            "--base",
            "HEAD~1",
            "--branch-a",
            "feature/a",
            "--branch-b",
            "feature/b",
        ],
    ] {
        let output = run_cli(root, &args);
        assert!(output.status.success(), "command failed: {args:?}: {output:?}");
    }
}

#[test]
fn history_reports_truncation_and_ownership_can_redact_emails() {
    let directory = fixture();
    let root = directory.path();
    let history = run_cli(
        root,
        &["history", "--repo", ".", "--revision", "main", "--max-commits", "1", "--json"],
    );
    assert!(history.status.success());
    let history_text = String::from_utf8(history.stdout).expect("UTF-8 JSON");
    assert!(history_text.contains("Truncated"));

    let ownership = run_cli(
        root,
        &[
            "ownership",
            "--repo",
            ".",
            "--revision",
            "main",
            "--max-commits",
            "3",
            "--json",
            "--redact-identities",
        ],
    );
    assert!(ownership.status.success());
    let ownership_text = String::from_utf8(ownership.stdout).expect("UTF-8 JSON");
    assert!(!ownership_text.contains("@example.com"));
    assert!(ownership_text.contains("[redacted]"));
}

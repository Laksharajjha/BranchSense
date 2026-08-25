//! Real Git-history coverage for semantic overlap analysis.
#![allow(missing_docs)]

use std::process::Command;

use branchsense_diff::SemanticDiffer;
use branchsense_git::{GitRepository, GitSnapshotIndexer, MergeBaseResult};
use branchsense_impact::ImpactAnalyzer;
use branchsense_overlap::{OverlapKind, SemanticOverlapAnalyzer};
use tempfile::{TempDir, tempdir};

fn run(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git runs");
    assert!(status.success(), "git command failed: {args:?}");
}

fn output(root: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git").args(args).current_dir(root).output().expect("git runs");
    assert!(output.status.success(), "git command failed: {args:?}");
    String::from_utf8(output.stdout).expect("git output is utf8")
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
    .expect("base source");
    std::fs::write(
        root.join("src/Checkout.java"),
        "package sample; public class Checkout { public void submit() { new PaymentService().process(); } }\n",
    )
    .expect("base source");
    run(root, &["add", "."]);
    run(root, &["commit", "-m", "base"]);
    run(root, &["branch", "feature/payment"]);
    run(root, &["checkout", "feature/payment"]);
    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { public void process(String currency) {} }\n",
    )
    .expect("payment change");
    run(root, &["commit", "-am", "change payment API"]);
    run(root, &["checkout", "main"]);
    run(root, &["branch", "feature/checkout"]);
    run(root, &["checkout", "feature/checkout"]);
    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { private String gateway; public void process() {} }\n",
    )
    .expect("checkout change");
    run(root, &["commit", "-am", "change checkout"]);
    run(root, &["checkout", "main"]);
    directory
}

#[test]
fn analyzes_direct_and_impact_overlap_from_common_base() {
    let directory = repository();
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    let branch_a = repository.resolve("feature/payment").expect("resolve branch A");
    let branch_b = repository.resolve("feature/checkout").expect("resolve branch B");
    let base = match repository.merge_bases(&branch_a, &branch_b).expect("merge base") {
        MergeBaseResult::Single(base) => base,
        result => panic!("unexpected merge-base result: {result:?}"),
    };
    let indexer = GitSnapshotIndexer::default();
    let base_snapshot = indexer.index_revision(&repository, &base, None).expect("index base");
    let snapshot_a = indexer.index_revision(&repository, &branch_a, None).expect("index A");
    let snapshot_b = indexer.index_revision(&repository, &branch_b, None).expect("index B");
    let differ = SemanticDiffer::new();
    let diff_a = differ.diff_git(&base_snapshot, &snapshot_a);
    let diff_b = differ.diff_git(&base_snapshot, &snapshot_b);
    let analyzer = ImpactAnalyzer::new();
    let impact_a = analyzer.analyze(&diff_a, base_snapshot.semantic(), snapshot_a.semantic());
    let impact_b = analyzer.analyze(&diff_b, base_snapshot.semantic(), snapshot_b.semantic());
    let overlaps = SemanticOverlapAnalyzer::new().analyze(&diff_a, &impact_a, &diff_b, &impact_b);

    assert!(overlaps.statistics().branch_a_changed() > 0);
    assert!(overlaps.statistics().branch_b_changed() > 0);
    assert!(overlaps.entries().iter().any(|entry| {
        matches!(entry.explanation().kind(), OverlapKind::DirectChange | OverlapKind::ImpactChange)
    }));
    assert!(!output(directory.path(), &["rev-parse", "main"]).is_empty());
}

//! End-to-end Git, snapshot, diff, impact, overlap, and collision coverage.
#![allow(missing_docs)]

use std::process::Command;

use branchsense_collision::{CollisionAnalyzer, CollisionFactorKind};
use branchsense_diff::SemanticDiffer;
use branchsense_git::{GitRepository, GitSnapshotIndexer, MergeBaseResult};
use branchsense_impact::ImpactAnalyzer;
use branchsense_overlap::SemanticOverlapAnalyzer;
use tempfile::{TempDir, tempdir};

fn run(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git runs");
    assert!(status.success(), "git command failed: {args:?}");
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
    run(root, &["add", "."]);
    run(root, &["commit", "-m", "base"]);
    run(root, &["branch", "feature/a"]);
    run(root, &["checkout", "feature/a"]);
    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { public void process() { System.out.println(\"a\"); } }\n",
    )
    .expect("branch A source");
    run(root, &["commit", "-am", "change process"]);
    run(root, &["checkout", "main"]);
    run(root, &["branch", "feature/b"]);
    run(root, &["checkout", "feature/b"]);
    std::fs::write(
        root.join("src/PaymentService.java"),
        "package sample; public class PaymentService { private String gateway; public void process() {} }\n",
    )
    .expect("branch B source");
    run(root, &["commit", "-am", "add gateway"]);
    run(root, &["checkout", "main"]);
    directory
}

#[test]
fn git_pipeline_produces_high_direct_collision_evidence() {
    let directory = repository();
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    let branch_a = repository.resolve("feature/a").expect("branch A");
    let branch_b = repository.resolve("feature/b").expect("branch B");
    let base = match repository.merge_bases(&branch_a, &branch_b).expect("merge base") {
        MergeBaseResult::Single(base) => base,
        result => panic!("unexpected merge base: {result:?}"),
    };
    let indexer = GitSnapshotIndexer::default();
    let base_snapshot = indexer.index_revision(&repository, &base, None).expect("base snapshot");
    let snapshot_a = indexer.index_revision(&repository, &branch_a, None).expect("A snapshot");
    let snapshot_b = indexer.index_revision(&repository, &branch_b, None).expect("B snapshot");
    let differ = SemanticDiffer::new();
    let diff_a = differ.diff_git(&base_snapshot, &snapshot_a);
    let diff_b = differ.diff_git(&base_snapshot, &snapshot_b);
    let analyzer = ImpactAnalyzer::new();
    let impact_a = analyzer.analyze(&diff_a, base_snapshot.semantic(), snapshot_a.semantic());
    let impact_b = analyzer.analyze(&diff_b, base_snapshot.semantic(), snapshot_b.semantic());
    let overlaps = SemanticOverlapAnalyzer::new().analyze(&diff_a, &impact_a, &diff_b, &impact_b);
    let assessment = CollisionAnalyzer::new().analyze(&overlaps);

    assert_eq!(diff_a.evidence().provenance().repository_id(), Some(repository.identity().id()));
    assert_eq!(impact_a.evidence().provenance(), diff_a.evidence().provenance());
    assert_eq!(overlaps.evidence().provenance().branch_a_revision_id(), Some(branch_a.id()));
    assert_eq!(assessment.evidence().provenance(), overlaps.evidence().provenance());
    assert!(assessment.evidence_score() >= 80);
    assert!(
        assessment
            .explanations()
            .iter()
            .any(|explanation| { explanation.factor() == CollisionFactorKind::SameSymbolChanged })
    );
}

#![allow(missing_docs)]

use std::{fs, process::Command};

use branchsense_diff::SemanticDiffer;
use branchsense_git::{GitRepository, GitSnapshotIndexer};
use branchsense_impact::{ImpactAnalyzer, ImpactKind};

fn run(root: &std::path::Path, args: &[&str]) {
    let status =
        Command::new("git").args(args).current_dir(root).status().expect("git is available");
    assert!(status.success(), "git command failed: {args:?}");
}

#[test]
fn git_revision_to_impact_pipeline_reports_callers() {
    let root = tempfile::tempdir().expect("repository");
    run(root.path(), &["init", "-b", "main"]);
    run(root.path(), &["config", "user.name", "BranchSense Test"]);
    run(root.path(), &["config", "user.email", "test@branchsense.invalid"]);
    fs::create_dir_all(root.path().join("src")).expect("source directory");
    fs::write(
        root.path().join("src/Payment.java"),
        "package payment; class PaymentService { void process(User user) {} }\nclass PaymentController { void submit() { payment.PaymentService.process(null); } }\nclass OrderService { void checkout() { payment.PaymentController.submit(); } }",
    )
    .expect("base source");
    run(root.path(), &["add", "."]);
    run(root.path(), &["commit", "-m", "base"]);
    run(root.path(), &["branch", "feature"]);
    run(root.path(), &["checkout", "feature"]);
    fs::write(
        root.path().join("src/Payment.java"),
        "package payment; class PaymentService { void process(User user, Currency currency) {} }\nclass PaymentController { void submit() { payment.PaymentService.process(null, null); } }\nclass OrderService { void checkout() { payment.PaymentController.submit(); } }",
    )
    .expect("feature source");
    run(root.path(), &["commit", "-am", "change payment signature"]);
    run(root.path(), &["checkout", "main"]);

    let repository = GitRepository::discover(root.path()).expect("repository");
    let before_revision = repository.resolve("main").expect("main revision");
    let after_revision = repository.resolve("feature").expect("feature revision");
    let indexer = GitSnapshotIndexer::default();
    let before =
        indexer.index_revision(&repository, &before_revision, None).expect("before snapshot");
    let after = indexer.index_revision(&repository, &after_revision, None).expect("after snapshot");
    let diff = SemanticDiffer::new().diff_git(&before, &after);
    let impacts = ImpactAnalyzer::new().analyze(&diff, before.semantic(), after.semantic());

    assert!(impacts.entries().iter().any(|entry| {
        entry.impacted_symbol().as_str().contains("PaymentController.submit")
            && entry
                .causes()
                .iter()
                .any(|cause| cause.explanation().kind() == ImpactKind::SignatureConsumer)
    }));
    assert!(impacts.entries().iter().any(|entry| {
        entry.impacted_symbol().as_str().contains("OrderService.checkout")
            && entry
                .causes()
                .iter()
                .any(|cause| cause.explanation().kind() == ImpactKind::TransitiveCaller)
    }));
}

use std::fs;

use branchsense_diff::SemanticDiffer;
use branchsense_index::{IndexOptions, RepositoryIndex};

use super::{ImpactAnalyzer, ImpactKind, ImpactOptions};

fn snapshots(
    source: &str,
) -> (branchsense_index::SemanticIndexSnapshot, branchsense_index::SemanticIndexSnapshot) {
    let root = tempfile::tempdir().expect("repository");
    fs::create_dir_all(root.path().join("src")).expect("source directory");
    fs::write(root.path().join("src/PaymentService.java"), source).expect("before source");
    let before = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("before index")
        .into_parts()
        .0;
    fs::write(root.path().join("src/PaymentService.java"), "package payment; class PaymentService { void process(User user, Currency currency) {} }\nclass PaymentController { void submit() { payment.PaymentService.process(null, null); } }\nclass OrderService { void checkout() { payment.PaymentController.submit(); } }").expect("after source");
    let after = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("after index")
        .into_parts()
        .0;
    (before, after)
}

#[test]
fn finds_direct_and_transitive_callers() {
    let (before, after) = snapshots(
        "package payment; class PaymentService { void process(User user) {} }\nclass PaymentController { void submit() { payment.PaymentService.process(null); } }\nclass OrderService { void checkout() { payment.PaymentController.submit(); } }",
    );
    let diff = SemanticDiffer::new().diff(&before, &after);
    let impacts = ImpactAnalyzer::new().analyze(&diff, &before, &after);
    assert!(impacts.entries().iter().any(|entry| {
        entry
            .causes()
            .iter()
            .any(|cause| cause.explanation().kind() == ImpactKind::SignatureConsumer)
    }));
    assert!(impacts.entries().iter().any(|entry| {
        entry
            .causes()
            .iter()
            .any(|cause| cause.explanation().kind() == ImpactKind::TransitiveCaller)
    }));
}

#[test]
fn bounds_are_reported_and_results_are_deterministic() {
    let (before, after) = snapshots(
        "package payment; class PaymentService { void process(User user) {} }\nclass PaymentController { void submit() { payment.PaymentService.process(null); } }\nclass OrderService { void checkout() { payment.PaymentController.submit(); } }",
    );
    let diff = SemanticDiffer::new().diff(&before, &after);
    let analyzer = ImpactAnalyzer::with_options(ImpactOptions::new(1, 1));
    let first = analyzer.analyze(&diff, &before, &after);
    let second = analyzer.analyze(&diff, &before, &after);
    assert_eq!(first, second);
    assert!(first.statistics().truncated());
    assert!(first.statistics().max_depth() <= 1);
}

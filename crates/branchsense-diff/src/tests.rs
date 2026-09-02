use std::fs;

use branchsense_index::{IndexOptions, RepositoryIndex};

use branchsense_semantic::EvidenceState;

use crate::{ChangeKind, SemanticDiffer, SymbolChangeReason};

fn repository(source: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary repository");
    fs::create_dir_all(root.path().join("src")).expect("source directory");
    fs::write(root.path().join("src/PaymentService.java"), source).expect("Java source");
    root
}

fn snapshot(root: &tempfile::TempDir) -> branchsense_index::SemanticIndexSnapshot {
    RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("repository indexes")
        .into_parts()
        .0
}

#[test]
fn empty_snapshots_have_no_changes() {
    let before_root = tempfile::tempdir().expect("before repository");
    let after_root = tempfile::tempdir().expect("after repository");
    let before = snapshot(&before_root);
    let after = snapshot(&after_root);
    let diff = SemanticDiffer::new().diff(&before, &after);

    assert!(diff.is_empty());
    assert_eq!(diff.statistics().documents_unchanged(), 0);
    assert!(diff.facts().is_empty());
}

#[test]
fn real_pipeline_detects_method_signature_and_return_changes() {
    let root = repository(
        "package payment; public class PaymentService { public Payment process(User user) { return null; } }",
    );
    let before = snapshot(&root);
    fs::write(
        root.path().join("src/PaymentService.java"),
        "package payment; public class PaymentService { public Invoice process(User user, Currency currency) { return null; } }",
    )
    .expect("changed Java source");
    let after = snapshot(&root);
    let diff = SemanticDiffer::new().diff(&before, &after);

    assert_eq!(diff.statistics().documents_modified(), 1);
    assert!(diff.statistics().symbols_modified() >= 1);
    let method = diff
        .symbols()
        .iter()
        .find(|change| {
            change.kind() == ChangeKind::Modified
                && change
                    .after()
                    .or(change.before())
                    .and_then(|definition| definition.qualified_name())
                    .is_some_and(|name| name.as_str().contains("process"))
        })
        .expect("modified method");
    assert!(method.reasons().contains(&SymbolChangeReason::MethodSignatureChanged));
    assert!(method.reasons().contains(&SymbolChangeReason::ParameterAdded));
    assert!(method.reasons().contains(&SymbolChangeReason::ReturnTypeChanged));
}

#[test]
fn added_and_removed_documents_are_reported() {
    let root = repository("package payment; public class PaymentService {} ");
    let before = snapshot(&root);
    fs::write(root.path().join("src/PaymentValidator.java"), "class PaymentValidator {}")
        .expect("added Java source");
    let added = snapshot(&root);
    let added_diff = SemanticDiffer::new().diff(&before, &added);
    assert_eq!(added_diff.statistics().documents_added(), 1);

    fs::remove_file(root.path().join("src/PaymentService.java")).expect("removed Java source");
    let removed = snapshot(&root);
    let removed_diff = SemanticDiffer::new().diff(&added, &removed);
    assert_eq!(removed_diff.statistics().documents_removed(), 1);
}

#[test]
fn relationship_changes_are_reported_and_ordered() {
    let root = repository(
        "package payment; import payment.User; public class PaymentService { private User user; }",
    );
    let before = snapshot(&root);
    fs::write(
        root.path().join("src/PaymentService.java"),
        "package payment; import payment.Account; public class PaymentService { private Account account; }",
    )
    .expect("changed relationship source");
    let after = snapshot(&root);
    let first = SemanticDiffer::new().diff(&before, &after);
    let second = SemanticDiffer::new().diff(&before, &after);

    assert_eq!(first, second);
    assert!(first.statistics().relationships_added() > 0);
    assert!(first.statistics().relationships_removed() > 0);
    assert!(
        first
            .relationships()
            .windows(2)
            .all(|window| { window[0].fact().id() <= window[1].fact().id() })
    );
}

#[test]
fn identical_snapshot_comparison_is_unchanged() {
    let root = repository("package payment; public class PaymentService {} ");
    let snapshot = snapshot(&root);
    let diff = SemanticDiffer::new().diff(&snapshot, &snapshot);

    assert!(diff.is_empty());
    assert_eq!(diff.statistics().documents_unchanged(), 1);
    assert!(diff.symbols().iter().all(|change| change.kind() == ChangeKind::Unchanged));
    assert_eq!(snapshot.graph().statistics().documents(), 1);
}

#[test]
fn type_relations_are_classified() {
    let root = repository(
        "/** service */\npackage payment;\ninterface Payable {}\nclass Base {}\npublic class PaymentService extends Base implements Payable {\n    public void process() {}\n}",
    );
    let before = snapshot(&root);
    fs::write(
        root.path().join("src/PaymentService.java"),
        "package payment;\ninterface Auditable {}\nclass OtherBase {}\npublic class PaymentService extends OtherBase implements Auditable {\n    private final void process() {}\n}",
    )
    .expect("changed declaration source");
    let after = snapshot(&root);
    let diff = SemanticDiffer::new().diff(&before, &after);

    let service = diff
        .symbols()
        .iter()
        .find(|change| {
            change.kind() == ChangeKind::Modified
                && change
                    .after()
                    .or(change.before())
                    .and_then(|definition| definition.qualified_name())
                    .is_some_and(|name| name.as_str() == "payment.PaymentService")
        })
        .expect("modified service declaration");
    assert!(service.reasons().contains(&SymbolChangeReason::SuperclassChanged));
    assert!(service.reasons().contains(&SymbolChangeReason::InterfaceAdded));
    assert!(service.reasons().contains(&SymbolChangeReason::InterfaceRemoved));
}

#[test]
fn unresolved_reference_state_is_preserved_by_diff() {
    let root = repository(
        "package payment; public class PaymentService { public void process() { missing(); } }",
    );
    let before = snapshot(&root);
    fs::write(
        root.path().join("src/PaymentService.java"),
        "package payment; public class PaymentService { public void process() { missing(); changed(); } }",
    )
    .expect("changed source");
    let after = snapshot(&root);
    let diff = SemanticDiffer::new().diff(&before, &after);

    assert_eq!(diff.evidence().state(), EvidenceState::Unresolved);
    assert_eq!(diff.evidence().completeness().semantic(), EvidenceState::Unresolved);
}

use branchsense_core::{
    DocumentId, Location, Name, Position, ProjectId, QualifiedName, Range, RepositoryId,
    RevisionId, SymbolId, Visibility, WorkspaceId,
};

use crate::{
    AnalysisProvenance, ContentHash, Documentation, EvidenceCompleteness, EvidenceEnvelope,
    EvidenceIdentity, EvidenceKind, EvidenceLink, EvidenceRelation, EvidenceState, FactDelta,
    FactId, FactProvenance, FactSnapshot, IdentityMatch, ProducerIdentity, ResolutionState,
    SemanticEntityIdentity, SemanticFact, SemanticFactRecord, SemanticFactSet, SnapshotIdentity,
    SymbolDefinition, SymbolKind, SymbolReference,
};

fn location() -> Location {
    let document = DocumentId::new("src/Payment.java").expect("document ID");
    let range = Range::new(Position::new(0, 0, 0), Position::new(0, 7, 7)).expect("ordered range");
    Location::new(document, range)
}

fn definition() -> SymbolDefinition {
    SymbolDefinition::new(
        SymbolId::new("symbol:payment").expect("symbol ID"),
        SymbolKind::Type,
        Name::new("Payment").expect("name"),
        location(),
    )
    .with_qualified_name(QualifiedName::new("billing.Payment").expect("qualified name"))
    .with_visibility(Visibility::Public)
    .with_documentation(Documentation::new("A payment value.").expect("documentation"))
}

#[test]
fn definition_preserves_identity_and_metadata() {
    let value = definition();

    assert_eq!(value.id().as_str(), "symbol:payment");
    assert_eq!(value.name().as_str(), "Payment");
    assert_eq!(value.qualified_name().expect("qualified name").as_str(), "billing.Payment");
    assert_eq!(value.visibility(), Visibility::Public);
    assert_eq!(value.documentation().expect("documentation").as_str(), "A payment value.");
}

#[test]
fn fact_records_and_sets_are_immutable_transport_values() {
    let fact = SemanticFactRecord::new(
        FactId::new("fact:payment-definition").expect("fact ID"),
        SemanticFact::Definition(definition()),
    );
    let set = SemanticFactSet::new(vec![fact.clone()]);

    assert_eq!(set.len(), 1);
    assert!(!set.is_empty());
    assert_eq!(set.facts()[0], fact);
}

#[test]
fn unresolved_references_retain_names_before_resolution() {
    let reference = SymbolReference::unresolved(
        QualifiedName::new("billing.Payment.validate").expect("qualified name"),
    );

    assert_eq!(reference.name().as_str(), "billing.Payment.validate");
    assert!(reference.resolved_symbol().is_none());
}

#[test]
fn semantic_facts_round_trip_through_json() {
    let fact = SemanticFactRecord::new(
        FactId::new("fact:payment-definition").expect("fact ID"),
        SemanticFact::Definition(definition()),
    );
    let encoded = serde_json::to_string(&fact).expect("serialization succeeds");
    let decoded: SemanticFactRecord =
        serde_json::from_str(&encoded).expect("deserialization succeeds");

    assert_eq!(decoded, fact);
}

#[test]
fn empty_values_are_rejected() {
    assert!(FactId::new(" ").is_err());
    assert!(Documentation::new("").is_err());
}

fn provenance(document: &str, revision: &str) -> FactProvenance {
    FactProvenance::new(
        RepositoryId::new("repo:branchsense").expect("repository ID"),
        WorkspaceId::new("workspace:branchsense").expect("workspace ID"),
        DocumentId::new(document).expect("document ID"),
        RevisionId::new(revision).expect("revision ID"),
        ContentHash::new("sha256:abc").expect("content hash"),
        ProducerIdentity::new("branchsense-test", "1").expect("producer identity"),
    )
    .with_project(ProjectId::new("project:core").expect("project ID"))
}

fn record(id: &str, name: &str) -> SemanticFactRecord {
    let definition = SymbolDefinition::new(
        SymbolId::new(format!("symbol:{id}")).expect("symbol ID"),
        SymbolKind::Type,
        Name::new(name).expect("name"),
        location(),
    );
    SemanticFactRecord::new(
        FactId::new(format!("fact:{id}")).expect("fact ID"),
        SemanticFact::Definition(definition),
    )
}

#[test]
fn repository_and_document_id_values_are_distinct() {
    let first = RepositoryId::new("repo:one").expect("repository ID");
    let second = RepositoryId::new("repo:two").expect("repository ID");
    assert_ne!(first, second);
    assert_eq!(DocumentId::new("src/Main.java").expect("document ID").as_str(), "src/Main.java");
}

#[test]
fn provenance_round_trips_and_separates_revisions() {
    let first = provenance("src/Main.java", "revision:one");
    let second = provenance("src/Main.java", "revision:two");
    assert_ne!(first.revision_id(), second.revision_id());
    let encoded = serde_json::to_string(&first).expect("serialization succeeds");
    assert_eq!(
        serde_json::from_str::<FactProvenance>(&encoded).expect("deserialization succeeds"),
        first
    );
}

#[test]
fn fact_delta_distinguishes_added_removed_updated_and_unchanged() {
    let old = SemanticFactSet::new(vec![record("same", "Same"), record("removed", "Removed")]);
    let new = SemanticFactSet::new(vec![record("same", "Changed"), record("added", "Added")]);
    let delta = FactDelta::between(
        DocumentId::new("src/Main.java").expect("document ID"),
        RevisionId::new("revision:two").expect("revision ID"),
        Some(&old),
        &new,
    );

    assert_eq!(delta.added().len(), 1);
    assert_eq!(delta.removed().len(), 1);
    assert_eq!(delta.updated().len(), 1);
    assert_eq!(delta.changed_count(), 3);
    assert!(!delta.is_empty());

    let unchanged = FactDelta::between(
        DocumentId::new("src/Main.java").expect("document ID"),
        RevisionId::new("revision:two").expect("revision ID"),
        Some(&new),
        &new,
    );
    assert!(unchanged.is_empty());
}

#[test]
fn canonical_identity_is_revision_independent_but_conservative() {
    let identity = SemanticEntityIdentity::from_definition(&definition()).expect("identity");
    assert_eq!(identity.document().to_str(), Some("src/Payment.java"));
    assert_eq!(identity.kind(), SymbolKind::Type);
    assert_eq!(identity.qualified_name(), "billing.Payment");

    let encoded = serde_json::to_string(&identity).expect("serialization succeeds");
    assert_eq!(
        serde_json::from_str::<SemanticEntityIdentity>(&encoded).expect("deserialization succeeds"),
        identity
    );
    assert!(matches!(IdentityMatch::Matched(identity), IdentityMatch::Matched(_)));
}

#[test]
fn canonical_identity_preserves_overload_signatures() {
    let one = SymbolDefinition::new(
        SymbolId::new("symbol:one").expect("symbol ID"),
        SymbolKind::Method,
        Name::new("foo").expect("name"),
        location(),
    )
    .with_qualified_name(QualifiedName::new("Payment.foo(String)").expect("qualified name"));
    let two = SymbolDefinition::new(
        SymbolId::new("symbol:two").expect("symbol ID"),
        SymbolKind::Method,
        Name::new("foo").expect("name"),
        location(),
    )
    .with_qualified_name(QualifiedName::new("Payment.foo(String, int)").expect("qualified name"));

    assert_ne!(
        SemanticEntityIdentity::from_definition(&one).expect("identity"),
        SemanticEntityIdentity::from_definition(&two).expect("identity")
    );
}

#[test]
fn evidence_states_distinguish_empty_from_inconclusive_analysis() {
    assert!(EvidenceState::Observed.is_observed());
    assert!(EvidenceState::NoEvidence.is_no_evidence());
    assert!(!EvidenceState::NoEvidence.is_inconclusive());
    for state in [
        EvidenceState::Unavailable,
        EvidenceState::Unsupported,
        EvidenceState::Unresolved,
        EvidenceState::Ambiguous,
        EvidenceState::Truncated,
        EvidenceState::Failed,
    ] {
        assert!(state.is_inconclusive());
        let encoded = serde_json::to_string(&state).expect("serialization succeeds");
        assert_eq!(
            serde_json::from_str::<EvidenceState>(&encoded).expect("deserialization succeeds"),
            state
        );
    }
}

#[test]
fn document_deletion_removes_every_fact() {
    let facts = SemanticFactSet::new(vec![record("one", "One"), record("two", "Two")]);
    let delta = FactDelta::delete(
        DocumentId::new("src/Main.java").expect("document ID"),
        RevisionId::new("revision:two").expect("revision ID"),
        &facts,
    );
    assert_eq!(delta.removed().len(), 2);
    assert!(delta.added().is_empty());
}

#[test]
fn references_keep_resolution_states_explicit() {
    let name = QualifiedName::new("billing.Payment").expect("qualified name");
    let unresolved = SymbolReference::unresolved(name.clone());
    assert_eq!(unresolved.resolution(), &ResolutionState::Unresolved);

    let symbol = SymbolId::new("symbol:payment").expect("symbol ID");
    let resolved = SymbolReference::resolved(name.clone(), symbol.clone());
    assert_eq!(resolved.resolved_symbol(), Some(&symbol));
    assert_eq!(resolved.resolution(), &ResolutionState::Resolved(symbol));

    let ambiguous = SymbolReference::ambiguous(name.clone(), vec![SymbolId::new("a").expect("ID")]);
    assert!(matches!(ambiguous.resolution(), ResolutionState::Ambiguous(_)));

    let external = SymbolReference::external(
        name.clone(),
        crate::ExternalSymbolId::new("java:java.lang.String").expect("external ID"),
    );
    assert!(matches!(external.resolution(), ResolutionState::External(_)));
}

#[test]
fn evidence_identity_and_provenance_are_deterministic() {
    let evidence = EvidenceIdentity::new(
        EvidenceKind::Derived,
        "symbol:payment",
        vec!["symbol:b".into(), "symbol:a".into(), "symbol:a".into()],
    );
    assert_eq!(evidence.related(), &["symbol:a".to_owned(), "symbol:b".to_owned()]);
    assert_eq!(
        serde_json::from_str::<EvidenceIdentity>(
            &serde_json::to_string(&evidence).expect("serialization succeeds")
        )
        .expect("deserialization succeeds"),
        evidence
    );

    let provenance = AnalysisProvenance::new()
        .with_repository(RepositoryId::new("repo:one").expect("repository ID"))
        .with_branches(
            RevisionId::new("revision:a").expect("revision ID"),
            RevisionId::new("revision:b").expect("revision ID"),
            RevisionId::new("revision:base").expect("revision ID"),
        )
        .with_history_window(25);
    assert_eq!(
        serde_json::from_str::<AnalysisProvenance>(
            &serde_json::to_string(&provenance).expect("serialization succeeds")
        )
        .expect("deserialization succeeds"),
        provenance
    );

    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::Truncated)
        .with_responsibility(EvidenceState::Unavailable);
    assert_eq!(completeness.semantic(), EvidenceState::Observed);
    assert_eq!(completeness.historical(), EvidenceState::Truncated);
    assert_eq!(completeness.responsibility(), EvidenceState::Unavailable);
}

#[test]
fn evidence_envelope_preserves_state_and_lineage() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let derived = EvidenceIdentity::new(EvidenceKind::Derived, "overlap:payment", Vec::new());
    let envelope = EvidenceEnvelope::new(
        EvidenceState::Observed,
        EvidenceCompleteness::new().with_semantic(EvidenceState::Observed),
        AnalysisProvenance::new(),
    )
    .with_identity(primary.clone())
    .with_identity(derived.clone())
    .with_link(EvidenceLink::new(derived, primary, EvidenceRelation::DerivedFrom));
    let decoded: EvidenceEnvelope =
        serde_json::from_str(&serde_json::to_string(&envelope).expect("serialize envelope"))
            .expect("deserialize envelope");
    assert_eq!(decoded, envelope);
    assert_eq!(envelope.lineage()[0].relation(), EvidenceRelation::DerivedFrom);
}

#[test]
fn snapshots_are_revision_pinned_and_reject_duplicate_documents() {
    let identity = SnapshotIdentity::new(
        RepositoryId::new("repo:branchsense").expect("repository ID"),
        WorkspaceId::new("workspace:branchsense").expect("workspace ID"),
        RevisionId::new("revision:one").expect("revision ID"),
    );
    let document = DocumentId::new("src/Main.java").expect("document ID");
    let facts = SemanticFactSet::new(vec![record("one", "One")])
        .with_provenance(provenance("src/Main.java", "revision:one"));
    let snapshot =
        FactSnapshot::new(identity, vec![crate::DocumentFactSet::new(document.clone(), facts)])
            .expect("unique document is valid");
    assert_eq!(snapshot.documents().len(), 1);
    assert_eq!(snapshot.identity().revision_id().as_str(), "revision:one");

    let duplicate = FactSnapshot::new(
        snapshot.identity().clone(),
        vec![
            crate::DocumentFactSet::new(document.clone(), SemanticFactSet::default()),
            crate::DocumentFactSet::new(document, SemanticFactSet::default()),
        ],
    );
    assert!(duplicate.is_err());
}

#[test]
fn snapshots_serialize_in_canonical_document_order() {
    let identity = SnapshotIdentity::new(
        RepositoryId::new("repo:branchsense").expect("repository ID"),
        WorkspaceId::new("workspace:branchsense").expect("workspace ID"),
        RevisionId::new("revision:one").expect("revision ID"),
    );
    let first = crate::DocumentFactSet::new(
        DocumentId::new("z.java").expect("document ID"),
        SemanticFactSet::default(),
    );
    let second = crate::DocumentFactSet::new(
        DocumentId::new("a.java").expect("document ID"),
        SemanticFactSet::default(),
    );
    let left = FactSnapshot::new(identity.clone(), vec![first.clone(), second.clone()])
        .expect("snapshot is valid");
    let right = FactSnapshot::new(identity, vec![second, first]).expect("snapshot is valid");

    assert_eq!(
        serde_json::to_string(&left).expect("serialization succeeds"),
        serde_json::to_string(&right).expect("serialization succeeds")
    );
}

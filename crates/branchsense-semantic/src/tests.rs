use branchsense_core::{
    DocumentId, Location, Name, Position, ProjectId, QualifiedName, Range, RepositoryId,
    RevisionId, SymbolId, Visibility, WorkspaceId,
};

use crate::{
    AbstentionDecision, AnalysisProvenance, CompletenessIssue, CompletenessScope,
    CompletenessSource, ContentHash, Documentation, EvidenceCompleteness, EvidenceEnvelope,
    EvidenceIdentity, EvidenceKind, EvidenceLedger, EvidenceLink, EvidenceRelation, EvidenceState,
    FactDelta, FactId, FactProvenance, FactSnapshot, IdentityMatch, ObservationIdentity,
    ProducerIdentity, ResolutionState, SemanticEntityIdentity, SemanticFact, SemanticFactRecord,
    SemanticFactSet, SnapshotIdentity, SymbolDefinition, SymbolKind, SymbolReference,
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
fn evidence_ledger_deduplicates_facts_but_not_relationships() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let supporting = EvidenceIdentity::new(EvidenceKind::Supporting, "history:payment", Vec::new());
    let derived = EvidenceIdentity::new(EvidenceKind::Derived, "impact:payment", Vec::new());
    let mut ledger = EvidenceLedger::new();

    assert!(ledger.insert_identity(primary.clone()));
    assert!(!ledger.insert_identity(primary.clone()));
    assert!(ledger.insert_identity(supporting.clone()));
    assert!(ledger.insert_link(EvidenceLink::new(
        derived.clone(),
        primary.clone(),
        EvidenceRelation::DerivedFrom,
    )));
    assert!(ledger.insert_link(EvidenceLink::new(
        supporting.clone(),
        primary.clone(),
        EvidenceRelation::Corroborates,
    )));
    assert_eq!(ledger.identities().count(), 2);
    assert_eq!(ledger.observations().count(), 2);
    assert!(primary.same_observation(&EvidenceIdentity::new(
        EvidenceKind::Supporting,
        "symbol:payment",
        Vec::new(),
    )));
    assert_eq!(ledger.lineage().count(), 2);

    let encoded = serde_json::to_string(&ledger).expect("serialize ledger");
    assert_eq!(
        serde_json::from_str::<EvidenceLedger>(&encoded).expect("deserialize ledger"),
        ledger
    );
}

#[test]
fn completeness_preserves_scoped_issues_and_serialization() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_issue(CompletenessIssue::new(
            EvidenceState::Unavailable,
            CompletenessScope::Document,
            CompletenessSource::Source,
            Some("src/Broken.java".into()),
            "invalid UTF-8",
        ))
        .with_issue(CompletenessIssue::new(
            EvidenceState::Ambiguous,
            CompletenessScope::AffectedSubgraph,
            CompletenessSource::Graph,
            None,
            "multiple targets",
        ));
    assert_eq!(completeness.issues().len(), 2);
    assert!(
        completeness
            .issues()
            .iter()
            .any(|issue| issue.scope() == CompletenessScope::AffectedSubgraph)
    );
    assert_eq!(
        serde_json::from_str::<EvidenceCompleteness>(
            &serde_json::to_string(&completeness).expect("serialize completeness")
        )
        .expect("deserialize completeness"),
        completeness
    );
}

#[test]
fn canonical_identity_matrix_is_conservative() {
    let make = |document: &str, kind: SymbolKind, qualified: &str, id: &str| {
        let document = DocumentId::new(document).expect("document ID");
        let location = Location::new(
            document,
            Range::new(Position::new(0, 0, 0), Position::new(0, 1, 1)).expect("range"),
        );
        SymbolDefinition::new(
            SymbolId::new(id).expect("symbol ID"),
            kind,
            Name::new("foo").expect("name"),
            location,
        )
        .with_qualified_name(QualifiedName::new(qualified).expect("qualified name"))
    };

    let foo_string = make("src/A.java", SymbolKind::Method, "a.A.foo(String)", "one");
    let foo_string_int = make("src/A.java", SymbolKind::Method, "a.A.foo(String, int)", "two");
    let foo_int = make("src/A.java", SymbolKind::Method, "a.A.foo(int)", "three");
    assert_ne!(
        SemanticEntityIdentity::from_definition(&foo_string).expect("identity"),
        SemanticEntityIdentity::from_definition(&foo_string_int).expect("identity")
    );
    assert_ne!(
        SemanticEntityIdentity::from_definition(&foo_string_int).expect("identity"),
        SemanticEntityIdentity::from_definition(&foo_int).expect("identity")
    );

    let same_revision = make("src/A.java", SymbolKind::Method, "a.A.foo(String)", "changed-id");
    assert_eq!(
        SemanticEntityIdentity::from_definition(&foo_string).expect("identity"),
        SemanticEntityIdentity::from_definition(&same_revision).expect("identity")
    );
    assert_ne!(
        SemanticEntityIdentity::from_definition(&foo_string).expect("identity"),
        SemanticEntityIdentity::from_definition(&make(
            "src/B.java",
            SymbolKind::Method,
            "a.A.foo(String)",
            "other-file",
        ))
        .expect("identity")
    );
    assert_ne!(
        SemanticEntityIdentity::from_definition(&foo_string).expect("identity"),
        SemanticEntityIdentity::from_definition(&make(
            "src/A.java",
            SymbolKind::Method,
            "b.A.foo(String)",
            "other-package",
        ))
        .expect("identity")
    );
    assert!(matches!(IdentityMatch::Unknown, IdentityMatch::Unknown));
    assert!(matches!(
        IdentityMatch::Ambiguous(vec![
            SemanticEntityIdentity::from_definition(&foo_string).expect("identity"),
            SemanticEntityIdentity::from_definition(&foo_int).expect("identity"),
        ]),
        IdentityMatch::Ambiguous(_)
    ));
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

// ── Prerequisite 1: Observation identity semantics ──────────────────────────

#[test]
fn observation_identity_is_independent_of_evidence_role() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let supporting = EvidenceIdentity::new(EvidenceKind::Supporting, "symbol:payment", Vec::new());
    let derived = EvidenceIdentity::new(EvidenceKind::Derived, "symbol:payment", Vec::new());

    // Different role-bearing identities are distinct.
    assert_ne!(primary, supporting);
    assert_ne!(primary, derived);

    // But they describe the same underlying observation.
    assert!(primary.same_observation(&supporting));
    assert!(primary.same_observation(&derived));
    assert_eq!(primary.observation(), supporting.observation());
}

#[test]
fn distinct_observations_on_same_subject_are_not_equal() {
    let a = ObservationIdentity::new("symbol:payment", Vec::new());
    let b = ObservationIdentity::new("symbol:checkout", Vec::new());
    assert_ne!(a, b);
}

#[test]
fn derived_observation_links_via_causal_ledger() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let derived = EvidenceIdentity::new(EvidenceKind::Derived, "overlap:payment", Vec::new());
    // Different subjects — they are NOT the same observation, but they can be
    // linked via DerivedFrom in the ledger.
    assert!(!primary.same_observation(&derived));

    let mut ledger = EvidenceLedger::new();
    ledger.insert_identity(primary.clone());
    ledger.insert_identity(derived.clone());
    ledger.insert_link(EvidenceLink::new(
        derived.clone(),
        primary.clone(),
        EvidenceRelation::DerivedFrom,
    ));
    assert_eq!(ledger.identities().count(), 2);
    assert_eq!(ledger.lineage().count(), 1);
    assert_eq!(
        ledger.lineage().next().map(EvidenceLink::relation),
        Some(EvidenceRelation::DerivedFrom)
    );
}

#[test]
fn corroborating_observation_remains_independent() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let corroborator =
        EvidenceIdentity::new(EvidenceKind::Supporting, "history:payment", Vec::new());

    let mut ledger = EvidenceLedger::new();
    ledger.insert_identity(primary.clone());
    ledger.insert_identity(corroborator.clone());
    ledger.insert_link(EvidenceLink::new(
        corroborator.clone(),
        primary.clone(),
        EvidenceRelation::Corroborates,
    ));

    // Corroborating evidence is distinct: two identities, two observations.
    assert_eq!(ledger.identities().count(), 2);
    assert_eq!(ledger.observations().count(), 2);
    assert_eq!(ledger.lineage().count(), 1);
    assert!(!primary.same_observation(&corroborator));
}

#[test]
fn observation_identity_is_deterministic_and_sorted() {
    let a =
        ObservationIdentity::new("symbol:payment", vec!["related:b".into(), "related:a".into()]);
    let b =
        ObservationIdentity::new("symbol:payment", vec!["related:a".into(), "related:b".into()]);
    assert_eq!(a, b);
    assert_eq!(a.related(), ["related:a", "related:b"]);
}

#[test]
fn ledger_deduplication_preserves_observations_not_roles() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let supporting = EvidenceIdentity::new(EvidenceKind::Supporting, "symbol:payment", Vec::new());

    let mut ledger = EvidenceLedger::new();
    ledger.insert_identity(primary.clone());
    ledger.insert_identity(supporting.clone());

    // Two distinct role-bearing identities, but one underlying observation.
    assert_eq!(ledger.identities().count(), 2);
    assert_eq!(ledger.observations().count(), 1);
    assert!(ledger.contains_observation(&primary.observation()));
}

#[test]
fn ledger_serialization_round_trip_preserves_observations() {
    let primary = EvidenceIdentity::new(EvidenceKind::Primary, "symbol:payment", Vec::new());
    let mut ledger = EvidenceLedger::new();
    ledger.insert_identity(primary.clone());
    let json = serde_json::to_string(&ledger).expect("serialize ledger");
    let decoded: EvidenceLedger = serde_json::from_str(&json).expect("deserialize ledger");
    assert_eq!(decoded, ledger);
    assert_eq!(decoded.observations().count(), 1);
}

// ── Prerequisite 4: Abstention rules ────────────────────────────────────────

#[test]
fn abstention_proceeds_when_all_domains_complete() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::Observed)
        .with_responsibility(EvidenceState::NoEvidence);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Proceed);
    assert!(!completeness.abstention_decision().must_abstain());
}

#[test]
fn abstention_proceeds_when_history_is_no_evidence() {
    // NoEvidence means "completed and found nothing" — not a warning.
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::NoEvidence)
        .with_responsibility(EvidenceState::NoEvidence);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Proceed);
}

#[test]
fn abstention_warns_when_history_truncated() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::Truncated);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Warn);
    assert!(!completeness.abstention_decision().must_abstain());
}

#[test]
fn abstention_warns_when_history_unsupported() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::Unsupported);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Warn);
}

#[test]
fn abstention_warns_when_history_unresolved() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::Unresolved);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Warn);
}

#[test]
fn abstention_indeterminate_when_semantic_unavailable() {
    // Semantic evidence is always required. Unavailable → must abstain.
    let completeness = EvidenceCompleteness::new().with_semantic(EvidenceState::Unavailable);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Indeterminate);
    assert!(completeness.abstention_decision().must_abstain());
}

#[test]
fn abstention_indeterminate_when_semantic_failed() {
    let completeness = EvidenceCompleteness::new().with_semantic(EvidenceState::Failed);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Indeterminate);
    assert!(completeness.abstention_decision().must_abstain());
}

#[test]
fn abstention_indeterminate_when_semantic_ambiguous() {
    let completeness = EvidenceCompleteness::new().with_semantic(EvidenceState::Ambiguous);
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Indeterminate);
}

#[test]
fn abstention_indeterminate_for_global_critical_issue() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_issue(CompletenessIssue::new(
            EvidenceState::Failed,
            CompletenessScope::Global,
            CompletenessSource::Graph,
            None,
            "graph construction failed entirely",
        ));
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Indeterminate);
}

#[test]
fn abstention_indeterminate_for_affected_subgraph_critical_issue() {
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_issue(CompletenessIssue::new(
            EvidenceState::Ambiguous,
            CompletenessScope::AffectedSubgraph,
            CompletenessSource::Extraction,
            Some("src/Payment.java".into()),
            "ambiguous symbol resolution in changed symbol",
        ));
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Indeterminate);
}

#[test]
fn abstention_warns_for_unrelated_document_failure() {
    // A parse failure in a file unrelated to the analyzed subgraph is
    // non-critical: BCS should warn but not abstain.
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_issue(CompletenessIssue::new(
            EvidenceState::Failed,
            CompletenessScope::Unrelated,
            CompletenessSource::Parsing,
            Some("src/Legacy.java".into()),
            "partial Java parsing",
        ));
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Warn);
}

#[test]
fn abstention_warns_for_document_scoped_issue() {
    // A document-level issue without subgraph classification is a warning:
    // the consumer must determine relevance.
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_issue(CompletenessIssue::new(
            EvidenceState::Unavailable,
            CompletenessScope::Document,
            CompletenessSource::Source,
            Some("src/Generated.java".into()),
            "invalid UTF-8 source",
        ));
    assert_eq!(completeness.abstention_decision(), AbstentionDecision::Warn);
}

#[test]
fn no_evidence_is_not_safety_evidence() {
    // NoEvidence != "safe"; it is a completed empty analysis.
    assert!(EvidenceState::NoEvidence.is_no_evidence());
    assert!(!EvidenceState::NoEvidence.is_inconclusive());
    assert!(!EvidenceState::NoEvidence.is_critical_for_abstention());
}

#[test]
fn unavailable_is_not_zero() {
    assert!(EvidenceState::Unavailable.is_inconclusive());
    assert!(EvidenceState::Unavailable.is_critical_for_abstention());
}

#[test]
fn unsupported_is_not_safe() {
    assert!(EvidenceState::Unsupported.is_inconclusive());
    // Unsupported is a warning, not critical by itself in non-semantic domain.
    assert!(!EvidenceState::Unsupported.is_critical_for_abstention());
}

#[test]
fn unresolved_is_not_resolved() {
    assert!(EvidenceState::Unresolved.is_inconclusive());
    assert!(!EvidenceState::Unresolved.is_critical_for_abstention());
}

#[test]
fn ambiguous_is_not_resolved() {
    assert!(EvidenceState::Ambiguous.is_inconclusive());
    assert!(EvidenceState::Ambiguous.is_critical_for_abstention());
}

#[test]
fn truncated_is_not_complete() {
    assert!(EvidenceState::Truncated.is_inconclusive());
    assert!(!EvidenceState::Truncated.is_critical_for_abstention());
}

#[test]
fn failed_is_not_trustworthy() {
    assert!(EvidenceState::Failed.is_inconclusive());
    assert!(EvidenceState::Failed.is_critical_for_abstention());
}

// ── Prerequisite 6: Evidence strength ≠ probability ─────────────────────────

#[test]
fn evidence_completeness_has_no_probability_field() {
    // Structural test: EvidenceCompleteness serializes to JSON that contains
    // no "probability" or "confidence" key, preventing accidental conflation.
    let completeness = EvidenceCompleteness::new()
        .with_semantic(EvidenceState::Observed)
        .with_historical(EvidenceState::Observed)
        .with_responsibility(EvidenceState::NoEvidence);
    let json = serde_json::to_string(&completeness).expect("serialize");
    assert!(!json.contains("probability"), "probability must not appear in completeness JSON");
    assert!(!json.contains("confidence"), "confidence must not appear in completeness JSON");
    assert!(!json.contains("likelihood"), "likelihood must not appear in completeness JSON");
}

#[test]
fn evidence_envelope_has_no_probability_field() {
    let envelope = EvidenceEnvelope::new(
        EvidenceState::Observed,
        EvidenceCompleteness::new().with_semantic(EvidenceState::Observed),
        AnalysisProvenance::new(),
    );
    let json = serde_json::to_string(&envelope).expect("serialize");
    assert!(!json.contains("probability"), "probability must not appear in envelope JSON");
    assert!(!json.contains("confidence"), "confidence must not appear in envelope JSON");
}

#[test]
fn abstention_decision_serializes_without_probability() {
    let decision = AbstentionDecision::Indeterminate;
    let json = serde_json::to_string(&decision).expect("serialize");
    assert!(!json.contains("probability"));
    assert_eq!(json, "\"Indeterminate\"");
}

// ── Prerequisite 3: Evaluation dataset contract ──────────────────────────────

#[test]
fn eval_record_serializes_and_preserves_schema_version() {
    use crate::{
        AnalysisProvenance, DatasetSchemaVersion, EvalOutcome, EvalRecord, EvalRepositoryIdentity,
        EvalRevision, EvidenceCompleteness, EvidenceState, PredictedOrdinalAssessment,
    };

    let record = EvalRecord::new(
        EvalRepositoryIdentity::new("repo:branchsense", Some("BranchSense")),
        EvalRevision::new("abc123"),
        EvalRevision::new("def456"),
        EvalRevision::new("ghi789"),
        EvalRevision::new("abc123"),
        EvidenceCompleteness::new()
            .with_semantic(EvidenceState::Observed)
            .with_historical(EvidenceState::Truncated),
        AnalysisProvenance::new(),
        "bcs-v1",
        "config-v1",
        EvidenceState::Observed,
    )
    .with_predicted_assessment(PredictedOrdinalAssessment::Moderate)
    .with_outcome(EvalOutcome::new().with_textual_merge_conflict(true).with_build_failure(false));

    let json = serde_json::to_string(&record).expect("serialize");
    let decoded: EvalRecord = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, record);
    assert_eq!(decoded.schema_version(), &DatasetSchemaVersion::current());
    assert_eq!(decoded.schema_version().major(), 1);
}

#[test]
fn eval_outcome_fields_are_independent() {
    use crate::EvalOutcome;

    // Textual conflict does not imply build failure.
    let outcome = EvalOutcome::new()
        .with_textual_merge_conflict(true)
        .with_build_failure(false)
        .with_test_failure(false)
        .with_semantic_integration_issue(false);
    assert_eq!(outcome.textual_merge_conflict, Some(true));
    assert_eq!(outcome.build_failure, Some(false));

    // Semantic issue independent of textual conflict.
    let semantic_only = EvalOutcome::new().with_semantic_integration_issue(true);
    assert_eq!(semantic_only.textual_merge_conflict, None);
    assert_eq!(semantic_only.semantic_integration_issue, Some(true));
}

#[test]
fn eval_record_has_no_probability_field() {
    use crate::{
        AnalysisProvenance, EvalRecord, EvalRepositoryIdentity, EvalRevision, EvidenceCompleteness,
        EvidenceState,
    };

    let record = EvalRecord::new(
        EvalRepositoryIdentity::new("repo:test", None::<String>),
        EvalRevision::new("aaa"),
        EvalRevision::new("bbb"),
        EvalRevision::new("ccc"),
        EvalRevision::new("aaa"),
        EvidenceCompleteness::new().with_semantic(EvidenceState::Observed),
        AnalysisProvenance::new(),
        "bcs-v1",
        "config-v1",
        EvidenceState::Observed,
    );

    let json = serde_json::to_string(&record).expect("serialize");
    // No raw probability or likelihood field must exist.
    assert!(!json.contains("\"probability\""), "eval record must not contain 'probability' key");
    assert!(!json.contains("\"likelihood\""), "eval record must not contain 'likelihood' key");
    // Algorithm and configuration versions must be present for reproducibility.
    assert!(json.contains("algorithm_version"));
    assert!(json.contains("configuration_version"));
    assert!(json.contains("schema_version"));
}

#[test]
fn eval_record_provenance_round_trips() {
    use branchsense_core::{RepositoryId, RevisionId};

    use crate::{
        AnalysisProvenance, EvalRecord, EvalRepositoryIdentity, EvalRevision, EvidenceCompleteness,
        EvidenceState, LabelProvenance, OutcomeConfidence,
    };

    let provenance = AnalysisProvenance::new()
        .with_repository(RepositoryId::new("repo:branchsense").expect("repository ID"))
        .with_branches(
            RevisionId::new("rev:a").expect("revision ID"),
            RevisionId::new("rev:b").expect("revision ID"),
            RevisionId::new("rev:base").expect("revision ID"),
        );

    let record = EvalRecord::new(
        EvalRepositoryIdentity::new("repo:branchsense", None::<String>),
        EvalRevision::new("base"),
        EvalRevision::new("rev-a"),
        EvalRevision::new("rev-b"),
        EvalRevision::new("base"),
        EvidenceCompleteness::new().with_semantic(EvidenceState::Observed),
        provenance.clone(),
        "bcs-v1",
        "config-v1",
        EvidenceState::Observed,
    )
    .with_outcome_confidence(OutcomeConfidence::High)
    .with_label_provenance(LabelProvenance::new(true, "automated-pipeline"));

    let json = serde_json::to_string(&record).expect("serialize");
    let decoded: EvalRecord = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, record);
    assert_eq!(decoded.provenance(), record.provenance());
    // outcome_confidence is accessible through round-trip equality.
    assert_eq!(decoded.schema_version(), record.schema_version());
}

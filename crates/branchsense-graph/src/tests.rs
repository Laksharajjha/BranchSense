use branchsense_core::{
    DocumentId, Location, Name, Position, QualifiedName, Range, RevisionId, SymbolId,
};
use branchsense_semantic::{
    CallFact, ContainsFact, FactDelta, FactId, ImportFact, ParameterFact, ReferenceFact,
    ReferenceKind, SemanticFact, SemanticFactRecord, SemanticFactSet, SymbolDefinition, SymbolKind,
    SymbolReference, TypeReference, TypeRelation, TypeRelationFact,
};

use crate::{EdgeKind, GraphNode, GraphNodeId, SemanticGraph};

fn document() -> DocumentId {
    DocumentId::new("src/Payment.java").expect("document ID")
}

fn revision(value: &str) -> RevisionId {
    RevisionId::new(value).expect("revision ID")
}

fn location() -> Location {
    Location::new(
        document(),
        Range::new(Position::new(0, 0, 0), Position::new(0, 1, 1)).expect("range"),
    )
}

fn symbol(value: &str) -> SymbolId {
    SymbolId::new(format!("symbol:{value}")).expect("symbol ID")
}

fn definition(kind: SymbolKind, name: &str) -> SymbolDefinition {
    SymbolDefinition::new(symbol(name), kind, Name::new(name).expect("name"), location())
        .with_qualified_name(QualifiedName::new(format!("billing.{name}")).expect("qualified name"))
}

fn record(id: &str, fact: SemanticFact) -> SemanticFactRecord {
    SemanticFactRecord::new(FactId::new(format!("fact:{id}")).expect("fact ID"), fact)
}

fn graph(facts: Vec<SemanticFactRecord>) -> SemanticGraph {
    SemanticGraph::from_document_facts(
        document(),
        revision("revision:one"),
        SemanticFactSet::new(facts),
    )
    .expect("graph builds")
}

#[test]
fn empty_graph_contains_no_nodes_or_edges() {
    let graph = SemanticGraph::empty();
    assert_eq!(graph.statistics().nodes(), 0);
    assert_eq!(graph.statistics().edges(), 0);
}

#[test]
fn document_and_declarations_become_nodes_and_defines_edges() {
    let payment = definition(SymbolKind::Type, "Payment");
    let graph = graph(vec![record("payment", SemanticFact::Definition(payment.clone()))]);

    let stats = graph.statistics();
    assert_eq!(stats.documents(), 1);
    assert_eq!(stats.symbols(), 1);
    assert_eq!(stats.edges(), 1);
    assert!(matches!(graph.find_symbol(payment.id()), Some(GraphNode::Symbol { .. })));
    assert_eq!(graph.edges().next().expect("defines edge").kind(), EdgeKind::Defines);
}

#[test]
fn containment_parameters_and_calls_are_traversable() {
    let payment = definition(SymbolKind::Type, "Payment");
    let process = definition(SymbolKind::Method, "process").with_container(payment.id().clone());
    let user = definition(SymbolKind::Parameter, "user");
    let facts = vec![
        record("payment", SemanticFact::Definition(payment.clone())),
        record("process", SemanticFact::Definition(process.clone())),
        record(
            "contains-process",
            SemanticFact::Contains(ContainsFact::new(payment.id().clone(), process.id().clone())),
        ),
        record(
            "parameter",
            SemanticFact::Parameter(ParameterFact::new(
                process.id().clone(),
                user.clone(),
                0,
                TypeReference::unresolved(QualifiedName::new("User").expect("type")),
            )),
        ),
        record(
            "call",
            SemanticFact::Call(CallFact::new(
                process.id().clone(),
                SymbolReference::unresolved(QualifiedName::new("validate").expect("name")),
                location(),
            )),
        ),
    ];
    let graph = graph(facts);

    let successors = graph.successors(&GraphNodeId::Symbol(process.id().clone()));
    assert!(successors.iter().any(|node| node.id() == GraphNodeId::Symbol(user.id().clone())));
    assert!(
        graph
            .outgoing_relationships(&GraphNodeId::Symbol(process.id().clone()))
            .iter()
            .any(|edge| edge.kind() == EdgeKind::Calls)
    );
    assert_eq!(graph.statistics().unresolved(), 1);
}

#[test]
fn imports_inheritance_and_implementations_preserve_edge_kinds() {
    let child = definition(SymbolKind::Type, "Payment");
    let facts = vec![
        record("child", SemanticFact::Definition(child.clone())),
        record(
            "import",
            SemanticFact::Import(ImportFact::new(
                document(),
                QualifiedName::new("billing.User").expect("qualified name"),
                false,
                location(),
            )),
        ),
        record(
            "extends",
            SemanticFact::TypeRelation(TypeRelationFact::new(
                child.id().clone(),
                SymbolReference::unresolved(QualifiedName::new("BasePayment").expect("name")),
                TypeRelation::Extends,
                location(),
            )),
        ),
        record(
            "implements",
            SemanticFact::TypeRelation(TypeRelationFact::new(
                child.id().clone(),
                SymbolReference::unresolved(QualifiedName::new("Payable").expect("name")),
                TypeRelation::Implements,
                location(),
            )),
        ),
    ];
    let graph = graph(facts);
    let kinds = graph.edges().map(crate::model::GraphEdge::kind).collect::<Vec<_>>();
    assert!(kinds.contains(&EdgeKind::Imports));
    assert!(kinds.contains(&EdgeKind::Extends));
    assert!(kinds.contains(&EdgeKind::Implements));
}

#[test]
fn resolved_ambiguous_and_external_targets_remain_distinct() {
    let caller = definition(SymbolKind::Method, "process");
    let resolved = definition(SymbolKind::Method, "validate");
    let facts = vec![
        record("caller", SemanticFact::Definition(caller.clone())),
        record("resolved", SemanticFact::Definition(resolved.clone())),
        record(
            "resolved-call",
            SemanticFact::Call(CallFact::new(
                caller.id().clone(),
                SymbolReference::resolved(
                    QualifiedName::new("billing.validate").expect("name"),
                    resolved.id().clone(),
                ),
                location(),
            )),
        ),
        record(
            "ambiguous-reference",
            SemanticFact::Reference(ReferenceFact::new(
                caller.id().clone(),
                SymbolReference::ambiguous(
                    QualifiedName::new("validate").expect("name"),
                    vec![resolved.id().clone(), symbol("other")],
                ),
                ReferenceKind::Call,
                location(),
            )),
        ),
        record(
            "external-call",
            SemanticFact::Call(CallFact::new(
                caller.id().clone(),
                SymbolReference::external(
                    QualifiedName::new("java.lang.String").expect("name"),
                    branchsense_semantic::ExternalSymbolId::new("java:java.lang.String")
                        .expect("external ID"),
                ),
                location(),
            )),
        ),
    ];
    let graph = graph(facts);
    assert_eq!(graph.statistics().external(), 1);
    assert_eq!(graph.statistics().unresolved(), 1);
    assert!(graph.edges().any(|edge| edge.resolution().is_some_and(|state| {
        matches!(state, branchsense_semantic::ResolutionState::Resolved(_))
    })));
}

#[test]
fn duplicate_identical_facts_are_idempotent_but_conflicts_are_rejected() {
    let payment = definition(SymbolKind::Type, "Payment");
    let duplicate = record("payment", SemanticFact::Definition(payment.clone()));
    let graph = graph(vec![duplicate.clone(), duplicate]);
    assert_eq!(graph.statistics().symbols(), 1);

    let conflict =
        record("payment", SemanticFact::Definition(definition(SymbolKind::Enum, "Payment")));
    let result = SemanticGraph::from_document_facts(
        document(),
        revision("revision:one"),
        SemanticFactSet::new(vec![record("payment", SemanticFact::Definition(payment)), conflict]),
    );
    assert!(matches!(result, Err(crate::GraphError::DuplicateFact { .. })));
}

#[test]
fn replacement_and_deletion_publish_new_snapshots_without_mutating_old() {
    let payment = definition(SymbolKind::Type, "Payment");
    let old = graph(vec![record("payment", SemanticFact::Definition(payment.clone()))]);
    let replacement = definition(SymbolKind::Type, "Invoice");
    let new = old
        .replace_document_facts(
            document(),
            revision("revision:two"),
            SemanticFactSet::new(vec![record(
                "invoice",
                SemanticFact::Definition(replacement.clone()),
            )]),
        )
        .expect("replacement succeeds");

    assert!(old.find_symbol(payment.id()).is_some());
    assert!(new.find_symbol(payment.id()).is_none());
    assert!(new.find_symbol(replacement.id()).is_some());

    let deleted =
        new.remove_document(&document(), revision("revision:three")).expect("deletion succeeds");
    assert_eq!(deleted.statistics().nodes(), 0);
    assert_eq!(new.statistics().nodes(), 2);
}

#[test]
fn fact_delta_application_replaces_document_facts() {
    let payment = definition(SymbolKind::Type, "Payment");
    let invoice = definition(SymbolKind::Type, "Invoice");
    let old_facts =
        SemanticFactSet::new(vec![record("payment", SemanticFact::Definition(payment))]);
    let new_facts =
        SemanticFactSet::new(vec![record("invoice", SemanticFact::Definition(invoice.clone()))]);
    let graph =
        SemanticGraph::from_document_facts(document(), revision("revision:one"), old_facts.clone())
            .expect("graph builds");
    let delta =
        FactDelta::between(document(), revision("revision:two"), Some(&old_facts), &new_facts);
    let updated = graph.apply_delta(&delta).expect("delta applies");
    assert!(updated.find_symbol(invoice.id()).is_some());
    assert_eq!(updated.revision_id().expect("revision").as_str(), "revision:two");
}

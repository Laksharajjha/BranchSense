use branchsense_core::{
    DocumentId, Location, Name, Position, QualifiedName, Range, SymbolId, Visibility,
};

use crate::{
    Documentation, FactId, SemanticFact, SemanticFactRecord, SemanticFactSet, SymbolDefinition,
    SymbolKind, SymbolReference,
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

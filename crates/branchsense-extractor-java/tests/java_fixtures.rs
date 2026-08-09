#![allow(missing_docs)]

use std::path::PathBuf;

use branchsense_core::Language;
use branchsense_extractor_java::JavaExtractor;
use branchsense_java::JavaParser;
use branchsense_parser::{Parser, ParserConfiguration};
use branchsense_semantic::{SemanticFact, SymbolKind, TypeRelation};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn extract_fixture(name: &str) -> branchsense_extractor_java::ExtractionResult {
    let path = fixture(name);
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let parsed = parser.parse(&path).expect("fixture parses");
    assert_eq!(parsed.document().language(), Language::Java);
    JavaExtractor::new().extract(parsed.document()).expect("extraction starts")
}

#[test]
fn spring_fixture_extracts_declarations_imports_and_docs() {
    let result = extract_fixture("SpringApplication.java");
    let facts = result.facts().facts();

    assert!(facts.iter().any(|record| matches!(record.fact(), SemanticFact::Import(_))));
    assert!(facts.iter().any(|record| matches!(record.fact(), SemanticFact::Documentation(_))));
    assert!(facts.iter().any(|record| matches!(record.fact(), SemanticFact::Annotation(_))));
    assert!(facts.iter().any(|record| {
        matches!(record.fact(), SemanticFact::Definition(definition) if definition.kind() == SymbolKind::Interface)
    }));
    assert!(facts.iter().any(|record| matches!(record.fact(), SemanticFact::Parameter(_))));
    assert!(facts.iter().any(|record| matches!(record.fact(), SemanticFact::Call(_))));
    assert!(facts.iter().any(|record| matches!(record.fact(), SemanticFact::Contains(_))));
}

#[test]
fn spring_fixture_extracts_implemented_interface_and_nested_types() {
    let result = extract_fixture("SpringApplication.java");
    let facts = result.facts().facts();

    assert!(facts.iter().any(|record| {
        matches!(record.fact(), SemanticFact::TypeRelation(relation) if relation.relation() == TypeRelation::Implements)
    }));
    assert!(facts.iter().any(|record| {
        matches!(record.fact(), SemanticFact::Definition(definition) if definition
            .qualified_name()
            .is_some_and(|name| name.as_str().contains("PaymentService.Repository")))
    }));
}

#[test]
fn malformed_fixture_keeps_partial_facts_and_reports_recovery() {
    let result = extract_fixture("Malformed.java");

    assert!(result.has_errors());
    assert!(!result.facts().is_empty());
}

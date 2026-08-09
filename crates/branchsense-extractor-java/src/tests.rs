use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use branchsense_semantic::{SemanticFact, SymbolKind};

use crate::JavaExtractor;

fn extract(source: &str) -> crate::ExtractionResult {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let parsed = parser
        .parse_source(ParseInput::new("Example.java", source, DocumentVersion::default()))
        .expect("source produces a parsed document");
    JavaExtractor::new().extract(parsed.document()).expect("extraction starts")
}

#[test]
fn extracts_package_type_method_field_and_parameter() {
    let result = extract(
        "package billing;\n\npublic class Payment {\n    private String id;\n    public void process(User user) {}\n}\n",
    );

    let definitions = result
        .facts()
        .facts()
        .iter()
        .filter_map(|record| match record.fact() {
            SemanticFact::Definition(definition) => Some(definition),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(definitions.iter().any(|definition| definition.kind() == SymbolKind::Package));
    assert!(definitions.iter().any(|definition| definition.kind() == SymbolKind::Type));
    assert!(definitions.iter().any(|definition| definition.kind() == SymbolKind::Method));
    assert!(definitions.iter().any(|definition| definition.kind() == SymbolKind::Field));
    assert!(
        result
            .facts()
            .facts()
            .iter()
            .any(|record| matches!(record.fact(), SemanticFact::Parameter(_)))
    );
}

#[test]
fn malformed_java_returns_partial_facts_and_diagnostics() {
    let result = extract("package broken; class Broken { public void run() {");

    assert!(result.has_errors());
    assert!(!result.facts().is_empty());
}

#[test]
fn annotations_and_nested_types_are_extracted() {
    let result =
        extract("package sample;\nclass Outer {\n    @Deprecated\n    class Inner {}\n}\n");

    assert!(
        result
            .facts()
            .facts()
            .iter()
            .any(|record| matches!(record.fact(), SemanticFact::Contains(_)))
    );
    assert!(
        result
            .facts()
            .facts()
            .iter()
            .any(|record| matches!(record.fact(), SemanticFact::Annotation(_)))
    );
}

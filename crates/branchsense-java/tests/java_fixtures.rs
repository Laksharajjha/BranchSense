#![allow(missing_docs)]

use std::path::PathBuf;

use branchsense_core::Language;
use branchsense_java::JavaAdapter;
use branchsense_java::JavaSyntaxTree;
use branchsense_language::{AdapterConfig, AdapterRegistry, Capability, LanguageAdapter};
use branchsense_parser::{Parser, ParserConfiguration};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn valid_fixture_parses_through_registered_adapter() {
    let registry = AdapterRegistry::default();
    registry.register(JavaAdapter::default()).expect("Java adapter registers");
    let adapter = registry.adapter(Language::Java).expect("Java adapter is available");
    let session = adapter.start(&AdapterConfig::default()).expect("Java session starts");
    let parser = session.parser();
    let result = parser.parse(&fixture("Hello.java")).expect("fixture parses");
    let tree = result
        .document()
        .syntax_tree()
        .as_any()
        .downcast_ref::<JavaSyntaxTree>()
        .expect("Java tree is opaque but recoverable by adapter consumers");
    let statistics = tree.statistics();

    assert!(!result.has_errors());
    assert_eq!(result.document().language(), Language::Java);
    assert!(statistics.node_count() > 10);
    assert!(statistics.depth() > 2);
}

#[test]
fn malformed_fixture_returns_recovered_tree_and_diagnostics() {
    let parser =
        branchsense_java::JavaParser::new(ParserConfiguration::default()).expect("grammar loads");
    let result =
        parser.parse(&fixture("Malformed.java")).expect("malformed source still produces a tree");

    assert!(result.has_errors());
    assert!(!result.diagnostics().is_empty());
}

#[test]
fn adapter_declares_only_current_java_capabilities() {
    let adapter = JavaAdapter::default();

    assert!(adapter.capabilities().contains(Capability::IncrementalParsing));
    assert!(adapter.capabilities().contains(Capability::Diagnostics));
    assert!(!adapter.capabilities().contains(Capability::SemanticExtraction));
}

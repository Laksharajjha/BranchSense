use branchsense_core::Language;
use branchsense_language::{AdapterConfig, Capability, LanguageAdapter};

use crate::JavaAdapter;

#[test]
fn adapter_metadata_identifies_java() {
    let adapter = JavaAdapter::default();

    assert_eq!(adapter.language(), Language::Java);
    assert_eq!(adapter.metadata().name(), "branchsense-java");
    assert!(adapter.capabilities().contains(Capability::IncrementalParsing));
    assert!(adapter.capabilities().contains(Capability::Diagnostics));
}

#[test]
fn adapter_starts_a_parser_session() {
    let adapter = JavaAdapter::default();
    let session = adapter.start(&AdapterConfig::default()).expect("Java session starts");

    assert_eq!(session.state(), branchsense_language::AdapterState::Running);
    assert!(session.features().enabled().contains(Capability::IncrementalParsing));
}

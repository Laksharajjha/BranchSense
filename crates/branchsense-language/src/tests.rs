use branchsense_core::Language;

use crate::{
    ADAPTER_API_VERSION, AdapterConfig, AdapterMetadata, AdapterRegistry, Capabilities, Capability,
    FeatureRequest, LanguageAdapter, RegistryError, Version, VersionRange,
};

struct TestAdapter {
    metadata: AdapterMetadata,
}

impl TestAdapter {
    fn new(language: Language, api: VersionRange) -> Self {
        Self {
            metadata: AdapterMetadata::new(
                "test-adapter",
                language,
                Version::new(1, 2, 0),
                api,
                "test adapter",
                Capabilities::single(Capability::IncrementalParsing).with(Capability::Diagnostics),
            ),
        }
    }
}

impl LanguageAdapter for TestAdapter {
    fn metadata(&self) -> &AdapterMetadata {
        &self.metadata
    }
    fn start(
        &self,
        _configuration: &AdapterConfig,
    ) -> Result<std::sync::Arc<dyn crate::AdapterSession>, crate::AdapterError> {
        Err(crate::AdapterError::Lifecycle {
            message: "test adapter has no runtime session".into(),
        })
    }
}

#[test]
fn registry_registers_and_exposes_metadata() {
    let registry = AdapterRegistry::default();
    registry
        .register(TestAdapter::new(Language::Java, VersionRange::from(ADAPTER_API_VERSION)))
        .expect("registration succeeds");
    let adapter = registry.adapter(Language::Java).expect("adapter is registered");

    assert_eq!(adapter.version(), Version::new(1, 2, 0));
    assert!(adapter.capabilities().contains(Capability::IncrementalParsing));
    assert_eq!(adapter.metadata().language(), Language::Java);
}

#[test]
fn registry_rejects_incompatible_adapter_versions() {
    let registry = AdapterRegistry::default();
    let result = registry
        .register(TestAdapter::new(Language::Rust, VersionRange::exact(Version::new(2, 0, 0))));

    assert!(matches!(
        result,
        Err(RegistryError::IncompatibleVersion { language: Language::Rust, .. })
    ));
}

#[test]
fn feature_negotiation_distinguishes_required_and_preferred() {
    let available = Capabilities::single(Capability::IncrementalParsing);
    let request =
        FeatureRequest::new(Capabilities::empty(), Capabilities::single(Capability::Formatting));
    let negotiated = crate::NegotiatedFeatures::negotiate(request, available)
        .expect("optional features may be absent");

    assert_eq!(negotiated.missing_required(), Capabilities::empty());
    assert!(negotiated.missing_preferred().contains(Capability::Formatting));
}

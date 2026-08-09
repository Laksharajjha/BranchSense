//! Java implementation of the language adapter framework.

use std::sync::Arc;

use branchsense_core::Language;
use branchsense_language::{
    ADAPTER_API_VERSION, AdapterConfig, AdapterError, AdapterMetadata, AdapterSession,
    AdapterState, Capabilities, Capability, LanguageAdapter, NegotiatedFeatures, Version,
    VersionRange,
};
use branchsense_parser::Parser;

use crate::JavaParser;

/// Java language adapter factory.
#[derive(Clone, Debug)]
pub struct JavaAdapter {
    metadata: AdapterMetadata,
}

impl Default for JavaAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaAdapter {
    /// Creates the Java adapter metadata and capability declaration.
    #[must_use]
    pub fn new() -> Self {
        let capabilities =
            Capabilities::single(Capability::IncrementalParsing).with(Capability::Diagnostics);
        Self {
            metadata: AdapterMetadata::new(
                "branchsense-java",
                Language::Java,
                Version::new(0, 1, 0),
                VersionRange::from(ADAPTER_API_VERSION),
                "Tree-sitter Java parser",
                capabilities,
            ),
        }
    }
}

impl LanguageAdapter for JavaAdapter {
    fn metadata(&self) -> &AdapterMetadata {
        &self.metadata
    }

    fn start(
        &self,
        configuration: &AdapterConfig,
    ) -> Result<Arc<dyn AdapterSession>, AdapterError> {
        if !self.metadata.api_compatibility().contains(configuration.host_api()) {
            return Err(AdapterError::IncompatibleApi {
                supported: self.metadata.api_compatibility(),
                host: configuration.host_api(),
            });
        }
        let features = self.negotiate(configuration.features())?;
        let parser = JavaParser::new(configuration.parser_configuration().clone())
            .map_err(|error| AdapterError::Lifecycle { message: error.to_string() })?;
        Ok(Arc::new(JavaSession { parser: Arc::new(parser), features }))
    }
}

struct JavaSession {
    parser: Arc<dyn Parser>,
    features: NegotiatedFeatures,
}

impl AdapterSession for JavaSession {
    fn parser(&self) -> Arc<dyn Parser> {
        Arc::clone(&self.parser)
    }
    fn features(&self) -> NegotiatedFeatures {
        self.features
    }
    fn state(&self) -> AdapterState {
        AdapterState::Running
    }
    fn shutdown(&mut self) -> Result<(), AdapterError> {
        Ok(())
    }
}

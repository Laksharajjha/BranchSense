//! Language adapter traits and startup configuration.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use branchsense_core::Language;
use branchsense_parser::ParserConfiguration;

use crate::{
    capabilities::{Capabilities, FeatureRequest, NegotiatedFeatures},
    error::AdapterError,
    lifecycle::AdapterSession,
    metadata::AdapterMetadata,
    version::{ADAPTER_API_VERSION, Version},
};

/// Immutable configuration supplied when an adapter session starts.
#[derive(Clone, Debug)]
pub struct AdapterConfig {
    parser: ParserConfiguration,
    features: FeatureRequest,
    host_api: Version,
}

impl Default for AdapterConfig {
    fn default() -> Self {
        Self {
            parser: ParserConfiguration::default(),
            features: FeatureRequest::default(),
            host_api: ADAPTER_API_VERSION,
        }
    }
}

impl AdapterConfig {
    /// Creates default adapter configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Sets parser configuration.
    #[must_use]
    pub fn with_parser_configuration(mut self, configuration: ParserConfiguration) -> Self {
        self.parser = configuration;
        self
    }
    /// Sets feature requirements.
    #[must_use]
    pub fn with_features(mut self, features: FeatureRequest) -> Self {
        self.features = features;
        self
    }
    /// Returns parser configuration.
    #[must_use]
    pub fn parser_configuration(&self) -> &ParserConfiguration {
        &self.parser
    }
    /// Returns feature requirements.
    #[must_use]
    pub const fn features(&self) -> FeatureRequest {
        self.features
    }
    /// Returns host framework API version.
    #[must_use]
    pub const fn host_api(&self) -> Version {
        self.host_api
    }
}

/// Framework contract implemented by each language adapter.
pub trait LanguageAdapter: Send + Sync {
    /// Returns immutable adapter metadata.
    fn metadata(&self) -> &AdapterMetadata;

    /// Returns declared capabilities.
    fn capabilities(&self) -> Capabilities {
        self.metadata().capabilities()
    }

    /// Returns adapter implementation version.
    fn version(&self) -> Version {
        self.metadata().version()
    }

    /// Starts a configured adapter session.
    fn start(&self, configuration: &AdapterConfig)
    -> Result<Arc<dyn AdapterSession>, AdapterError>;

    /// Returns the adapter language.
    fn language(&self) -> Language {
        self.metadata().language()
    }

    /// Negotiates requested features before starting a session.
    fn negotiate(&self, request: FeatureRequest) -> Result<NegotiatedFeatures, AdapterError> {
        NegotiatedFeatures::negotiate(request, self.capabilities())
    }
}

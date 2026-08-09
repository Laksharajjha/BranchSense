//! Descriptive metadata for language adapters.

use branchsense_core::Language;
use serde::{Deserialize, Serialize};

use crate::{
    capabilities::Capabilities,
    version::{Version, VersionRange},
};

/// Immutable descriptive metadata published by an adapter.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdapterMetadata {
    name: String,
    language: Language,
    version: Version,
    api_compatibility: VersionRange,
    description: String,
    capabilities: Capabilities,
}

impl AdapterMetadata {
    /// Creates adapter metadata.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        language: Language,
        version: Version,
        api_compatibility: VersionRange,
        description: impl Into<String>,
        capabilities: Capabilities,
    ) -> Self {
        Self {
            name: name.into(),
            language,
            version,
            api_compatibility,
            description: description.into(),
            capabilities,
        }
    }

    /// Returns the adapter name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the language handled by the adapter.
    #[must_use]
    pub const fn language(&self) -> Language {
        self.language
    }

    /// Returns the adapter implementation version.
    #[must_use]
    pub const fn version(&self) -> Version {
        self.version
    }

    /// Returns the supported framework API range.
    #[must_use]
    pub const fn api_compatibility(&self) -> VersionRange {
        self.api_compatibility
    }

    /// Returns the human-readable description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns declared capabilities.
    #[must_use]
    pub const fn capabilities(&self) -> Capabilities {
        self.capabilities
    }
}

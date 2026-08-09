//! Adapter capability declarations and feature negotiation.

#![allow(clippy::missing_errors_doc)]

use serde::{Deserialize, Serialize};

use crate::error::AdapterError;

/// A capability an adapter may expose.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Capability {
    /// Supports parser-level incremental edits.
    IncrementalParsing,
    /// Can produce semantic declarations and relationships.
    SemanticExtraction,
    /// Can resolve types across the language's type system.
    TypeResolution,
    /// Can resolve symbols and references.
    SymbolResolution,
    /// Can analyze dependencies across documents.
    CrossFileAnalysis,
    /// Can analyze build or module dependencies.
    DependencyAnalysis,
    /// Can format source text according to language rules.
    Formatting,
    /// Can emit syntax or semantic diagnostics.
    Diagnostics,
}

impl Capability {
    const fn bit(self) -> u16 {
        match self {
            Self::IncrementalParsing => 1 << 0,
            Self::SemanticExtraction => 1 << 1,
            Self::TypeResolution => 1 << 2,
            Self::SymbolResolution => 1 << 3,
            Self::CrossFileAnalysis => 1 << 4,
            Self::DependencyAnalysis => 1 << 5,
            Self::Formatting => 1 << 6,
            Self::Diagnostics => 1 << 7,
        }
    }
}

/// A compact set of adapter capabilities.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Capabilities(u16);

impl Capabilities {
    /// Creates an empty capability set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Creates a set containing every currently defined capability.
    #[must_use]
    pub const fn all() -> Self {
        Self((1 << 8) - 1)
    }

    /// Creates a set containing one capability.
    #[must_use]
    pub const fn single(capability: Capability) -> Self {
        Self(capability.bit())
    }

    /// Adds a capability and returns the updated set.
    #[must_use]
    pub const fn with(self, capability: Capability) -> Self {
        Self(self.0 | capability.bit())
    }

    /// Returns whether this set contains a capability.
    #[must_use]
    pub const fn contains(self, capability: Capability) -> bool {
        self.0 & capability.bit() != 0
    }

    /// Returns whether this set contains every capability in `required`.
    #[must_use]
    pub const fn contains_all(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    /// Returns the capabilities in declaration order.
    pub fn iter(self) -> impl Iterator<Item = Capability> {
        CAPABILITIES.into_iter().filter(move |capability| self.contains(*capability))
    }

    /// Returns capabilities present in `self` but absent from `other`.
    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

const CAPABILITIES: [Capability; 8] = [
    Capability::IncrementalParsing,
    Capability::SemanticExtraction,
    Capability::TypeResolution,
    Capability::SymbolResolution,
    Capability::CrossFileAnalysis,
    Capability::DependencyAnalysis,
    Capability::Formatting,
    Capability::Diagnostics,
];

/// Required and preferred capabilities requested by a host.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FeatureRequest {
    required: Capabilities,
    preferred: Capabilities,
}

impl FeatureRequest {
    /// Creates a feature request.
    #[must_use]
    pub const fn new(required: Capabilities, preferred: Capabilities) -> Self {
        Self { required, preferred }
    }

    /// Returns required capabilities.
    #[must_use]
    pub const fn required(self) -> Capabilities {
        self.required
    }

    /// Returns preferred capabilities.
    #[must_use]
    pub const fn preferred(self) -> Capabilities {
        self.preferred
    }
}

/// Result of negotiating host features against adapter capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NegotiatedFeatures {
    enabled: Capabilities,
    missing_required: Capabilities,
    missing_preferred: Capabilities,
}

impl NegotiatedFeatures {
    /// Negotiates a request against available capabilities.
    pub fn negotiate(
        request: FeatureRequest,
        available: Capabilities,
    ) -> Result<Self, AdapterError> {
        let missing_required = request.required.difference(available);
        if missing_required != Capabilities::empty() {
            return Err(AdapterError::MissingCapabilities { missing: missing_required });
        }
        Ok(Self {
            enabled: available,
            missing_required,
            missing_preferred: request.preferred.difference(available),
        })
    }

    /// Returns capabilities enabled for this session.
    #[must_use]
    pub const fn enabled(self) -> Capabilities {
        self.enabled
    }

    /// Returns required capabilities that were unavailable.
    #[must_use]
    pub const fn missing_required(self) -> Capabilities {
        self.missing_required
    }

    /// Returns preferred capabilities that were unavailable.
    #[must_use]
    pub const fn missing_preferred(self) -> Capabilities {
        self.missing_preferred
    }
}

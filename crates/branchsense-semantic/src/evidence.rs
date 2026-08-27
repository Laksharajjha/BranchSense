//! Explicit availability states for analytical evidence.

use serde::{Deserialize, Serialize};

/// Whether an evidence item is directly observed or derived from other facts.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum EvidenceKind {
    /// Directly observed from a source analysis.
    Primary,
    /// Retained context that supports another evidence item.
    Supporting,
    /// Deterministically derived from other evidence.
    Derived,
}

/// Stable identity for deduplicating one causal evidence item.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EvidenceIdentity {
    kind: EvidenceKind,
    subject: String,
    related: Vec<String>,
}

impl EvidenceIdentity {
    /// Creates a deterministic evidence identity.
    #[must_use]
    pub fn new(kind: EvidenceKind, subject: impl Into<String>, mut related: Vec<String>) -> Self {
        related.sort();
        related.dedup();
        Self { kind, subject: subject.into(), related }
    }

    /// Returns the evidence category.
    #[must_use]
    pub const fn kind(&self) -> EvidenceKind {
        self.kind
    }

    /// Returns the causal subject.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Returns sorted, deduplicated related identities.
    #[must_use]
    pub fn related(&self) -> &[String] {
        &self.related
    }
}

/// Availability of an analytical result.
///
/// Consumers must not interpret every state other than [`EvidenceState::Observed`] as a
/// zero-risk result. In particular, unavailable, unsupported, unresolved, and
/// truncated analysis require different treatment from a completed analysis
/// that found no evidence.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub enum EvidenceState {
    /// Analysis completed and produced one or more evidence items.
    Observed,
    /// Analysis completed for the requested scope and found no evidence.
    #[default]
    NoEvidence,
    /// Analysis could not be run for the requested scope.
    Unavailable,
    /// The requested analysis is not supported by the current adapter.
    Unsupported,
    /// Analysis ran but one or more semantic references could not be resolved.
    Unresolved,
    /// Multiple candidates prevented a unique semantic conclusion.
    Ambiguous,
    /// Analysis was bounded before the complete scope was examined.
    Truncated,
    /// Analysis failed before producing a trustworthy result.
    Failed,
}

/// Completeness of the independent evidence domains used by future analysis.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceCompleteness {
    semantic: EvidenceState,
    historical: EvidenceState,
    responsibility: EvidenceState,
}

impl EvidenceCompleteness {
    /// Creates completeness with no domains analyzed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            semantic: EvidenceState::Unavailable,
            historical: EvidenceState::Unavailable,
            responsibility: EvidenceState::Unavailable,
        }
    }

    /// Sets the semantic evidence state.
    #[must_use]
    pub const fn with_semantic(mut self, state: EvidenceState) -> Self {
        self.semantic = state;
        self
    }

    /// Sets the historical evidence state.
    #[must_use]
    pub const fn with_historical(mut self, state: EvidenceState) -> Self {
        self.historical = state;
        self
    }

    /// Sets the responsibility evidence state.
    #[must_use]
    pub const fn with_responsibility(mut self, state: EvidenceState) -> Self {
        self.responsibility = state;
        self
    }

    /// Returns semantic evidence completeness.
    #[must_use]
    pub const fn semantic(&self) -> EvidenceState {
        self.semantic
    }

    /// Returns historical evidence completeness.
    #[must_use]
    pub const fn historical(&self) -> EvidenceState {
        self.historical
    }

    /// Returns responsibility evidence completeness.
    #[must_use]
    pub const fn responsibility(&self) -> EvidenceState {
        self.responsibility
    }
}

impl Default for EvidenceCompleteness {
    fn default() -> Self {
        Self::new()
    }
}

impl EvidenceState {
    /// Returns whether this state represents a completed, positive observation.
    #[must_use]
    pub const fn is_observed(self) -> bool {
        matches!(self, Self::Observed)
    }

    /// Returns whether this state represents a completed negative observation.
    #[must_use]
    pub const fn is_no_evidence(self) -> bool {
        matches!(self, Self::NoEvidence)
    }

    /// Returns whether the result must be treated as incomplete or unknown.
    #[must_use]
    pub const fn is_inconclusive(self) -> bool {
        !matches!(self, Self::Observed | Self::NoEvidence)
    }
}

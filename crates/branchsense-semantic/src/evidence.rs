//! Explicit availability states for analytical evidence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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

/// A relationship between two evidence observations.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum EvidenceRelation {
    /// The first observation provides support for the second.
    Supports,
    /// The first observation was deterministically derived from the second.
    DerivedFrom,
    /// The first observation independently corroborates the second.
    Corroborates,
}

/// A typed lineage link between evidence observations.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EvidenceLink {
    from: EvidenceIdentity,
    to: EvidenceIdentity,
    relation: EvidenceRelation,
}

/// Deterministic, immutable-friendly evidence registry.
///
/// The ledger is the shared deduplication contract for analytical consumers.
/// Identities represent the same underlying observation; links represent
/// relationships between observations and are never collapsed merely because
/// their subjects match. `Supports` and `Corroborates` therefore retain their
/// distinct semantics, while `DerivedFrom` records deterministic lineage.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceLedger {
    identities: BTreeSet<EvidenceIdentity>,
    lineage: BTreeSet<EvidenceLink>,
}

impl EvidenceLedger {
    /// Creates an empty deterministic ledger.
    #[must_use]
    pub const fn new() -> Self {
        Self { identities: BTreeSet::new(), lineage: BTreeSet::new() }
    }

    /// Adds an identity and returns whether it was new.
    pub fn insert_identity(&mut self, identity: EvidenceIdentity) -> bool {
        self.identities.insert(identity)
    }

    /// Adds a lineage link and returns whether it was new.
    pub fn insert_link(&mut self, link: EvidenceLink) -> bool {
        self.lineage.insert(link)
    }

    /// Merges another ledger without collapsing distinct relationships.
    pub fn merge(&mut self, other: &Self) {
        self.identities.extend(other.identities.iter().cloned());
        self.lineage.extend(other.lineage.iter().cloned());
    }

    /// Returns identities in deterministic order.
    pub fn identities(&self) -> impl Iterator<Item = &EvidenceIdentity> {
        self.identities.iter()
    }

    /// Returns lineage links in deterministic order.
    pub fn lineage(&self) -> impl Iterator<Item = &EvidenceLink> {
        self.lineage.iter()
    }

    /// Returns whether the ledger contains the identity.
    #[must_use]
    pub fn contains_identity(&self, identity: &EvidenceIdentity) -> bool {
        self.identities.contains(identity)
    }
}

impl EvidenceLink {
    /// Creates a deterministic relationship between two evidence identities.
    #[must_use]
    pub const fn new(
        from: EvidenceIdentity,
        to: EvidenceIdentity,
        relation: EvidenceRelation,
    ) -> Self {
        Self { from, to, relation }
    }

    /// Returns the derived or supporting evidence identity.
    #[must_use]
    pub const fn from(&self) -> &EvidenceIdentity {
        &self.from
    }

    /// Returns the source evidence identity.
    #[must_use]
    pub const fn to(&self) -> &EvidenceIdentity {
        &self.to
    }

    /// Returns the lineage relationship.
    #[must_use]
    pub const fn relation(&self) -> EvidenceRelation {
        self.relation
    }
}

impl EvidenceIdentity {
    /// Creates a deterministic evidence identity.
    #[must_use]
    pub fn new(kind: EvidenceKind, subject: impl Into<String>, mut related: Vec<String>) -> Self {
        related.sort();
        related.dedup();
        Self { kind, subject: subject.into(), related }
    }

    /// Creates an identity for a semantic entity.
    #[must_use]
    pub fn semantic(kind: EvidenceKind, identity: &crate::SemanticEntityIdentity) -> Self {
        Self::new(
            kind,
            format!(
                "{}:{:?}:{}",
                identity.document().display(),
                identity.kind(),
                identity.qualified_name()
            ),
            Vec::new(),
        )
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

/// Common metadata carried by an analytical result.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceEnvelope {
    state: EvidenceState,
    completeness: EvidenceCompleteness,
    provenance: crate::AnalysisProvenance,
    identities: Vec<EvidenceIdentity>,
    lineage: Vec<EvidenceLink>,
}

impl EvidenceEnvelope {
    /// Creates an analytical envelope with explicit state and provenance.
    #[must_use]
    pub fn new(
        state: EvidenceState,
        completeness: EvidenceCompleteness,
        provenance: crate::AnalysisProvenance,
    ) -> Self {
        Self { state, completeness, provenance, identities: Vec::new(), lineage: Vec::new() }
    }

    /// Creates a result envelope that preserves its parent's provenance and
    /// evidence lineage while changing the result state.
    #[must_use]
    pub fn derived_from(
        parent: &Self,
        state: EvidenceState,
        completeness: EvidenceCompleteness,
    ) -> Self {
        let mut envelope = Self::new(state, completeness, parent.provenance.clone());
        for identity in &parent.identities {
            envelope = envelope.with_identity(identity.clone());
        }
        for link in &parent.lineage {
            envelope = envelope.with_link(link.clone());
        }
        envelope
    }

    /// Adds an evidence identity, preserving deterministic uniqueness.
    #[must_use]
    pub fn with_identity(mut self, identity: EvidenceIdentity) -> Self {
        let mut ledger = EvidenceLedger::new();
        ledger.identities.extend(self.identities);
        ledger.insert_identity(identity);
        self.identities = ledger.identities.into_iter().collect();
        self
    }

    /// Replaces the result state while retaining its lineage metadata.
    #[must_use]
    pub const fn with_state(mut self, state: EvidenceState) -> Self {
        self.state = state;
        self
    }

    /// Replaces completeness while retaining identity and provenance metadata.
    #[must_use]
    pub const fn with_completeness(mut self, completeness: EvidenceCompleteness) -> Self {
        self.completeness = completeness;
        self
    }

    /// Adds a lineage link, preserving deterministic uniqueness.
    #[must_use]
    pub fn with_link(mut self, link: EvidenceLink) -> Self {
        let mut ledger = EvidenceLedger::new();
        ledger.lineage.extend(self.lineage);
        ledger.insert_link(link);
        self.lineage = ledger.lineage.into_iter().collect();
        self
    }

    /// Returns the overall evidence state.
    #[must_use]
    pub const fn state(&self) -> EvidenceState {
        self.state
    }

    /// Returns completeness for each evidence domain.
    #[must_use]
    pub const fn completeness(&self) -> &EvidenceCompleteness {
        &self.completeness
    }

    /// Returns analysis provenance.
    #[must_use]
    pub const fn provenance(&self) -> &crate::AnalysisProvenance {
        &self.provenance
    }

    /// Returns stable evidence identities carried by this result.
    #[must_use]
    pub fn identities(&self) -> &[EvidenceIdentity] {
        &self.identities
    }

    /// Returns relationships between evidence observations.
    #[must_use]
    pub fn lineage(&self) -> &[EvidenceLink] {
        &self.lineage
    }

    /// Returns this envelope as a standalone deterministic ledger.
    #[must_use]
    pub fn ledger(&self) -> EvidenceLedger {
        EvidenceLedger {
            identities: self.identities.iter().cloned().collect(),
            lineage: self.lineage.iter().cloned().collect(),
        }
    }
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
    /// Combines two states without treating incomplete evidence as negative
    /// evidence. Inconclusive states take precedence over observed and empty
    /// states; this keeps uncertainty visible as results are composed.
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        if self.rank() >= other.rank() { self } else { other }
    }

    const fn rank(self) -> u8 {
        match self {
            Self::NoEvidence => 0,
            Self::Observed => 1,
            Self::Truncated => 2,
            Self::Unresolved => 3,
            Self::Ambiguous => 4,
            Self::Unsupported => 5,
            Self::Unavailable => 6,
            Self::Failed => 7,
        }
    }

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

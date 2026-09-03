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

/// Identity of an underlying observation, independent of its evidence role.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ObservationIdentity {
    subject: String,
    related: Vec<String>,
}

impl ObservationIdentity {
    /// Creates a deterministic observation identity.
    #[must_use]
    pub fn new(subject: impl Into<String>, mut related: Vec<String>) -> Self {
        related.sort();
        related.dedup();
        Self { subject: subject.into(), related }
    }

    /// Returns the observation subject.
    #[must_use]
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Returns related identities in sorted order.
    #[must_use]
    pub fn related(&self) -> &[String] {
        &self.related
    }
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
    observations: BTreeSet<ObservationIdentity>,
    identities: BTreeSet<EvidenceIdentity>,
    lineage: BTreeSet<EvidenceLink>,
}

impl EvidenceLedger {
    /// Creates an empty deterministic ledger.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observations: BTreeSet::new(),
            identities: BTreeSet::new(),
            lineage: BTreeSet::new(),
        }
    }

    /// Adds an identity and returns whether it was new.
    pub fn insert_identity(&mut self, identity: EvidenceIdentity) -> bool {
        let observation = identity.observation();
        let is_new = self.identities.insert(identity);
        self.observations.insert(observation);
        is_new
    }

    /// Adds a lineage link and returns whether it was new.
    pub fn insert_link(&mut self, link: EvidenceLink) -> bool {
        self.lineage.insert(link)
    }

    /// Merges another ledger without collapsing distinct relationships.
    pub fn merge(&mut self, other: &Self) {
        self.observations.extend(other.observations.iter().cloned());
        self.identities.extend(other.identities.iter().cloned());
        self.lineage.extend(other.lineage.iter().cloned());
    }

    /// Returns identities in deterministic order.
    pub fn identities(&self) -> impl Iterator<Item = &EvidenceIdentity> {
        self.identities.iter()
    }

    /// Returns underlying observations in deterministic order.
    pub fn observations(&self) -> impl Iterator<Item = &ObservationIdentity> {
        self.observations.iter()
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

    /// Returns whether the ledger contains the underlying observation.
    #[must_use]
    pub fn contains_observation(&self, observation: &ObservationIdentity) -> bool {
        self.observations.contains(observation)
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

    /// Returns the underlying observation identity without its role.
    #[must_use]
    pub fn observation(&self) -> ObservationIdentity {
        ObservationIdentity::new(self.subject.clone(), self.related.clone())
    }

    /// Returns whether two role-bearing identities describe one observation.
    #[must_use]
    pub fn same_observation(&self, other: &Self) -> bool {
        self.observation() == other.observation()
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

/// Scope of an incompleteness issue.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CompletenessScope {
    /// The entire requested analysis is affected.
    Global,
    /// A single source document is affected; downstream consumers determine
    /// whether it participates in their changed subgraph.
    Document,
    /// The issue is known to affect the analyzed subgraph.
    AffectedSubgraph,
    /// The issue is known not to affect the analyzed subgraph.
    Unrelated,
}

/// Origin of a completeness issue.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CompletenessSource {
    /// Git or another source provider could not load bytes.
    Source,
    /// Parsing did not produce complete input.
    Parsing,
    /// Semantic extraction did not produce complete facts.
    Extraction,
    /// Graph construction or publication was incomplete.
    Graph,
    /// Historical evidence was incomplete.
    History,
    /// Responsibility evidence was incomplete.
    Responsibility,
}

/// A deterministic, structured explanation for incomplete evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompletenessIssue {
    state: EvidenceState,
    scope: CompletenessScope,
    source: CompletenessSource,
    path: Option<String>,
    message: String,
}

impl CompletenessIssue {
    /// Creates a completeness issue associated with an optional source path.
    #[must_use]
    pub fn new(
        state: EvidenceState,
        scope: CompletenessScope,
        source: CompletenessSource,
        path: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        Self { state, scope, source, path, message: message.into() }
    }

    /// Returns the issue state.
    #[must_use]
    pub const fn state(&self) -> EvidenceState {
        self.state
    }

    /// Returns the issue scope.
    #[must_use]
    pub const fn scope(&self) -> CompletenessScope {
        self.scope
    }

    /// Returns the issue source.
    #[must_use]
    pub const fn source(&self) -> CompletenessSource {
        self.source
    }

    /// Returns the optional affected path.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Returns the human-readable diagnostic.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
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
    pub fn with_completeness(mut self, completeness: EvidenceCompleteness) -> Self {
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
            observations: self.identities.iter().map(EvidenceIdentity::observation).collect(),
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
    #[serde(default)]
    issues: Vec<CompletenessIssue>,
}

impl EvidenceCompleteness {
    /// Creates completeness with no domains analyzed.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            semantic: EvidenceState::Unavailable,
            historical: EvidenceState::Unavailable,
            responsibility: EvidenceState::Unavailable,
            issues: Vec::new(),
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

    /// Adds a deterministic completeness issue.
    #[must_use]
    pub fn with_issue(mut self, issue: CompletenessIssue) -> Self {
        self.issues.push(issue);
        self.issues.sort_by(|left, right| {
            (left.scope, left.source, left.path.as_deref(), &left.message).cmp(&(
                right.scope,
                right.source,
                right.path.as_deref(),
                &right.message,
            ))
        });
        self.issues.dedup();
        self
    }

    /// Returns structured incompleteness issues in deterministic order.
    #[must_use]
    pub fn issues(&self) -> &[CompletenessIssue] {
        &self.issues
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

    /// Returns whether a state in a *critical* evidence domain forces BCS to
    /// return `Indeterminate` rather than a normal ordinal result.
    ///
    /// Non-critical inconclusive states reduce completeness and produce
    /// warnings but do not by themselves require abstention. The distinction
    /// depends on the domain (see [`EvidenceCompleteness::abstention_decision`]).
    #[must_use]
    pub const fn is_critical_for_abstention(self) -> bool {
        matches!(self, Self::Unavailable | Self::Failed | Self::Ambiguous)
    }
}

/// The BCS abstention decision for a given evidence envelope.
///
/// BCS must return `Indeterminate` whenever trustworthy evidence is absent for
/// a required domain. It may continue with warnings when incompleteness is
/// localized and outside the analyzed changed subgraph.
///
/// # Semantics
///
/// | Variant        | Meaning                                                     |
/// |---------------|-------------------------------------------------------------|
/// | `Proceed`      | All required domains have conclusive evidence.               |
/// | `Warn`         | One or more non-critical domains are incomplete.             |
/// | `Indeterminate`| A critical domain is inconclusive; normal scoring must stop. |
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AbstentionDecision {
    /// All required domains have at least conclusive (Observed or `NoEvidence`)
    /// evidence; BCS may produce a normal ordinal result.
    Proceed,
    /// One or more non-critical domains are incomplete. BCS may still produce
    /// an ordinal result but must attach warnings and reduced completeness.
    Warn,
    /// A required domain is critically inconclusive. BCS must return
    /// `Indeterminate` and must not produce a normal ordinal band.
    Indeterminate,
}

impl AbstentionDecision {
    /// Returns whether BCS must abstain from producing a normal ordinal result.
    #[must_use]
    pub const fn must_abstain(self) -> bool {
        matches!(self, Self::Indeterminate)
    }
}

impl EvidenceCompleteness {
    /// Computes the BCS abstention decision from the current completeness.
    ///
    /// Rules (in priority order):
    ///
    /// 1. If any *globally-scoped* issue carries a state that is critical for
    ///    abstention, or if the semantic domain is critically inconclusive,
    ///    return `Indeterminate`.
    /// 2. If the semantic domain is `Unavailable` or `Failed`, return
    ///    `Indeterminate` (semantic evidence is always required for BCS).
    /// 3. If any remaining domain is inconclusive (`Unresolved`, `Truncated`,
    ///    `Unsupported`, `Ambiguous`, `Unavailable`, `Failed`) return `Warn`.
    /// 4. Otherwise return `Proceed`.
    ///
    /// Historical and responsibility signals are contextual enrichments; their
    /// absence is non-critical (`Warn`) unless accompanied by a global issue.
    #[must_use]
    pub fn abstention_decision(&self) -> AbstentionDecision {
        // Semantic evidence is always required for BCS.
        if self.semantic.is_critical_for_abstention() {
            return AbstentionDecision::Indeterminate;
        }

        // Any global-scope issue with a critical state forces abstention.
        let has_global_critical = self.issues.iter().any(|issue| {
            issue.scope == CompletenessScope::Global && issue.state.is_critical_for_abstention()
        });
        if has_global_critical {
            return AbstentionDecision::Indeterminate;
        }

        // Any affected-subgraph issue with a critical state forces abstention.
        let has_subgraph_critical = self.issues.iter().any(|issue| {
            issue.scope == CompletenessScope::AffectedSubgraph
                && issue.state.is_critical_for_abstention()
        });
        if has_subgraph_critical {
            return AbstentionDecision::Indeterminate;
        }

        // Non-critical inconclusive domains produce a Warn.
        let any_warn = self.semantic.is_inconclusive()
            || self.historical.is_inconclusive()
            || self.responsibility.is_inconclusive()
            || self.issues.iter().any(|issue| issue.state.is_inconclusive());
        if any_warn { AbstentionDecision::Warn } else { AbstentionDecision::Proceed }
    }
}

//! Versioned evaluation dataset contract for future BCS calibration.
//!
//! This module defines the schema for future BCS evaluation records. It does
//! **not** implement model training, probability estimation, or data
//! collection. An evaluation record is a reproducible ground-truth snapshot
//! that pairs known repository histories with separate, independently labelled
//! outcome fields.
//!
//! # Design principles
//!
//! - **Reproducibility**: every record must retain the exact input identities
//!   required to replay the same analysis without reloading repository bytes.
//! - **Outcome independence**: textual merge outcome, build outcome, test
//!   outcome, and semantic integration outcome are labelled separately. One
//!   outcome does not prove another.
//! - **Version pinning**: algorithm and configuration versions are recorded so
//!   that older records can be retrained against a newer algorithm.
//! - **No network dependency**: records are curated, not scraped. The schema
//!   does not embed large payloads; it references content-addressed inputs.
//! - **Label provenance**: every outcome label records its source so that
//!   automated and manual labels can be weighted differently during evaluation.

use serde::{Deserialize, Serialize};

use crate::{AnalysisProvenance, EvidenceCompleteness, EvidenceState};

/// Semantic version of the evaluation dataset schema.
///
/// Increment the major version when fields are removed or semantically
/// reordered. Increment the minor version when fields are added.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct DatasetSchemaVersion {
    major: u32,
    minor: u32,
}

impl DatasetSchemaVersion {
    /// Creates a dataset schema version.
    #[must_use]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    /// Returns the current schema version for newly created records.
    #[must_use]
    pub const fn current() -> Self {
        Self::new(1, 0)
    }

    /// Returns the major version.
    #[must_use]
    pub const fn major(&self) -> u32 {
        self.major
    }

    /// Returns the minor version.
    #[must_use]
    pub const fn minor(&self) -> u32 {
        self.minor
    }
}

/// Identity of a repository used in an evaluation record.
///
/// This is a content-addressed reference and does not embed a URL or
/// filesystem path; consumers should resolve it through a registry.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EvalRepositoryIdentity {
    /// Stable repository identifier (e.g., a normalized remote URL hash).
    id: String,
    /// Human-readable hint for debugging; not used for equality.
    hint: Option<String>,
}

impl EvalRepositoryIdentity {
    /// Creates a repository identity.
    #[must_use]
    pub fn new(id: impl Into<String>, hint: Option<impl Into<String>>) -> Self {
        Self { id: id.into(), hint: hint.map(Into::into) }
    }

    /// Returns the stable repository identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the optional human-readable hint.
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

/// Identifies a single point in a repository's revision graph.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EvalRevision {
    /// Content-addressed commit hash or equivalent.
    hash: String,
}

impl EvalRevision {
    /// Creates a revision from a content-addressed hash.
    #[must_use]
    pub fn new(hash: impl Into<String>) -> Self {
        Self { hash: hash.into() }
    }

    /// Returns the content-addressed hash.
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }
}

/// Outcome labels for a single evaluation record.
///
/// Each field is independently labelled. One outcome does not prove another.
///
/// - A textual merge conflict does not prove that the build would fail.
/// - A passing build does not prove semantic integration success.
/// - A test failure is weak evidence of semantic integration issues unless
///   the failure's relationship to the branch interaction is documented.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvalOutcome {
    /// Whether the three-way Git merge required manual textual conflict
    /// resolution.
    pub textual_merge_conflict: Option<bool>,
    /// Whether the merged result failed the defined build command.
    pub build_failure: Option<bool>,
    /// Whether the merged result failed the defined test scope.
    pub test_failure: Option<bool>,
    /// Whether a semantic integration issue was separately adjudicated.
    pub semantic_integration_issue: Option<bool>,
}

impl EvalOutcome {
    /// Creates an outcome record with all fields unknown.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            textual_merge_conflict: None,
            build_failure: None,
            test_failure: None,
            semantic_integration_issue: None,
        }
    }

    /// Sets the textual merge conflict label.
    #[must_use]
    pub const fn with_textual_merge_conflict(mut self, value: bool) -> Self {
        self.textual_merge_conflict = Some(value);
        self
    }

    /// Sets the build failure label.
    #[must_use]
    pub const fn with_build_failure(mut self, value: bool) -> Self {
        self.build_failure = Some(value);
        self
    }

    /// Sets the test failure label.
    #[must_use]
    pub const fn with_test_failure(mut self, value: bool) -> Self {
        self.test_failure = Some(value);
        self
    }

    /// Sets the semantic integration issue label.
    #[must_use]
    pub const fn with_semantic_integration_issue(mut self, value: bool) -> Self {
        self.semantic_integration_issue = Some(value);
        self
    }
}

/// How confident the labeller is in the assigned outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum OutcomeConfidence {
    /// Outcome was directly observed from a clean automated pipeline.
    High,
    /// Outcome was inferred or partially automated with human review.
    Medium,
    /// Outcome was manually judged or is disputed.
    Low,
}

/// How an outcome label was produced.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LabelProvenance {
    /// Whether the label was applied by an automated pipeline.
    automated: bool,
    /// Free-text description of the labelling source.
    source: String,
    /// Optional version of the labelling tool or script.
    tool_version: Option<String>,
}

impl LabelProvenance {
    /// Creates a label provenance record.
    #[must_use]
    pub fn new(automated: bool, source: impl Into<String>) -> Self {
        Self { automated, source: source.into(), tool_version: None }
    }

    /// Sets the tool version used for automated labelling.
    #[must_use]
    pub fn with_tool_version(mut self, version: impl Into<String>) -> Self {
        self.tool_version = Some(version.into());
        self
    }

    /// Returns whether the label was produced by an automated pipeline.
    #[must_use]
    pub const fn automated(&self) -> bool {
        self.automated
    }

    /// Returns the labelling source description.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }
}

/// Ordinal assessment predicted by a BCS algorithm version.
///
/// This type represents the future BCS output without implementing scoring.
/// Its values mirror the `BcsAssessment` ordinal bands defined in
/// `docs/bcs-contract.md`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PredictedOrdinalAssessment {
    /// No semantic evidence of an integration concern.
    None,
    /// Weak evidence; useful context but not a clear integration signal.
    Low,
    /// Meaningful semantic interaction that deserves review.
    Moderate,
    /// Strong semantic evidence of potential integration friction.
    High,
    /// Critical semantic evidence of a likely integration concern.
    Critical,
    /// Insufficient trustworthy evidence to produce a normal assessment.
    Indeterminate,
}

/// A versioned evaluation record for future BCS calibration.
///
/// # Completeness note
///
/// Not all fields need to be populated for a record to be useful.
/// `Option` fields may be absent when that evidence domain was unavailable.
/// Consumers must not treat absent fields as zero-evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvalRecord {
    /// Schema version for forward-compatibility checks.
    schema_version: DatasetSchemaVersion,
    /// Repository this record describes.
    repository: EvalRepositoryIdentity,
    /// The common base revision for both branches.
    base_revision: EvalRevision,
    /// The tip revision of branch A at analysis time.
    branch_a_revision: EvalRevision,
    /// The tip revision of branch B at analysis time.
    branch_b_revision: EvalRevision,
    /// The three-way merge base revision (may differ from `base_revision`).
    merge_base: EvalRevision,
    /// Evidence completeness at the time the record was captured.
    completeness: EvidenceCompleteness,
    /// Provenance of the analysis that produced this record.
    provenance: AnalysisProvenance,
    /// BCS algorithm version string.
    algorithm_version: String,
    /// Configuration fingerprint version string.
    configuration_version: String,
    /// The ordinal assessment predicted by the BCS algorithm, if computed.
    predicted_assessment: Option<PredictedOrdinalAssessment>,
    /// Independently labelled outcomes.
    outcome: EvalOutcome,
    /// Confidence in the outcome labels.
    outcome_confidence: Option<OutcomeConfidence>,
    /// Provenance of the outcome labels.
    label_provenance: Option<LabelProvenance>,
    /// Optional free-text notes about manual conflict-resolution steps.
    conflict_resolution_notes: Option<String>,
    /// Overall semantic evidence state at the time of capture.
    semantic_evidence_state: EvidenceState,
}

impl EvalRecord {
    /// Creates a minimal evaluation record.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::similar_names)]
    #[must_use]
    pub fn new(
        repository: EvalRepositoryIdentity,
        base_revision: EvalRevision,
        branch_a_revision: EvalRevision,
        branch_b_revision: EvalRevision,
        merge_base: EvalRevision,
        completeness: EvidenceCompleteness,
        provenance: AnalysisProvenance,
        algorithm_version: impl Into<String>,
        configuration_version: impl Into<String>,
        semantic_evidence_state: EvidenceState,
    ) -> Self {
        Self {
            schema_version: DatasetSchemaVersion::current(),
            repository,
            base_revision,
            branch_a_revision,
            branch_b_revision,
            merge_base,
            completeness,
            provenance,
            algorithm_version: algorithm_version.into(),
            configuration_version: configuration_version.into(),
            predicted_assessment: None,
            outcome: EvalOutcome::new(),
            outcome_confidence: None,
            label_provenance: None,
            conflict_resolution_notes: None,
            semantic_evidence_state,
        }
    }

    /// Attaches a predicted ordinal assessment.
    #[must_use]
    pub fn with_predicted_assessment(mut self, assessment: PredictedOrdinalAssessment) -> Self {
        self.predicted_assessment = Some(assessment);
        self
    }

    /// Attaches independently labelled outcomes.
    #[must_use]
    pub fn with_outcome(mut self, outcome: EvalOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Attaches outcome confidence.
    #[must_use]
    pub const fn with_outcome_confidence(mut self, confidence: OutcomeConfidence) -> Self {
        self.outcome_confidence = Some(confidence);
        self
    }

    /// Attaches label provenance.
    #[must_use]
    pub fn with_label_provenance(mut self, provenance: LabelProvenance) -> Self {
        self.label_provenance = Some(provenance);
        self
    }

    /// Returns the schema version.
    #[must_use]
    pub const fn schema_version(&self) -> &DatasetSchemaVersion {
        &self.schema_version
    }

    /// Returns the repository identity.
    #[must_use]
    pub const fn repository(&self) -> &EvalRepositoryIdentity {
        &self.repository
    }

    /// Returns the base revision.
    #[must_use]
    pub const fn base_revision(&self) -> &EvalRevision {
        &self.base_revision
    }

    /// Returns the branch A tip revision.
    #[must_use]
    pub const fn branch_a_revision(&self) -> &EvalRevision {
        &self.branch_a_revision
    }

    /// Returns the branch B tip revision.
    #[must_use]
    pub const fn branch_b_revision(&self) -> &EvalRevision {
        &self.branch_b_revision
    }

    /// Returns the merge base revision.
    #[must_use]
    pub const fn merge_base(&self) -> &EvalRevision {
        &self.merge_base
    }

    /// Returns the evidence completeness.
    #[must_use]
    pub const fn completeness(&self) -> &EvidenceCompleteness {
        &self.completeness
    }

    /// Returns the analysis provenance.
    #[must_use]
    pub const fn provenance(&self) -> &AnalysisProvenance {
        &self.provenance
    }

    /// Returns the predicted ordinal assessment, if any.
    #[must_use]
    pub const fn predicted_assessment(&self) -> Option<PredictedOrdinalAssessment> {
        self.predicted_assessment
    }

    /// Returns the independently labelled outcomes.
    #[must_use]
    pub const fn outcome(&self) -> &EvalOutcome {
        &self.outcome
    }

    /// Returns the semantic evidence state at the time of capture.
    #[must_use]
    pub const fn semantic_evidence_state(&self) -> EvidenceState {
        self.semantic_evidence_state
    }

    /// Returns the algorithm version string.
    #[must_use]
    pub fn algorithm_version(&self) -> &str {
        &self.algorithm_version
    }

    /// Returns the configuration version string.
    #[must_use]
    pub fn configuration_version(&self) -> &str {
        &self.configuration_version
    }
}

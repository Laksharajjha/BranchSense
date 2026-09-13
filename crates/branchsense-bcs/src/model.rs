//! Core types and domain model for BCS assessment.

use branchsense_semantic::{AbstentionDecision, EvidenceEnvelope};
use serde::{Deserialize, Serialize};

/// The deterministic, explainable ordinal assessment bands for BCS V1.
///
/// Indeterminate is not a risk band; it means the system abstained due to
/// insufficient or untrustworthy evidence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BcsOrdinalBand {
    /// No semantic interaction detected.
    None,
    /// Weak or distant interaction; unlikely to require deep review.
    Low,
    /// Indirect or moderate interaction; should be reviewed.
    Moderate,
    /// Direct semantic interaction or shared responsibility; requires review.
    High,
    /// Severe collision, signature conflict, or heavy shared coupling.
    Critical,
    /// Trustworthy evidence was missing, forcing the system to abstain.
    Indeterminate,
}

/// The structured explanation for a BCS assessment.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BcsExplanation {
    reasons: Vec<String>,
}

impl BcsExplanation {
    /// Creates a new explanation from a list of structured reason strings.
    #[must_use]
    pub const fn new(reasons: Vec<String>) -> Self {
        Self { reasons }
    }

    /// The human-readable structured reasons.
    #[must_use]
    pub fn reasons(&self) -> &[String] {
        &self.reasons
    }
}

/// The final outcome of a deterministic BCS evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BcsAssessment {
    band: BcsOrdinalBand,
    ordinal_score: u16,
    evidence: EvidenceEnvelope,
    abstention: Option<AbstentionDecision>,
    explanation: BcsExplanation,
}

impl BcsAssessment {
    /// Constructs a new BCS assessment.
    #[must_use]
    pub const fn new(
        band: BcsOrdinalBand,
        ordinal_score: u16,
        evidence: EvidenceEnvelope,
        abstention: Option<AbstentionDecision>,
        explanation: BcsExplanation,
    ) -> Self {
        Self { band, ordinal_score, evidence, abstention, explanation }
    }

    /// The primary ordinal band.
    #[must_use]
    pub const fn band(&self) -> BcsOrdinalBand {
        self.band
    }

    /// The deterministic bounded score.
    #[must_use]
    pub const fn ordinal_score(&self) -> u16 {
        self.ordinal_score
    }

    /// The evidence envelope supporting this assessment.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceEnvelope {
        &self.evidence
    }

    /// The abstention decision if the system failed to produce a fully trustworthy result.
    #[must_use]
    pub const fn abstention(&self) -> Option<&AbstentionDecision> {
        self.abstention.as_ref()
    }

    /// The structured explanation for this assessment.
    #[must_use]
    pub const fn explanation(&self) -> &BcsExplanation {
        &self.explanation
    }
}

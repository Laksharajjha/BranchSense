//! Deterministic factor model for BCS scoring.

use branchsense_semantic::EvidenceIdentity;
use serde::{Deserialize, Serialize};

/// A specific deterministic dimension of semantic interaction.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BcsFactorKind {
    /// Direct semantic collision (e.g., both branches modify the same method signature).
    DirectCollision,
    /// Shared transitive impact.
    SharedImpact,
    /// Overlap in historical co-changes.
    HistoricalCochange,
    /// Shared code responsibility or ownership.
    ResponsibilityConcentration,
}

/// A structured contribution from a specific factor to the BCS score.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BcsFactorContribution {
    kind: BcsFactorKind,
    source_identity: EvidenceIdentity,
    score: u16,
    explanation: String,
}

impl BcsFactorContribution {
    /// Creates a deterministic factor contribution.
    #[must_use]
    pub fn new(
        kind: BcsFactorKind,
        source_identity: EvidenceIdentity,
        score: u16,
        explanation: impl Into<String>,
    ) -> Self {
        Self { kind, source_identity, score, explanation: explanation.into() }
    }

    /// The kind of factor.
    #[must_use]
    pub const fn kind(&self) -> BcsFactorKind {
        self.kind
    }

    /// The score contributed by this factor.
    #[must_use]
    pub const fn score(&self) -> u16 {
        self.score
    }
    
    /// The structural explanation.
    #[must_use]
    pub fn explanation(&self) -> &str {
        &self.explanation
    }
}

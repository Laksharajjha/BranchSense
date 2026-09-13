//! Normalized evidence model for BCS consumption.

use serde::{Deserialize, Serialize};
use branchsense_semantic::EvidenceEnvelope;

/// Categories of evidence consumed by the BCS scoring engine.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BcsEvidenceCategory {
    /// Semantic collision between two concurrent modifications.
    Collision,
    /// Transitive semantic impact.
    Impact,
    /// Structural overlap between branches.
    Overlap,
    /// Shared historical contribution activity.
    History,
    /// Shared contributor ownership or responsibility.
    Ownership,
}

/// A normalized unit of semantic evidence.
///
/// This representation isolates the BCS layer from the internal implementation details
/// of the various analysis subsystems. It ensures all evidence is deterministically
/// structured and traceable.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BcsNormalizedEvidence {
    category: BcsEvidenceCategory,
    envelope: EvidenceEnvelope,
    affected_entities: Vec<String>,
    description: String,
    strength: u8,
}

impl BcsNormalizedEvidence {
    /// Constructs a normalized evidence record.
    #[must_use]
    pub fn new(
        category: BcsEvidenceCategory,
        envelope: EvidenceEnvelope,
        mut affected_entities: Vec<String>,
        description: String,
        strength: u8,
    ) -> Self {
        // Ensure deterministic ordering of affected entities.
        affected_entities.sort();
        Self {
            category,
            envelope,
            affected_entities,
            description,
            strength,
        }
    }

    /// The broad category of this evidence.
    #[must_use]
    pub const fn category(&self) -> BcsEvidenceCategory {
        self.category
    }

    /// The deterministic envelope carrying state, completeness, and provenance.
    #[must_use]
    pub const fn envelope(&self) -> &EvidenceEnvelope {
        &self.envelope
    }

    /// The entities (symbols, files) explicitly involved in this evidence.
    #[must_use]
    pub fn affected_entities(&self) -> &[String] {
        &self.affected_entities
    }

    /// A structured human-readable explanation of the evidence.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The raw strength scalar provided by the source subsystem (e.g. collision score).
    #[must_use]
    pub const fn strength(&self) -> u8 {
        self.strength
    }
}

//! Policy definition for BCS V1 ordinal scoring.

use crate::factor::{BcsFactorContribution, BcsFactorKind};
use crate::model::BcsOrdinalBand;
use crate::normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
use branchsense_semantic::EvidenceKind;

/// Explicit configuration rules for BCS V1.
#[derive(Clone, Debug)]
#[allow(clippy::struct_field_names)]
pub struct BcsPolicyV1 {
    low_threshold: u16,
    moderate_threshold: u16,
    high_threshold: u16,
    critical_threshold: u16,
}

impl BcsPolicyV1 {
    /// Creates the standard V1 policy configuration.
    #[must_use]
    pub const fn default() -> Self {
        Self {
            low_threshold: 10,
            moderate_threshold: 30,
            high_threshold: 60,
            critical_threshold: 90,
        }
    }

    /// Evaluates a bounded ordinal scalar into a semantic band.
    #[must_use]
    pub const fn evaluate_band(&self, score: u16) -> BcsOrdinalBand {
        if score >= self.critical_threshold {
            BcsOrdinalBand::Critical
        } else if score >= self.high_threshold {
            BcsOrdinalBand::High
        } else if score >= self.moderate_threshold {
            BcsOrdinalBand::Moderate
        } else if score >= self.low_threshold {
            BcsOrdinalBand::Low
        } else {
            BcsOrdinalBand::None
        }
    }

    /// Extracts explicit factors from normalized evidence.
    #[must_use]
    pub fn extract_factors(
        &self,
        evidence: &[BcsNormalizedEvidence],
    ) -> Vec<BcsFactorContribution> {
        let mut factors = Vec::new();

        for ev in evidence {
            let identity = branchsense_semantic::EvidenceIdentity::new(
                EvidenceKind::Primary,
                format!("{:?}", ev.category()),
                ev.affected_entities().to_vec(),
            );

            match ev.category() {
                BcsEvidenceCategory::Collision => {
                    factors.push(BcsFactorContribution::new(
                        BcsFactorKind::DirectCollision,
                        identity,
                        u16::from(ev.strength()) * 5, // Policy explicitly weights collisions
                        format!("Direct collision detected: {}", ev.description()),
                    ));
                }
                BcsEvidenceCategory::Impact => {
                    factors.push(BcsFactorContribution::new(
                        BcsFactorKind::SharedImpact,
                        identity,
                        u16::from(ev.strength()) * 2,
                        format!("Shared transitive impact: {}", ev.description()),
                    ));
                }
                BcsEvidenceCategory::History => {
                    factors.push(BcsFactorContribution::new(
                        BcsFactorKind::HistoricalCochange,
                        identity,
                        u16::from(ev.strength()),
                        format!("Historical co-change overlap: {}", ev.description()),
                    ));
                }
                BcsEvidenceCategory::Ownership => {
                    factors.push(BcsFactorContribution::new(
                        BcsFactorKind::ResponsibilityConcentration,
                        identity,
                        u16::from(ev.strength()),
                        format!("Responsibility overlap: {}", ev.description()),
                    ));
                }
                BcsEvidenceCategory::Overlap => {
                    // Overlap is structural, acts as a baseline multiplier if needed,
                    // or just a low base score.
                    factors.push(BcsFactorContribution::new(
                        BcsFactorKind::SharedImpact, // Treated as impact
                        identity,
                        u16::from(ev.strength()),
                        format!("Structural overlap: {}", ev.description()),
                    ));
                }
            }
        }

        factors
    }
}

impl Default for BcsPolicyV1 {
    fn default() -> Self {
        Self::default()
    }
}

//! Policy definition for BCS V1 ordinal scoring.

use crate::model::BcsOrdinalBand;

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
}

impl Default for BcsPolicyV1 {
    fn default() -> Self {
        Self::default()
    }
}

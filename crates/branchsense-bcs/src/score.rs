//! BCS assessment engine and execution logic.

use crate::ledger::BcsEvidenceAggregator;
use crate::model::{BcsAssessment, BcsExplanation};
use crate::policy::BcsPolicyV1;
use branchsense_semantic::{
    AbstentionDecision, AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceState,
};

/// The main engine responsible for aggregating evidence and scoring.
#[derive(Debug)]
pub struct BcsEngine {
    policy: BcsPolicyV1,
}

impl BcsEngine {
    /// Creates a new engine with the default V1 policy.
    #[must_use]
    pub const fn new() -> Self {
        Self { policy: BcsPolicyV1::default() }
    }

    /// Evaluates the aggregated evidence to produce a deterministic assessment.
    #[must_use]
    pub fn assess(&self, aggregator: &BcsEvidenceAggregator) -> BcsAssessment {
        let consolidated = aggregator.consolidate();

        let mut total_score: u16 = 0;
        let mut reasons = Vec::new();

        // 1. Calculate base ordinal score
        for ev in &consolidated {
            total_score = total_score.saturating_add(u16::from(ev.strength()));
            reasons.push(format!("Included {:?} evidence: {}", ev.category(), ev.description()));
        }

        // Cap at 100 for ordinal band logic (if our max is 100).
        if total_score > 100 {
            total_score = 100;
        }

        let mut band = self.policy.evaluate_band(total_score);

        // 2. Synthesize an overarching completeness / state / abstention
        let mut is_indeterminate = false;
        let mut abstention = None;

        // Naive evaluation: we check if any evidence carries a failed or indeterminate state
        for ev in &consolidated {
            let state = ev.envelope().state();
            if matches!(
                state,
                EvidenceState::Unavailable | EvidenceState::Unsupported | EvidenceState::Failed
            ) {
                is_indeterminate = true;
                reasons.push(format!("Abstaining due to {:?} evidence", state));
                abstention = Some(AbstentionDecision::Indeterminate);
                break;
            }
        }

        if is_indeterminate {
            band = crate::model::BcsOrdinalBand::Indeterminate;
        }

        // TODO: In a more rigorous implementation, we should extract the global
        // EvidenceCompleteness and AbstentionDecision from the input snapshots
        // but for V1 we will synthesize a simple envelope.
        let comp = EvidenceCompleteness::new();
        let prov = AnalysisProvenance::new();
        let env = EvidenceEnvelope::new(EvidenceState::Observed, comp, prov);

        BcsAssessment::new(band, total_score, env, abstention, BcsExplanation::new(reasons))
    }
}

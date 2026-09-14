//! BCS assessment engine and execution logic.

use crate::ledger::BcsEvidenceAggregator;
use crate::model::{BcsAssessment, BcsExplanation};
use crate::policy::BcsPolicyV1;
use branchsense_semantic::{
    AbstentionDecision, AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceState,
};

/// The main engine responsible for aggregating evidence and scoring.
#[derive(Debug, Default)]
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

        // 1. Synthesize an overarching completeness / state / abstention
        let mut is_indeterminate = false;
        let mut abstention = None;
        let mut reasons = Vec::new();

        let mut combined_state = EvidenceState::NoEvidence;

        // Evaluate state and abstention gates
        for ev in &consolidated {
            let state = ev.envelope().state();
            combined_state = combined_state.combine(state);

            // Indeterminate rules:
            if matches!(
                state,
                EvidenceState::Unavailable
                    | EvidenceState::Failed
                    | EvidenceState::Ambiguous
                    | EvidenceState::Unresolved
            ) {
                is_indeterminate = true;
                reasons.push(format!("Abstaining due to untrustworthy evidence state: {state:?}"));
                abstention = Some(AbstentionDecision::Indeterminate);
            }
            // Truncated triggers a warn but allows proceed
            if state == EvidenceState::Truncated {
                reasons.push("Warning: BCS analysis was based on truncated evidence".to_owned());
                if abstention.is_none() {
                    abstention = Some(AbstentionDecision::Warn);
                }
            }
        }

        if combined_state == EvidenceState::NoEvidence {
            abstention = Some(AbstentionDecision::Proceed);
        }

        // 2. Extract explicit deterministic factors via Policy
        let factors = self.policy.extract_factors(&consolidated);
        let mut total_score: u16 = 0;

        for factor in &factors {
            total_score = total_score.saturating_add(factor.score());
            reasons.push(format!("Factor [{}]: {}", factor.score(), factor.explanation()));
        }

        // Cap at 100 for ordinal band logic.
        if total_score > 100 {
            total_score = 100;
        }

        let mut band = self.policy.evaluate_band(total_score);

        if is_indeterminate {
            band = crate::model::BcsOrdinalBand::Indeterminate;
            total_score = 0;
        }

        let comp = EvidenceCompleteness::new();
        let prov = AnalysisProvenance::new();
        let env = EvidenceEnvelope::new(combined_state, comp, prov);

        BcsAssessment::new(band, total_score, env, abstention, BcsExplanation::new(reasons))
    }
}

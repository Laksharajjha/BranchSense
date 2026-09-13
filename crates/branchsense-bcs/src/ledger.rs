//! Evidence ledger integration and deduplication for BCS.

use crate::normalization::BcsNormalizedEvidence;
use branchsense_semantic::EvidenceLedger;

/// Aggregates and deduplicates normalized evidence from multiple subsystems.
#[derive(Clone, Debug, Default)]
pub struct BcsEvidenceAggregator {
    raw_evidence: Vec<BcsNormalizedEvidence>,
}

impl BcsEvidenceAggregator {
    /// Creates a new empty aggregator.
    #[must_use]
    pub const fn new() -> Self {
        Self { raw_evidence: Vec::new() }
    }

    /// Adds a piece of normalized evidence to the aggregator.
    pub fn add(&mut self, evidence: BcsNormalizedEvidence) {
        self.raw_evidence.push(evidence);
    }

    /// Returns the raw (non-deduplicated) list of evidence.
    #[must_use]
    pub fn raw_evidence(&self) -> &[BcsNormalizedEvidence] {
        &self.raw_evidence
    }

    /// Consolidates all evidence using a shared ledger to remove duplicate observations.
    ///
    /// Double-counting is avoided by merging the underlying EvidenceEnvelope identities
    /// into an EvidenceLedger. If a normalized evidence item provides no new unique
    /// observations, it is considered fully redundant and excluded from the final scoring set.
    #[must_use]
    pub fn consolidate(&self) -> Vec<BcsNormalizedEvidence> {
        let mut unified_ledger = EvidenceLedger::new();
        let mut consolidated = Vec::new();

        // Sort by strength descending to keep the strongest evidence when deduplicating
        let mut sorted = self.raw_evidence.clone();
        sorted.sort_by(|a, b| b.strength().cmp(&a.strength()));

        for ev in sorted {
            let mut provided_new_observation = false;

            for identity in ev.envelope().identities() {
                if unified_ledger.insert_identity(identity.clone()) {
                    provided_new_observation = true;
                }
            }

            // Even if it didn't provide new identities, maybe it's the only evidence
            // and we shouldn't throw it away if it has no identities at all.
            // But structurally, valid evidence should have identities.
            // For now, if it provides new observations OR has no identities (baseline), we keep it.
            if provided_new_observation || ev.envelope().identities().len() == 0 {
                consolidated.push(ev);
            }
        }

        consolidated
    }
}

use branchsense_bcs::ledger::BcsEvidenceAggregator;
use branchsense_bcs::normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
use branchsense_semantic::{
    AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceState,
};

#[test]
fn test_deduplication_keeps_highest_strength_first() {
    let mut aggregator = BcsEvidenceAggregator::new();
    let prov = AnalysisProvenance::new();
    let comp = EvidenceCompleteness::new();
    let env = EvidenceEnvelope::new(EvidenceState::Observed, comp, prov);

    let ev1 = BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Impact,
        env.clone(),
        vec![],
        "A".into(),
        10,
    );
    let ev2 = BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Overlap,
        env.clone(),
        vec![],
        "B".into(),
        50,
    );

    aggregator.add(ev1);
    aggregator.add(ev2);

    let consolidated = aggregator.consolidate();
    assert_eq!(consolidated.len(), 2); // since identities are empty, both are kept
    assert_eq!(consolidated[0].strength(), 50);
    assert_eq!(consolidated[1].strength(), 10);
}

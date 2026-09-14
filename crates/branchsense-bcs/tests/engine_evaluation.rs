//! Engine evaluation tests.
use branchsense_bcs::BcsOrdinalBand;
use branchsense_bcs::ledger::BcsEvidenceAggregator;
use branchsense_bcs::normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
use branchsense_bcs::score::BcsEngine;
use branchsense_semantic::{
    AbstentionDecision, AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceState,
};

#[test]
fn test_engine_evaluates_clean_evidence() {
    let engine = BcsEngine::new();
    let mut aggregator = BcsEvidenceAggregator::new();

    let env = EvidenceEnvelope::new(
        EvidenceState::Observed,
        EvidenceCompleteness::new(),
        AnalysisProvenance::new(),
    );
    aggregator.add(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Collision,
        env,
        vec![],
        "A".into(),
        9, // 9 * 5 = 45
    ));

    let assessment = engine.assess(&aggregator);
    assert_eq!(assessment.ordinal_score(), 45);
    assert_eq!(assessment.band(), BcsOrdinalBand::Moderate);
}

#[test]
fn test_engine_abtains_on_unavailable_evidence() {
    let engine = BcsEngine::new();
    let mut aggregator = BcsEvidenceAggregator::new();

    let env = EvidenceEnvelope::new(
        EvidenceState::Unavailable,
        EvidenceCompleteness::new(),
        AnalysisProvenance::new(),
    );
    aggregator.add(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::History,
        env,
        vec![],
        "A".into(),
        50,
    ));

    let assessment = engine.assess(&aggregator);
    assert_eq!(assessment.band(), BcsOrdinalBand::Indeterminate);
    assert_eq!(assessment.abstention(), Some(&AbstentionDecision::Indeterminate));
}

#[test]
fn test_engine_warns_on_truncated_evidence() {
    let engine = BcsEngine::new();
    let mut aggregator = BcsEvidenceAggregator::new();

    let env = EvidenceEnvelope::new(
        EvidenceState::Truncated,
        EvidenceCompleteness::new(),
        AnalysisProvenance::new(),
    );
    aggregator.add(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::History,
        env,
        vec![],
        "A".into(),
        10,
    ));

    let assessment = engine.assess(&aggregator);
    assert_eq!(assessment.abstention(), Some(&AbstentionDecision::Warn));
}

#[test]
fn test_engine_caps_at_100() {
    let engine = BcsEngine::new();
    let mut aggregator = BcsEvidenceAggregator::new();

    let env = EvidenceEnvelope::new(
        EvidenceState::Observed,
        EvidenceCompleteness::new(),
        AnalysisProvenance::new(),
    );
    aggregator.add(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Collision,
        env.clone(),
        vec![],
        "A".into(),
        20, // 20 * 5 = 100
    ));
    aggregator.add(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Impact,
        env,
        vec![],
        "B".into(),
        10, // 10 * 2 = 20 -> total 120 -> capped at 100
    ));

    let assessment = engine.assess(&aggregator);
    assert_eq!(assessment.ordinal_score(), 100);
    assert_eq!(assessment.band(), BcsOrdinalBand::Critical);
}

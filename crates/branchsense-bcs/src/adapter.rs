//! Adapters for converting subsystem-specific models into normalized BCS evidence.

use crate::normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
use branchsense_collision::CollisionAssessment;
use branchsense_history::HistoricalSignals;
use branchsense_impact::ImpactSet;
use branchsense_overlap::OverlapSet;
use branchsense_ownership::ResponsibilitySignals;

/// Normalize a CollisionAssessment into standard evidence.
#[must_use]
pub fn normalize_collision(assessment: &CollisionAssessment) -> Vec<BcsNormalizedEvidence> {
    let mut evidence = Vec::new();
    let mut entities = Vec::new();
    for explanation in assessment.explanations() {
        entities.push(explanation.summary().to_owned());
    }

    evidence.push(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Collision,
        assessment.evidence().clone(),
        entities,
        format!("{:?} collision", assessment.severity()),
        assessment.evidence_score(),
    ));
    evidence
}

/// Normalize an ImpactSet into standard evidence.
#[must_use]
pub fn normalize_impact(impact: &ImpactSet) -> Vec<BcsNormalizedEvidence> {
    let mut evidence = Vec::new();
    let entities: Vec<String> =
        impact.entries().iter().map(|e| e.impacted_symbol().as_str().to_owned()).collect();

    evidence.push(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Impact,
        impact.evidence().clone(),
        entities,
        "Transitive semantic impact".to_owned(),
        10,
    ));
    evidence
}

/// Normalize an OverlapSet into standard evidence.
#[must_use]
pub fn normalize_overlap(overlap: &OverlapSet) -> Vec<BcsNormalizedEvidence> {
    let mut evidence = Vec::new();
    let mut entities: Vec<String> = Vec::new();
    for e in overlap.entries() {
        for target in e.explanation().targets() {
            entities.push(target.as_str().to_owned());
        }
    }

    evidence.push(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Overlap,
        overlap.evidence().clone(),
        entities,
        "Structural overlap".to_owned(),
        5,
    ));
    evidence
}

/// Normalize HistoricalSignals into standard evidence.
#[must_use]
pub fn normalize_history(signals: &HistoricalSignals) -> Vec<BcsNormalizedEvidence> {
    let mut evidence = Vec::new();
    let mut entities = Vec::new();
    for signal in signals.symbol_co_change() {
        entities.push(format!(
            "{} & {}",
            signal.left().qualified_name(),
            signal.right().qualified_name()
        ));
    }

    evidence.push(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::History,
        signals.evidence().clone(),
        entities,
        "Historical co-changes".to_owned(),
        5,
    ));
    evidence
}

/// Normalize ResponsibilitySignals into standard evidence.
#[must_use]
pub fn normalize_ownership(signals: &ResponsibilitySignals) -> Vec<BcsNormalizedEvidence> {
    let mut evidence = Vec::new();
    let entities = Vec::new();

    evidence.push(BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Ownership,
        signals.evidence().clone(),
        entities,
        "Responsibility overlap".to_owned(),
        5,
    ));
    evidence
}

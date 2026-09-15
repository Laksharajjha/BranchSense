//! Adapters for converting subsystem-specific models into normalized BCS evidence.

use crate::normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
use branchsense_collision::CollisionAssessment;
use branchsense_history::HistoricalSignals;
use branchsense_impact::ImpactSet;
use branchsense_overlap::OverlapSet;
use branchsense_ownership::ResponsibilitySignals;

/// Normalize a `CollisionAssessment` into standard evidence.
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

/// Normalize an `ImpactSet` into standard evidence.
#[must_use]
pub fn normalize_impact(impact: &ImpactSet) -> Vec<BcsNormalizedEvidence> {
    if impact.is_empty() {
        return Vec::new();
    }

    let mut entities: Vec<String> =
        impact.entries().iter().map(|e| e.impacted_symbol().as_str().to_owned()).collect();

    // Deduplicate entities
    entities.sort();
    entities.dedup();

    let mut signature_consumers = 0;
    for entry in impact.entries() {
        for cause in entry.causes() {
            if cause.explanation().kind() == branchsense_impact::ImpactKind::SignatureConsumer {
                signature_consumers += 1;
            }
        }
    }

    let stats = impact.statistics();
    let mut strength: u32 = 0;

    strength += (stats.direct_impacts() as u32) * 5;
    strength += (stats.transitive_impacts() as u32) * 2;
    strength += signature_consumers * 10;
    
    if stats.truncated() {
        strength += 20;
    }

    // Ensure we provide a non-zero minimum if there are entries
    let strength_u8 = std::cmp::max(1, std::cmp::min(100, strength)) as u8;

    let description = if signature_consumers > 0 {
        format!("Semantic impact with {} signature consumer(s)", signature_consumers)
    } else if stats.transitive_impacts() > 0 {
        format!("Transitive semantic impact (depth {})", stats.max_depth())
    } else {
        "Direct semantic impact".to_owned()
    };

    vec![BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Impact,
        impact.evidence().clone(),
        entities,
        description,
        strength_u8,
    )]
}

/// Normalize an `OverlapSet` into standard evidence.
#[must_use]
pub fn normalize_overlap(overlap: &OverlapSet) -> Vec<BcsNormalizedEvidence> {
    if overlap.is_empty() {
        return Vec::new();
    }

    let mut entities: Vec<String> = Vec::new();
    let mut direct_changes = 0;
    let mut impact_changes = 0;
    let mut cross_impacts = 0;
    let mut shared_impacts = 0;

    for e in overlap.entries() {
        match e.explanation().kind() {
            branchsense_overlap::OverlapKind::DirectChange => direct_changes += 1,
            branchsense_overlap::OverlapKind::ImpactChange => impact_changes += 1,
            branchsense_overlap::OverlapKind::CrossImpact => cross_impacts += 1,
            branchsense_overlap::OverlapKind::SharedImpact => shared_impacts += 1,
        }
        for target in e.explanation().targets() {
            entities.push(target.as_str().to_owned());
        }
    }

    entities.sort();
    entities.dedup();

    let mut strength: u32 = 0;
    // Direct change is the strongest form of overlap.
    strength += direct_changes * 30;
    // Cross/Impact changes show direct dependencies between changed logic.
    strength += impact_changes * 15;
    strength += cross_impacts * 15;
    // Shared impact shows common downstream dependencies.
    strength += shared_impacts * 5;

    let strength_u8 = std::cmp::max(1, std::cmp::min(100, strength)) as u8;
    
    let description = if direct_changes > 0 {
        format!("Direct structural overlap ({} symbol(s))", direct_changes)
    } else if impact_changes > 0 || cross_impacts > 0 {
        "Direct causal overlap".to_owned()
    } else {
        "Shared downstream impact".to_owned()
    };

    vec![BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Overlap,
        overlap.evidence().clone(),
        entities,
        description,
        strength_u8,
    )]
}

/// Normalize `HistoricalSignals` into standard evidence.
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

/// Normalize `ResponsibilitySignals` into standard evidence.
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

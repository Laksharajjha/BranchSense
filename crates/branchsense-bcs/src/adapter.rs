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
    let mut entities = Vec::new();
    let mut total_co_changes = 0;
    
    for signal in signals.symbol_co_change() {
        entities.push(format!(
            "{} & {}",
            signal.left().qualified_name(),
            signal.right().qualified_name()
        ));
        total_co_changes += signal.co_change_count();
    }
    for signal in signals.file_co_change() {
        entities.push(format!(
            "{} & {}",
            signal.left().display(),
            signal.right().display()
        ));
        total_co_changes += signal.co_change_count();
    }

    if entities.is_empty() && signals.evidence().state() == branchsense_semantic::EvidenceState::Observed {
        // If there are no entities and the evidence is fully observed, it's just an empty set of historical signals.
        return Vec::new();
    }

    entities.sort();
    entities.dedup();

    let mut strength: u32 = 0;

    // Base strength from co-changes
    strength += (total_co_changes as u32) * 5;

    // Recency boost: if there is recent activity (age_in_commits <= 10), add strength
    let mut recency_boost = 0;
    for signal in signals.recency() {
        if signal.age_in_commits() <= 10 {
            recency_boost += 5;
        }
    }
    strength += recency_boost;

    // Truncation means there might be more history we couldn't analyze
    if signals.evidence().state() == branchsense_semantic::EvidenceState::Truncated {
        strength += 10;
    }

    // Minimum strength if we generated evidence but calculated 0
    let strength_u8 = std::cmp::max(1, std::cmp::min(100, strength)) as u8;

    let description = if total_co_changes > 0 {
        if recency_boost > 0 {
            format!("Recent historical co-changes ({} instance(s))", total_co_changes)
        } else {
            format!("Historical co-changes ({} instance(s))", total_co_changes)
        }
    } else if signals.evidence().state() == branchsense_semantic::EvidenceState::Truncated {
        "Truncated history".to_owned()
    } else {
        "Historical signals".to_owned()
    };

    vec![BcsNormalizedEvidence::new(
        BcsEvidenceCategory::History,
        signals.evidence().clone(),
        entities,
        description,
        strength_u8,
    )]
}

/// Normalize `ResponsibilitySignals` into standard evidence.
#[must_use]
pub fn normalize_ownership(signals: &ResponsibilitySignals) -> Vec<BcsNormalizedEvidence> {
    let mut entities = Vec::new();
    let mut total_active_contributors = 0;
    let mut highest_concentration: f64 = 0.0;

    for ev in signals.symbol_responsibility() {
        match ev.entity() {
            branchsense_ownership::ResponsibilityEntity::Symbol(key) => {
                entities.push(key.qualified_name().to_owned());
            }
            branchsense_ownership::ResponsibilityEntity::File(path) => {
                entities.push(path.display().to_string());
            }
        }
        total_active_contributors += ev.concentration().active_contributors();
        let share = ev.concentration().top_contributor_share();
        if share > highest_concentration {
            highest_concentration = share;
        }
    }

    for ev in signals.file_responsibility() {
        match ev.entity() {
            branchsense_ownership::ResponsibilityEntity::Symbol(key) => {
                entities.push(key.qualified_name().to_owned());
            }
            branchsense_ownership::ResponsibilityEntity::File(path) => {
                entities.push(path.display().to_string());
            }
        }
        total_active_contributors += ev.concentration().active_contributors();
        let share = ev.concentration().top_contributor_share();
        if share > highest_concentration {
            highest_concentration = share;
        }
    }

    if entities.is_empty() && signals.evidence().state() == branchsense_semantic::EvidenceState::Observed {
        return Vec::new();
    }

    entities.sort();
    entities.dedup();

    let mut strength: u32 = 0;

    // Scale strength by highest concentration (0-1) * 20.
    if highest_concentration > 0.0 {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let bonus = (highest_concentration * 20.0) as u32;
        strength += bonus;
    }

    if total_active_contributors > 0 {
        strength += std::cmp::min(10, total_active_contributors as u32);
    }

    if signals.evidence().state() == branchsense_semantic::EvidenceState::Truncated {
        strength += 5;
    }

    let strength_u8 = std::cmp::max(1, std::cmp::min(100, strength)) as u8;

    let description = if highest_concentration > 0.5 {
        "Strong responsibility concentration".to_owned()
    } else if total_active_contributors > 0 {
        "Distributed responsibility".to_owned()
    } else if signals.evidence().state() == branchsense_semantic::EvidenceState::Truncated {
        "Truncated ownership data".to_owned()
    } else {
        "Responsibility signals".to_owned()
    };

    vec![BcsNormalizedEvidence::new(
        BcsEvidenceCategory::Ownership,
        signals.evidence().clone(),
        entities,
        description,
        strength_u8,
    )]
}

#![allow(clippy::cast_possible_truncation)]

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

    // Ensure we provide a non-zero minimum if there are entries
    let strength_u8 = strength.clamp(1, 100) as u8;

    let description = if signature_consumers > 0 {
        format!("Semantic impact with {signature_consumers} signature consumer(s)")
    } else if stats.transitive_impacts() > 0 {
        format!("Transitive semantic impact (depth {})", stats.max_depth())
    } else if stats.direct_impacts() > 0 {
        "Direct semantic impact".to_owned()
    } else if stats.truncated() {
        "Truncated semantic impact".to_owned()
    } else {
        "Semantic impact".to_owned()
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
    for e in overlap.entries() {
        for target in e.explanation().targets() {
            entities.push(target.as_str().to_owned());
        }
    }
    entities.sort();
    entities.dedup();

    let stats = overlap.statistics();
    let mut strength: u32 = 0;

    // Direct change is the strongest form of overlap.
    strength += (stats.direct_changes() as u32) * 30;
    // Cross/Impact changes show direct dependencies between changed logic.
    strength += (stats.impact_changes() as u32) * 15;
    strength += (stats.cross_impacts() as u32) * 15;
    // Shared impact shows common downstream dependencies.
    strength += (stats.shared_impacts() as u32) * 5;

    let strength_u8 = strength.clamp(1, 100) as u8;

    let description = if stats.direct_changes() > 0 {
        format!("Direct structural overlap ({} symbol(s))", stats.direct_changes())
    } else if stats.impact_changes() > 0 || stats.cross_impacts() > 0 {
        "Direct causal overlap".to_owned()
    } else if stats.shared_impacts() > 0 {
        "Shared downstream impact".to_owned()
    } else if stats.truncated() {
        "Truncated overlap analysis".to_owned()
    } else {
        "Semantic overlap".to_owned()
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
        entities.push(format!("{} & {}", signal.left().display(), signal.right().display()));
        total_co_changes += signal.co_change_count();
    }

    if entities.is_empty()
        && signals.evidence().state() == branchsense_semantic::EvidenceState::Observed
    {
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

    // Minimum strength if we generated evidence but calculated 0
    let strength_u8 = strength.clamp(1, 100) as u8;

    let description = if total_co_changes > 0 {
        if recency_boost > 0 {
            format!("Recent historical co-changes ({total_co_changes} instance(s))")
        } else {
            format!("Historical co-changes ({total_co_changes} instance(s))")
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

    if entities.is_empty()
        && signals.evidence().state() == branchsense_semantic::EvidenceState::Observed
    {
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

    let strength_u8 = strength.clamp(1, 100) as u8;

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

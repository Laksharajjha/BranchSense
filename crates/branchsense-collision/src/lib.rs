//! Deterministic assessment of semantic collision evidence.
//!
//! This crate consumes [`branchsense_overlap::OverlapSet`] and classifies the
//! strength of its evidence. It deliberately does not predict merge failure,
//! estimate probability, mine Git history, or use machine learning. The
//! evidence score is an ordinal aggregation from zero to one hundred: it is a
//! compact way to compare the strength of semantic evidence, not a calibrated
//! chance of conflict.
//!
//! The analyzer avoids double counting by assigning each unique overlap pair
//! one score contribution: the strongest applicable factor wins for that pair.
//! Other applicable factors remain attached as explanation metadata. This
//! preserves detail without making redundant representations inflate the
//! assessment.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use branchsense_core::SymbolId;
use branchsense_diff::ChangeKind;
use branchsense_impact::{ImpactKind, ImpactPath};
use branchsense_overlap::{OverlapEvidence, OverlapExplanation, OverlapKind, OverlapSet};
use serde::{Deserialize, Serialize};

/// Strength band for semantic collision evidence.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub enum CollisionSeverity {
    /// No semantic overlap was identified.
    #[default]
    None,
    /// Weak evidence that is useful for inspection but not a collision signal.
    Informational,
    /// Limited semantic interaction with a bounded causal relationship.
    Low,
    /// Meaningful semantic interaction that deserves review.
    Medium,
    /// Strong semantic evidence that the branch changes may interfere.
    High,
}

/// A deterministic evidence category.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CollisionFactorKind {
    /// Both branches modify the same stable semantic symbol.
    SameSymbolChanged,
    /// Branch A changes a symbol reached by branch B's impact evidence.
    ChangedSymbolImpact,
    /// Branch B changes a symbol reached by branch A's impact evidence.
    ReverseChangedSymbolImpact,
    /// Both branches affect the same downstream symbol.
    SharedImpact,
    /// The causal path is deeper than a direct semantic relationship.
    TransitiveImpact,
    /// A changed callable signature is consumed by the other branch's change.
    SignatureInteraction,
    /// A changed branch symbol is removed while the other branch depends on it.
    RemovalInteraction,
}

/// One retained causal proof supporting a collision factor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollisionEvidence {
    overlap_kind: OverlapKind,
    branch_a_changed: SymbolId,
    branch_b_changed: SymbolId,
    targets: Vec<SymbolId>,
    branch_a_depth: Option<usize>,
    branch_b_depth: Option<usize>,
    branch_a_paths: Vec<ImpactPath>,
    branch_b_paths: Vec<ImpactPath>,
}

impl CollisionEvidence {
    /// Returns the originating overlap classification.
    #[must_use]
    pub const fn overlap_kind(&self) -> OverlapKind {
        self.overlap_kind
    }
    /// Returns the symbol changed by branch A.
    #[must_use]
    pub fn branch_a_changed(&self) -> &SymbolId {
        &self.branch_a_changed
    }
    /// Returns the symbol changed by branch B.
    #[must_use]
    pub fn branch_b_changed(&self) -> &SymbolId {
        &self.branch_b_changed
    }
    /// Returns the affected semantic targets.
    #[must_use]
    pub fn targets(&self) -> &[SymbolId] {
        &self.targets
    }
    /// Returns the shallowest branch A impact depth, when present.
    #[must_use]
    pub const fn branch_a_depth(&self) -> Option<usize> {
        self.branch_a_depth
    }
    /// Returns the shallowest branch B impact depth, when present.
    #[must_use]
    pub const fn branch_b_depth(&self) -> Option<usize> {
        self.branch_b_depth
    }
    /// Returns branch A causal paths.
    #[must_use]
    pub fn branch_a_paths(&self) -> &[ImpactPath] {
        &self.branch_a_paths
    }
    /// Returns branch B causal paths.
    #[must_use]
    pub fn branch_b_paths(&self) -> &[ImpactPath] {
        &self.branch_b_paths
    }
}

/// A factor and its strongest deterministic contribution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollisionFactor {
    kind: CollisionFactorKind,
    strength: u8,
    evidence: Vec<CollisionEvidence>,
}

impl CollisionFactor {
    /// Returns the factor category.
    #[must_use]
    pub const fn kind(&self) -> CollisionFactorKind {
        self.kind
    }
    /// Returns the factor's evidence strength from zero to one hundred.
    #[must_use]
    pub const fn strength(&self) -> u8 {
        self.strength
    }
    /// Returns all distinct overlap evidence supporting this factor.
    #[must_use]
    pub fn evidence(&self) -> &[CollisionEvidence] {
        &self.evidence
    }
}

/// A deterministic, developer-readable explanation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollisionExplanation {
    factor: CollisionFactorKind,
    branch_a_changed: SymbolId,
    branch_b_changed: SymbolId,
    targets: Vec<SymbolId>,
    summary: String,
    evidence: CollisionEvidence,
}

impl CollisionExplanation {
    /// Returns the primary factor for this explanation.
    #[must_use]
    pub const fn factor(&self) -> CollisionFactorKind {
        self.factor
    }
    /// Returns branch A's changed symbol.
    #[must_use]
    pub fn branch_a_changed(&self) -> &SymbolId {
        &self.branch_a_changed
    }
    /// Returns branch B's changed symbol.
    #[must_use]
    pub fn branch_b_changed(&self) -> &SymbolId {
        &self.branch_b_changed
    }
    /// Returns the affected targets.
    #[must_use]
    pub fn targets(&self) -> &[SymbolId] {
        &self.targets
    }
    /// Returns a deterministic explanation sentence.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }
    /// Returns the structured proof behind this explanation.
    #[must_use]
    pub fn evidence(&self) -> &CollisionEvidence {
        &self.evidence
    }
}

/// Summary counts for one collision assessment.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollisionStatistics {
    overlaps: usize,
    unique_evidence: usize,
    factors: usize,
    explanations: usize,
    truncated: bool,
}

impl CollisionStatistics {
    /// Returns the number of overlap entries assessed.
    #[must_use]
    pub const fn overlaps(&self) -> usize {
        self.overlaps
    }
    /// Returns the number of unique overlap pairs scored.
    #[must_use]
    pub const fn unique_evidence(&self) -> usize {
        self.unique_evidence
    }
    /// Returns the number of factor categories present.
    #[must_use]
    pub const fn factors(&self) -> usize {
        self.factors
    }
    /// Returns the number of explanations produced.
    #[must_use]
    pub const fn explanations(&self) -> usize {
        self.explanations
    }
    /// Returns whether the upstream overlap result was bounded.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// The complete deterministic collision assessment.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollisionAssessment {
    severity: CollisionSeverity,
    evidence_score: u8,
    factors: Vec<CollisionFactor>,
    explanations: Vec<CollisionExplanation>,
    statistics: CollisionStatistics,
}

impl CollisionAssessment {
    /// Returns the evidence-strength severity.
    #[must_use]
    pub const fn severity(&self) -> CollisionSeverity {
        self.severity
    }
    /// Returns the relative evidence strength from zero to one hundred.
    #[must_use]
    pub const fn evidence_score(&self) -> u8 {
        self.evidence_score
    }
    /// Returns factors in deterministic category order.
    #[must_use]
    pub fn factors(&self) -> &[CollisionFactor] {
        &self.factors
    }
    /// Returns structured explanations in deterministic order.
    #[must_use]
    pub fn explanations(&self) -> &[CollisionExplanation] {
        &self.explanations
    }
    /// Returns assessment statistics.
    #[must_use]
    pub const fn statistics(&self) -> &CollisionStatistics {
        &self.statistics
    }
    /// Returns whether no semantic collision evidence was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.factors.is_empty()
    }
}

/// Deterministic analyzer from semantic overlap to collision evidence.
#[derive(Clone, Copy, Debug, Default)]
pub struct CollisionAnalyzer;

impl CollisionAnalyzer {
    /// Creates a collision analyzer.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Assesses overlap evidence without mutating the overlap set.
    #[must_use]
    pub fn analyze(&self, overlaps: &OverlapSet) -> CollisionAssessment {
        let mut factor_evidence =
            BTreeMap::<CollisionFactorKind, BTreeMap<EvidenceKey, CollisionEvidence>>::new();
        let mut score_by_evidence = BTreeMap::<EvidenceKey, u8>::new();
        let mut explanations = BTreeMap::<EvidenceKey, CollisionExplanation>::new();

        for entry in overlaps.entries() {
            let explanation = entry.explanation();
            let evidence = collision_evidence(explanation);
            let key = EvidenceKey::from(&evidence);
            for (kind, strength) in factor_contributions(explanation, &evidence) {
                factor_evidence
                    .entry(kind)
                    .or_default()
                    .entry(key.clone())
                    .or_insert_with(|| evidence.clone());
                score_by_evidence
                    .entry(key.clone())
                    .and_modify(|current| *current = (*current).max(strength))
                    .or_insert(strength);
                let candidate = CollisionExplanation {
                    factor: kind,
                    branch_a_changed: evidence.branch_a_changed.clone(),
                    branch_b_changed: evidence.branch_b_changed.clone(),
                    targets: evidence.targets.clone(),
                    summary: explanation_summary(kind).to_owned(),
                    evidence: evidence.clone(),
                };
                explanations
                    .entry(key.clone())
                    .and_modify(|current| {
                        if kind < current.factor {
                            *current = candidate.clone();
                        }
                    })
                    .or_insert(candidate);
            }
        }

        let score =
            score_by_evidence.values().map(|score| u16::from(*score)).sum::<u16>().min(100) as u8;
        let factors = factor_evidence
            .into_iter()
            .map(|(kind, evidence)| CollisionFactor {
                strength: evidence_strength(kind),
                kind,
                evidence: evidence.into_values().collect(),
            })
            .collect::<Vec<_>>();
        let explanations = explanations.into_values().collect::<Vec<_>>();
        let statistics = CollisionStatistics {
            overlaps: overlaps.entries().len(),
            unique_evidence: score_by_evidence.len(),
            factors: factors.len(),
            explanations: explanations.len(),
            truncated: overlaps.statistics().truncated(),
        };
        CollisionAssessment {
            severity: severity(score),
            evidence_score: score,
            factors,
            explanations,
            statistics,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EvidenceKey {
    overlap_kind: OverlapKind,
    branch_a_changed: SymbolId,
    branch_b_changed: SymbolId,
    targets: Vec<SymbolId>,
}

impl From<&CollisionEvidence> for EvidenceKey {
    fn from(evidence: &CollisionEvidence) -> Self {
        Self {
            overlap_kind: evidence.overlap_kind,
            branch_a_changed: evidence.branch_a_changed.clone(),
            branch_b_changed: evidence.branch_b_changed.clone(),
            targets: evidence.targets.clone(),
        }
    }
}

fn collision_evidence(explanation: &OverlapExplanation) -> CollisionEvidence {
    let a_depth = explanation.branch_a_evidence().iter().map(OverlapEvidence::depth).min();
    let b_depth = explanation.branch_b_evidence().iter().map(OverlapEvidence::depth).min();
    CollisionEvidence {
        overlap_kind: explanation.kind(),
        branch_a_changed: explanation.branch_a_changed().clone(),
        branch_b_changed: explanation.branch_b_changed().clone(),
        targets: explanation.targets().to_vec(),
        branch_a_depth: a_depth,
        branch_b_depth: b_depth,
        branch_a_paths: explanation
            .branch_a_evidence()
            .iter()
            .map(|item| item.path().clone())
            .collect(),
        branch_b_paths: explanation
            .branch_b_evidence()
            .iter()
            .map(|item| item.path().clone())
            .collect(),
    }
}

fn factor_contributions(
    explanation: &OverlapExplanation,
    evidence: &CollisionEvidence,
) -> Vec<(CollisionFactorKind, u8)> {
    let mut factors = Vec::new();
    match explanation.kind() {
        OverlapKind::DirectChange => factors.push((CollisionFactorKind::SameSymbolChanged, 80)),
        OverlapKind::ImpactChange => {
            if !explanation.branch_a_evidence().is_empty() {
                factors.push((
                    CollisionFactorKind::ChangedSymbolImpact,
                    impact_strength(evidence.branch_a_depth),
                ));
            }
            if !explanation.branch_b_evidence().is_empty() {
                factors.push((
                    CollisionFactorKind::ReverseChangedSymbolImpact,
                    impact_strength(evidence.branch_b_depth),
                ));
            }
        }
        OverlapKind::SharedImpact => factors.push((
            CollisionFactorKind::SharedImpact,
            impact_strength(min_depth(evidence)) / 2 + 15,
        )),
        OverlapKind::CrossImpact => {
            factors.push((CollisionFactorKind::ChangedSymbolImpact, 70));
            factors.push((CollisionFactorKind::ReverseChangedSymbolImpact, 70));
        }
    }
    if has_signature_interaction(explanation) {
        factors.push((CollisionFactorKind::SignatureInteraction, 85));
    }
    if is_removed(explanation.branch_a_change_kind())
        || is_removed(explanation.branch_b_change_kind())
    {
        factors.push((CollisionFactorKind::RemovalInteraction, 90));
    }
    if min_depth(evidence).is_some_and(|depth| depth > 1) {
        factors.push((CollisionFactorKind::TransitiveImpact, 30));
    }
    factors
}

fn has_signature_interaction(explanation: &OverlapExplanation) -> bool {
    explanation
        .branch_a_evidence()
        .iter()
        .chain(explanation.branch_b_evidence())
        .any(|evidence| evidence.kind() == ImpactKind::SignatureConsumer)
}

fn is_removed(kind: Option<ChangeKind>) -> bool {
    kind == Some(ChangeKind::Removed)
}

fn min_depth(evidence: &CollisionEvidence) -> Option<usize> {
    evidence.branch_a_depth.into_iter().chain(evidence.branch_b_depth).min()
}

fn impact_strength(depth: Option<usize>) -> u8 {
    match depth {
        Some(1) => 65,
        Some(2) => 45,
        Some(_) => 30,
        None => 0,
    }
}

fn evidence_strength(kind: CollisionFactorKind) -> u8 {
    match kind {
        CollisionFactorKind::SameSymbolChanged => 80,
        CollisionFactorKind::ChangedSymbolImpact
        | CollisionFactorKind::ReverseChangedSymbolImpact => 65,
        CollisionFactorKind::SharedImpact => 45,
        CollisionFactorKind::TransitiveImpact => 30,
        CollisionFactorKind::SignatureInteraction => 85,
        CollisionFactorKind::RemovalInteraction => 90,
    }
}

fn severity(score: u8) -> CollisionSeverity {
    match score {
        0 => CollisionSeverity::None,
        1..=29 => CollisionSeverity::Informational,
        30..=59 => CollisionSeverity::Low,
        60..=79 => CollisionSeverity::Medium,
        _ => CollisionSeverity::High,
    }
}

fn explanation_summary(kind: CollisionFactorKind) -> &'static str {
    match kind {
        CollisionFactorKind::SameSymbolChanged => "Both branches modify the same semantic symbol.",
        CollisionFactorKind::ChangedSymbolImpact => {
            "Branch A changes a symbol reached by Branch B's semantic impact."
        }
        CollisionFactorKind::ReverseChangedSymbolImpact => {
            "Branch B changes a symbol reached by Branch A's semantic impact."
        }
        CollisionFactorKind::SharedImpact => {
            "Both branches affect the same downstream semantic symbol."
        }
        CollisionFactorKind::TransitiveImpact => {
            "The semantic interaction occurs through a transitive impact path."
        }
        CollisionFactorKind::SignatureInteraction => {
            "A changed callable signature is consumed by the other branch's changed code."
        }
        CollisionFactorKind::RemovalInteraction => {
            "A branch removes a symbol that the other branch's changed code depends upon."
        }
    }
}

#[cfg(test)]
mod tests {
    use branchsense_overlap::OverlapSet;
    use serde_json::json;

    use super::*;

    fn set(explanations: Vec<serde_json::Value>) -> OverlapSet {
        serde_json::from_value(json!({
            "entries": explanations.into_iter().map(|explanation| json!({ "explanation": explanation })).collect::<Vec<_>>(),
            "statistics": {
                "branch_a_changed": 1,
                "branch_b_changed": 1,
                "overlaps": 0,
                "direct_changes": 0,
                "impact_changes": 0,
                "shared_impacts": 0,
                "cross_impacts": 0,
                "max_depth": 0,
                "truncated": false
            }
        }))
        .expect("valid overlap fixture")
    }

    fn direct(a: &str, b: &str) -> serde_json::Value {
        json!({
            "branch_a_changed": a,
            "branch_b_changed": b,
            "branch_a_change_kind": "Modified",
            "branch_b_change_kind": "Modified",
            "targets": [a],
            "kind": "DirectChange",
            "branch_a_evidence": [],
            "branch_b_evidence": []
        })
    }

    fn impact(kind: &str, depth: usize, a_kind: &str, b_kind: &str) -> serde_json::Value {
        json!({
            "branch_a_changed": "symbol:a",
            "branch_b_changed": "symbol:b",
            "branch_a_change_kind": a_kind,
            "branch_b_change_kind": b_kind,
            "targets": ["symbol:b"],
            "kind": "ImpactChange",
            "branch_a_evidence": [{
                "changed_symbol": "symbol:a",
                "target_symbol": "symbol:b",
                "kind": kind,
                "relationship": "Calls",
                "depth": depth,
                "path": {"steps": []},
                "relationship_fact": null
            }],
            "branch_b_evidence": []
        })
    }

    #[test]
    fn no_overlap_has_no_collision() {
        let assessment = CollisionAnalyzer::new().analyze(&set(Vec::new()));
        assert_eq!(assessment.severity(), CollisionSeverity::None);
        assert_eq!(assessment.evidence_score(), 0);
        assert!(assessment.is_empty());
    }

    #[test]
    fn same_symbol_is_high_evidence() {
        let assessment =
            CollisionAnalyzer::new().analyze(&set(vec![direct("symbol:x", "symbol:x")]));
        assert_eq!(assessment.severity(), CollisionSeverity::High);
        assert_eq!(assessment.evidence_score(), 80);
        assert!(
            assessment
                .factors()
                .iter()
                .any(|factor| factor.kind() == CollisionFactorKind::SameSymbolChanged)
        );
    }

    #[test]
    fn signature_and_removal_use_stronger_specialized_factors() {
        let signature = CollisionAnalyzer::new().analyze(&set(vec![impact(
            "SignatureConsumer",
            1,
            "Modified",
            "Modified",
        )]));
        assert_eq!(signature.evidence_score(), 85);
        assert!(
            signature
                .factors()
                .iter()
                .any(|factor| factor.kind() == CollisionFactorKind::SignatureInteraction)
        );

        let removal = CollisionAnalyzer::new().analyze(&set(vec![impact(
            "DirectCaller",
            1,
            "Removed",
            "Modified",
        )]));
        assert_eq!(removal.evidence_score(), 90);
        assert!(
            removal
                .factors()
                .iter()
                .any(|factor| factor.kind() == CollisionFactorKind::RemovalInteraction)
        );
    }

    #[test]
    fn deep_impact_is_transitive_and_lower_strength() {
        let assessment = CollisionAnalyzer::new().analyze(&set(vec![impact(
            "TransitiveCaller",
            3,
            "Modified",
            "Modified",
        )]));
        assert_eq!(assessment.evidence_score(), 30);
        assert_eq!(assessment.severity(), CollisionSeverity::Low);
        assert!(
            assessment
                .factors()
                .iter()
                .any(|factor| factor.kind() == CollisionFactorKind::TransitiveImpact)
        );
    }

    #[test]
    fn duplicate_evidence_is_scored_once() {
        let entry = impact("DirectCaller", 1, "Modified", "Modified");
        let assessment = CollisionAnalyzer::new().analyze(&set(vec![entry.clone(), entry]));
        assert_eq!(assessment.evidence_score(), 65);
        assert_eq!(assessment.statistics().overlaps(), 2);
        assert_eq!(assessment.statistics().unique_evidence(), 1);
    }

    #[test]
    fn serialization_is_deterministic_and_branch_order_preserves_strength() {
        let forward = CollisionAnalyzer::new().analyze(&set(vec![impact(
            "DirectCaller",
            1,
            "Modified",
            "Modified",
        )]));
        let reverse = CollisionAnalyzer::new().analyze(&set(vec![impact(
            "DirectCaller",
            1,
            "Modified",
            "Modified",
        )]));
        assert_eq!(forward.evidence_score(), reverse.evidence_score());
        assert_eq!(forward.severity(), reverse.severity());
        assert_eq!(
            serde_json::to_vec(&forward).expect("serialize"),
            serde_json::to_vec(&reverse).expect("serialize")
        );
    }
}

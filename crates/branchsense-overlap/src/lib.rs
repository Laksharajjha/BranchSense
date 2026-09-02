//! Deterministic semantic overlap analysis for two independent changes.
//!
//! The analyzer compares the semantic changes made by branch A and branch B
//! relative to the same base. It consumes the existing [`SemanticDiff`] and
//! [`ImpactSet`] values; it does not parse source, inspect Git, traverse a
//! graph, or assign a risk score. This separation makes overlap analysis
//! reusable by the CLI, future editors, and collaboration services.
//!
//! An overlap result is evidence, not a prediction. Each entry identifies the
//! changed symbols, the shared or cross-branch target, and the causal paths
//! supplied by impact analysis. Callers may apply their own presentation or
//! policy without changing the deterministic result.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use branchsense_core::SymbolId;
use branchsense_diff::{ChangeKind, SemanticDiff};
use branchsense_impact::{ImpactCause, ImpactKind, ImpactPath, ImpactRelationship, ImpactSet};
use branchsense_semantic::{
    AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceIdentity, EvidenceKind,
    EvidenceLink, EvidenceRelation, EvidenceState, FactId, SemanticEntityIdentity, SemanticFact,
    SemanticFactRecord,
};
use serde::{Deserialize, Serialize};

/// The semantic relationship between two branch changes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum OverlapKind {
    /// Both branches changed the same stable symbol.
    DirectChange,
    /// One branch changed a symbol that the other branch changed directly.
    ImpactChange,
    /// Both branches impact the same downstream symbol.
    SharedImpact,
    /// Each branch changed a symbol that impacts the other branch's change.
    CrossImpact,
}

/// One causal explanation preserved from impact analysis.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OverlapEvidence {
    changed_symbol: SymbolId,
    target_symbol: SymbolId,
    kind: ImpactKind,
    relationship: ImpactRelationship,
    depth: usize,
    path: ImpactPath,
    relationship_fact: Option<FactId>,
}

impl OverlapEvidence {
    /// Returns the symbol changed by the branch supplying this evidence.
    #[must_use]
    pub fn changed_symbol(&self) -> &SymbolId {
        &self.changed_symbol
    }
    /// Returns the symbol reached by the causal path.
    #[must_use]
    pub fn target_symbol(&self) -> &SymbolId {
        &self.target_symbol
    }
    /// Returns the impact classification supplied by impact analysis.
    #[must_use]
    pub const fn kind(&self) -> ImpactKind {
        self.kind
    }
    /// Returns the nearest relationship to the changed symbol.
    #[must_use]
    pub const fn relationship(&self) -> ImpactRelationship {
        self.relationship
    }
    /// Returns the number of traversed relationships.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
    /// Returns the complete impacted-to-changed causal path.
    #[must_use]
    pub fn path(&self) -> &ImpactPath {
        &self.path
    }
    /// Returns the source fact, when the graph supplied one.
    #[must_use]
    pub fn relationship_fact(&self) -> Option<&FactId> {
        self.relationship_fact.as_ref()
    }
}

/// Structured explanation for one semantic overlap.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OverlapExplanation {
    branch_a_changed: SymbolId,
    branch_b_changed: SymbolId,
    branch_a_change_kind: Option<ChangeKind>,
    branch_b_change_kind: Option<ChangeKind>,
    targets: Vec<SymbolId>,
    kind: OverlapKind,
    branch_a_evidence: Vec<OverlapEvidence>,
    branch_b_evidence: Vec<OverlapEvidence>,
}

impl OverlapExplanation {
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
    /// Returns the declaration change kind for branch A, when available.
    #[must_use]
    pub const fn branch_a_change_kind(&self) -> Option<ChangeKind> {
        self.branch_a_change_kind
    }
    /// Returns the declaration change kind for branch B, when available.
    #[must_use]
    pub const fn branch_b_change_kind(&self) -> Option<ChangeKind> {
        self.branch_b_change_kind
    }
    /// Returns the shared or cross-branch targets in stable order.
    #[must_use]
    pub fn targets(&self) -> &[SymbolId] {
        &self.targets
    }
    /// Returns the overlap classification.
    #[must_use]
    pub const fn kind(&self) -> OverlapKind {
        self.kind
    }
    /// Returns causal evidence originating in branch A's impact set.
    #[must_use]
    pub fn branch_a_evidence(&self) -> &[OverlapEvidence] {
        &self.branch_a_evidence
    }
    /// Returns causal evidence originating in branch B's impact set.
    #[must_use]
    pub fn branch_b_evidence(&self) -> &[OverlapEvidence] {
        &self.branch_b_evidence
    }
}

/// One deduplicated semantic overlap with all meaningful causal paths.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OverlapEntry {
    explanation: OverlapExplanation,
}

impl OverlapEntry {
    /// Returns the structured overlap explanation.
    #[must_use]
    pub fn explanation(&self) -> &OverlapExplanation {
        &self.explanation
    }
}

/// Summary of a bounded overlap analysis.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OverlapStatistics {
    branch_a_changed: usize,
    branch_b_changed: usize,
    overlaps: usize,
    direct_changes: usize,
    impact_changes: usize,
    shared_impacts: usize,
    cross_impacts: usize,
    max_depth: usize,
    truncated: bool,
}

impl OverlapStatistics {
    /// Returns the number of changed symbols in branch A.
    #[must_use]
    pub const fn branch_a_changed(&self) -> usize {
        self.branch_a_changed
    }
    /// Returns the number of changed symbols in branch B.
    #[must_use]
    pub const fn branch_b_changed(&self) -> usize {
        self.branch_b_changed
    }
    /// Returns the number of overlap entries.
    #[must_use]
    pub const fn overlaps(&self) -> usize {
        self.overlaps
    }
    /// Returns the number of direct overlaps.
    #[must_use]
    pub const fn direct_changes(&self) -> usize {
        self.direct_changes
    }
    /// Returns the number of one-direction impact overlaps.
    #[must_use]
    pub const fn impact_changes(&self) -> usize {
        self.impact_changes
    }
    /// Returns the number of shared downstream impacts.
    #[must_use]
    pub const fn shared_impacts(&self) -> usize {
        self.shared_impacts
    }
    /// Returns the number of bidirectional cross impacts.
    #[must_use]
    pub const fn cross_impacts(&self) -> usize {
        self.cross_impacts
    }
    /// Returns the greatest retained evidence depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }
    /// Returns whether a result bound or upstream impact bound truncated evidence.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Immutable, deterministically ordered overlap results.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct OverlapSet {
    entries: Vec<OverlapEntry>,
    statistics: OverlapStatistics,
    #[serde(default)]
    evidence: EvidenceEnvelope,
}

impl OverlapSet {
    /// Returns overlap entries in stable classification and identity order.
    #[must_use]
    pub fn entries(&self) -> &[OverlapEntry] {
        &self.entries
    }
    /// Returns summary statistics.
    #[must_use]
    pub const fn statistics(&self) -> &OverlapStatistics {
        &self.statistics
    }

    /// Returns evidence state, provenance, and lineage for this overlap set.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceEnvelope {
        &self.evidence
    }
    /// Returns whether no overlap was found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Bounds for one overlap analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverlapOptions {
    max_depth: usize,
    max_results: usize,
}

impl Default for OverlapOptions {
    fn default() -> Self {
        Self { max_depth: 3, max_results: 1_000 }
    }
}

impl OverlapOptions {
    /// Creates explicit overlap bounds.
    #[must_use]
    pub const fn new(max_depth: usize, max_results: usize) -> Self {
        Self { max_depth, max_results }
    }
    /// Returns the maximum retained causal depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }
    /// Returns the maximum number of returned entries.
    #[must_use]
    pub const fn max_results(&self) -> usize {
        self.max_results
    }
}

/// Composes semantic diffs and bounded impact sets into overlap evidence.
#[derive(Clone, Copy, Debug, Default)]
pub struct SemanticOverlapAnalyzer {
    options: OverlapOptions,
}

impl SemanticOverlapAnalyzer {
    /// Creates an analyzer with conservative default bounds.
    #[must_use]
    pub const fn new() -> Self {
        Self { options: OverlapOptions::new(3, 1_000) }
    }
    /// Creates an analyzer with explicit bounds.
    #[must_use]
    pub const fn with_options(options: OverlapOptions) -> Self {
        Self { options }
    }
    /// Returns the configured bounds.
    #[must_use]
    pub const fn options(&self) -> OverlapOptions {
        self.options
    }

    /// Computes deterministic overlap evidence without mutating its inputs.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn analyze(
        &self,
        diff_a: &SemanticDiff,
        impact_a: &ImpactSet,
        diff_b: &SemanticDiff,
        impact_b: &ImpactSet,
    ) -> OverlapSet {
        let changed_a = changed_symbols(diff_a);
        let changed_b = changed_symbols(diff_b);
        let change_kinds_a = symbol_change_kinds(diff_a);
        let change_kinds_b = symbol_change_kinds(diff_b);
        let evidence_a = evidence_by_pair(impact_a, self.options.max_depth);
        let evidence_b = evidence_by_pair(impact_b, self.options.max_depth);
        let mut candidates = BTreeMap::<OverlapKey, Candidate>::new();

        for symbol in changed_a.intersection(&changed_b) {
            add_candidate(
                &mut candidates,
                OverlapKind::DirectChange,
                symbol.clone(),
                symbol.clone(),
                vec![symbol.clone()],
                Vec::new(),
                Vec::new(),
            );
        }

        for ((changed, target), evidence) in &evidence_a {
            if changed_a.contains(changed)
                && changed_b.contains(target)
                && changed != target
                && !evidence_b.contains_key(&(target.clone(), changed.clone()))
            {
                add_candidate(
                    &mut candidates,
                    OverlapKind::ImpactChange,
                    changed.clone(),
                    target.clone(),
                    vec![target.clone()],
                    evidence.clone(),
                    Vec::new(),
                );
            }
        }
        for ((changed, target), evidence) in &evidence_b {
            if changed_b.contains(changed)
                && changed_a.contains(target)
                && changed != target
                && !evidence_a.contains_key(&(target.clone(), changed.clone()))
            {
                add_candidate(
                    &mut candidates,
                    OverlapKind::ImpactChange,
                    target.clone(),
                    changed.clone(),
                    vec![target.clone()],
                    Vec::new(),
                    evidence.clone(),
                );
            }
        }

        for ((a_changed, a_target), a_evidence) in &evidence_a {
            if let Some(b_evidence) = evidence_b.get(&(a_target.clone(), a_changed.clone())) {
                if changed_a.contains(a_changed) && changed_b.contains(a_target) {
                    add_candidate(
                        &mut candidates,
                        OverlapKind::CrossImpact,
                        a_changed.clone(),
                        a_target.clone(),
                        vec![a_changed.clone(), a_target.clone()],
                        a_evidence.clone(),
                        b_evidence.clone(),
                    );
                }
            }
        }

        let targets_a = evidence_by_target(impact_a, self.options.max_depth);
        let targets_b = evidence_by_target(impact_b, self.options.max_depth);
        for target in targets_a.keys().filter(|target| targets_b.contains_key(*target)) {
            if changed_a.contains(target) || changed_b.contains(target) {
                continue;
            }
            for (a_changed, a_evidence) in &targets_a[target] {
                for (b_changed, b_evidence) in &targets_b[target] {
                    add_candidate(
                        &mut candidates,
                        OverlapKind::SharedImpact,
                        a_changed.clone(),
                        b_changed.clone(),
                        vec![target.clone()],
                        a_evidence.clone(),
                        b_evidence.clone(),
                    );
                }
            }
        }

        let mut statistics = OverlapStatistics {
            branch_a_changed: changed_a.len(),
            branch_b_changed: changed_b.len(),
            truncated: impact_a.statistics().truncated() || impact_b.statistics().truncated(),
            ..OverlapStatistics::default()
        };
        let mut truncated = statistics.truncated;
        let mut entries = Vec::new();
        for candidate in candidates.into_values() {
            if entries.len() >= self.options.max_results {
                truncated = true;
                break;
            }
            let explanation = candidate.into_explanation(&change_kinds_a, &change_kinds_b);
            statistics.max_depth = statistics.max_depth.max(
                explanation
                    .branch_a_evidence
                    .iter()
                    .chain(&explanation.branch_b_evidence)
                    .map(OverlapEvidence::depth)
                    .max()
                    .unwrap_or(0),
            );
            match explanation.kind {
                OverlapKind::DirectChange => statistics.direct_changes += 1,
                OverlapKind::ImpactChange => statistics.impact_changes += 1,
                OverlapKind::SharedImpact => statistics.shared_impacts += 1,
                OverlapKind::CrossImpact => statistics.cross_impacts += 1,
            }
            entries.push(OverlapEntry { explanation });
        }
        statistics.overlaps = entries.len();
        statistics.truncated = truncated;
        let state = if statistics.truncated {
            EvidenceState::Truncated
        } else if entries.is_empty() {
            EvidenceState::NoEvidence
        } else {
            EvidenceState::Observed
        }
        .combine(diff_a.evidence().state())
        .combine(diff_b.evidence().state());
        let mut provenance = AnalysisProvenance::new();
        if let Some(repository) = diff_a
            .evidence()
            .provenance()
            .repository_id()
            .or_else(|| diff_b.evidence().provenance().repository_id())
        {
            provenance = provenance.with_repository(repository.clone());
        }
        if let (Some(branch_a), Some(branch_b), Some(base)) = (
            diff_a.evidence().provenance().revision_id(),
            diff_b.evidence().provenance().revision_id(),
            diff_a.evidence().provenance().base_revision_id(),
        ) {
            provenance = provenance.with_base_revision(base.clone()).with_branches(
                branch_a.clone(),
                branch_b.clone(),
                base.clone(),
            );
        }
        let mut evidence = EvidenceEnvelope::new(
            state,
            EvidenceCompleteness::new().with_semantic(state),
            provenance,
        );
        for entry in &entries {
            let explanation = entry.explanation();
            for symbol in [explanation.branch_a_changed(), explanation.branch_b_changed()] {
                if let Some(definition) = changed_definition(diff_a, symbol)
                    .or_else(|| changed_definition(diff_b, symbol))
                {
                    if let Ok(identity) = SemanticEntityIdentity::from_definition(definition) {
                        let derived = EvidenceIdentity::semantic(EvidenceKind::Derived, &identity);
                        let primary = EvidenceIdentity::semantic(EvidenceKind::Primary, &identity);
                        evidence = evidence.with_identity(derived.clone()).with_link(
                            EvidenceLink::new(derived, primary, EvidenceRelation::DerivedFrom),
                        );
                    }
                }
            }
        }
        OverlapSet { entries, statistics, evidence }
    }
}

fn changed_definition<'a>(
    diff: &'a SemanticDiff,
    id: &SymbolId,
) -> Option<&'a branchsense_semantic::SymbolDefinition> {
    diff.symbols()
        .iter()
        .find(|change| change.after_id().or(change.before_id()) == Some(id))
        .and_then(|change| change.after().or(change.before()))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct OverlapKey {
    kind: OverlapKind,
    left_changed: SymbolId,
    right_changed: SymbolId,
    targets: Vec<SymbolId>,
}

struct Candidate {
    key: OverlapKey,
    a_evidence: Vec<OverlapEvidence>,
    b_evidence: Vec<OverlapEvidence>,
}

impl Candidate {
    fn into_explanation(
        mut self,
        change_kinds_a: &BTreeMap<SymbolId, ChangeKind>,
        change_kinds_b: &BTreeMap<SymbolId, ChangeKind>,
    ) -> OverlapExplanation {
        self.a_evidence.sort_by(evidence_order);
        self.a_evidence.dedup();
        self.b_evidence.sort_by(evidence_order);
        self.b_evidence.dedup();
        let left_kind = change_kinds_a.get(&self.key.left_changed).copied();
        let right_kind = change_kinds_b.get(&self.key.right_changed).copied();
        OverlapExplanation {
            branch_a_changed: self.key.left_changed,
            branch_b_changed: self.key.right_changed,
            branch_a_change_kind: left_kind,
            branch_b_change_kind: right_kind,
            targets: self.key.targets,
            kind: self.key.kind,
            branch_a_evidence: self.a_evidence,
            branch_b_evidence: self.b_evidence,
        }
    }
}

fn add_candidate(
    candidates: &mut BTreeMap<OverlapKey, Candidate>,
    kind: OverlapKind,
    left_changed: SymbolId,
    right_changed: SymbolId,
    targets: Vec<SymbolId>,
    left_evidence: Vec<OverlapEvidence>,
    right_evidence: Vec<OverlapEvidence>,
) {
    let key = OverlapKey { kind, left_changed, right_changed, targets };
    let candidate = candidates.entry(key.clone()).or_insert_with(|| Candidate {
        key,
        a_evidence: Vec::new(),
        b_evidence: Vec::new(),
    });
    candidate.a_evidence.extend(left_evidence);
    candidate.b_evidence.extend(right_evidence);
}

fn changed_symbols(diff: &SemanticDiff) -> BTreeSet<SymbolId> {
    let mut symbols = diff
        .symbols()
        .iter()
        .filter(|change| change.kind() != ChangeKind::Unchanged)
        .filter_map(|change| change.after_id().or(change.before_id()).cloned())
        .collect::<BTreeSet<_>>();
    for relationship in diff.relationships() {
        let fact_change = relationship.fact();
        if fact_change.kind() == ChangeKind::Unchanged {
            continue;
        }
        if let Some(source) = fact_change
            .after()
            .or(fact_change.before())
            .map(SemanticFactRecord::fact)
            .and_then(source_symbol)
        {
            symbols.insert(source);
        }
    }
    symbols
}

fn symbol_change_kinds(diff: &SemanticDiff) -> BTreeMap<SymbolId, ChangeKind> {
    diff.symbols()
        .iter()
        .filter(|change| change.kind() != ChangeKind::Unchanged)
        .filter_map(|change| {
            change.after_id().or(change.before_id()).cloned().map(|id| (id, change.kind()))
        })
        .collect()
}

fn source_symbol(fact: &SemanticFact) -> Option<SymbolId> {
    match fact {
        SemanticFact::Contains(fact) => Some(fact.container().clone()),
        SemanticFact::Call(fact) => Some(fact.caller().clone()),
        SemanticFact::Reference(fact) => Some(fact.source().clone()),
        SemanticFact::TypeRelation(fact) => Some(fact.source().clone()),
        SemanticFact::Dependency(fact) => Some(fact.source().clone()),
        SemanticFact::Definition(_)
        | SemanticFact::Parameter(_)
        | SemanticFact::ReturnType(_)
        | SemanticFact::Import(_)
        | SemanticFact::Documentation(_)
        | SemanticFact::Annotation(_) => None,
    }
}

fn evidence_by_pair(
    impact: &ImpactSet,
    max_depth: usize,
) -> BTreeMap<(SymbolId, SymbolId), Vec<OverlapEvidence>> {
    let mut result = BTreeMap::new();
    for entry in impact.entries() {
        for cause in entry.causes() {
            if cause.explanation().depth() <= max_depth {
                result
                    .entry((
                        cause.explanation().changed_symbol().clone(),
                        entry.impacted_symbol().clone(),
                    ))
                    .or_insert_with(Vec::new)
                    .push(evidence(entry.impacted_symbol(), cause));
            }
        }
    }
    for values in result.values_mut() {
        values.sort_by(evidence_order);
        values.dedup();
    }
    result
}

fn evidence_by_target(
    impact: &ImpactSet,
    max_depth: usize,
) -> BTreeMap<SymbolId, BTreeMap<SymbolId, Vec<OverlapEvidence>>> {
    let mut result = BTreeMap::new();
    for entry in impact.entries() {
        for cause in entry.causes() {
            if cause.explanation().depth() <= max_depth {
                result
                    .entry(entry.impacted_symbol().clone())
                    .or_insert_with(BTreeMap::new)
                    .entry(cause.explanation().changed_symbol().clone())
                    .or_insert_with(Vec::new)
                    .push(evidence(entry.impacted_symbol(), cause));
            }
        }
    }
    for values in result.values_mut().flat_map(BTreeMap::values_mut) {
        values.sort_by(evidence_order);
        values.dedup();
    }
    result
}

fn evidence(target: &SymbolId, cause: &ImpactCause) -> OverlapEvidence {
    let explanation = cause.explanation();
    OverlapEvidence {
        changed_symbol: explanation.changed_symbol().clone(),
        target_symbol: target.clone(),
        kind: explanation.kind(),
        relationship: explanation.relationship(),
        depth: explanation.depth(),
        path: explanation.path().clone(),
        relationship_fact: cause.relationship_fact().cloned(),
    }
}

fn evidence_order(left: &OverlapEvidence, right: &OverlapEvidence) -> std::cmp::Ordering {
    (
        &left.changed_symbol,
        &left.target_symbol,
        left.depth,
        left.kind,
        left.path.steps(),
        &left.relationship_fact,
    )
        .cmp(&(
            &right.changed_symbol,
            &right.target_symbol,
            right.depth,
            right.kind,
            right.path.steps(),
            &right.relationship_fact,
        ))
}

//! Bounded, deterministic semantic impact analysis.
//!
//! Impact analysis answers which declarations may depend on changed semantic
//! declarations. It consumes [`branchsense_diff::SemanticDiff`] and immutable
//! graph snapshots; it does not parse source, invoke Git, or mutate either
//! input. Call edges are traversed backwards from callee to caller, while
//! references, inheritance, implementation, and dependency edges are reported
//! as direct impacts.
//!
//! The result is intentionally separate from a diff. A diff describes what
//! changed. An [`ImpactSet`] describes the other declarations that may need
//! review because of those changes. This distinction is the foundation for
//! future branch-overlap analysis.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use branchsense_core::SymbolId;
use branchsense_diff::{ChangeKind, SemanticDiff, SymbolChangeReason};
use branchsense_graph::{EdgeKind, GraphNode, GraphNodeId, SemanticGraph};
use branchsense_index::SemanticIndexSnapshot;
use branchsense_semantic::{
    EvidenceCompleteness, EvidenceEnvelope, EvidenceIdentity, EvidenceKind, EvidenceLink,
    EvidenceRelation, EvidenceState, FactId, SemanticEntityIdentity, SemanticFact,
};
use serde::{Deserialize, Serialize};

/// Relationship category that caused one impact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ImpactRelationship {
    /// A call edge from an impacted caller to a changed callee.
    Calls,
    /// A reference edge to a changed symbol.
    References,
    /// An implementation edge to a changed interface or type.
    Implements,
    /// An extension edge to a changed superclass.
    Extends,
    /// An explicit dependency edge to a changed symbol.
    DependsOn,
}

/// Classification of one semantic impact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ImpactKind {
    /// A caller directly invokes a changed callable.
    DirectCaller,
    /// A caller is reached through one or more intermediate call edges.
    TransitiveCaller,
    /// A symbol directly references a changed symbol.
    Reference,
    /// A type implements a changed interface.
    Implementation,
    /// A type extends a changed superclass.
    Subtype,
    /// A symbol has a changed dependency.
    Dependency,
    /// A call site may consume a changed callable signature.
    SignatureConsumer,
}

/// One traversed semantic edge in an impact explanation.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ImpactStep {
    from: SymbolId,
    to: SymbolId,
    relationship: ImpactRelationship,
}

impl ImpactStep {
    fn new(from: SymbolId, to: SymbolId, relationship: ImpactRelationship) -> Self {
        Self { from, to, relationship }
    }

    /// Returns the source symbol of this step.
    #[must_use]
    pub fn from(&self) -> &SymbolId {
        &self.from
    }

    /// Returns the target symbol of this step.
    #[must_use]
    pub fn to(&self) -> &SymbolId {
        &self.to
    }

    /// Returns the semantic relationship represented by this step.
    #[must_use]
    pub const fn relationship(&self) -> ImpactRelationship {
        self.relationship
    }
}

/// An immutable path from an impacted symbol to a changed symbol.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImpactPath {
    steps: Vec<ImpactStep>,
}

impl ImpactPath {
    fn new(steps: Vec<ImpactStep>) -> Self {
        Self { steps }
    }

    /// Returns path steps in impacted-to-changed order.
    #[must_use]
    pub fn steps(&self) -> &[ImpactStep] {
        &self.steps
    }
}

/// Structured explanation for one cause of an impact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImpactExplanation {
    changed_symbol: SymbolId,
    relationship: ImpactRelationship,
    kind: ImpactKind,
    depth: usize,
    path: ImpactPath,
}

impl ImpactExplanation {
    /// Returns the changed declaration that initiated this explanation.
    #[must_use]
    pub fn changed_symbol(&self) -> &SymbolId {
        &self.changed_symbol
    }

    /// Returns the relationship nearest to the changed declaration.
    #[must_use]
    pub const fn relationship(&self) -> ImpactRelationship {
        self.relationship
    }

    /// Returns the impact classification.
    #[must_use]
    pub const fn kind(&self) -> ImpactKind {
        self.kind
    }

    /// Returns the number of traversed edges.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the structured causal path.
    #[must_use]
    pub fn path(&self) -> &ImpactPath {
        &self.path
    }
}

/// One causal reason why a symbol is impacted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImpactCause {
    explanation: ImpactExplanation,
    relationship_fact: Option<FactId>,
}

impl ImpactCause {
    /// Returns the structured explanation.
    #[must_use]
    pub fn explanation(&self) -> &ImpactExplanation {
        &self.explanation
    }

    /// Returns the graph fact that supplied the causal edge, when available.
    #[must_use]
    pub fn relationship_fact(&self) -> Option<&FactId> {
        self.relationship_fact.as_ref()
    }
}

/// A symbol impacted by one or more changed symbols.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImpactEntry {
    impacted_symbol: SymbolId,
    causes: Vec<ImpactCause>,
}

impl ImpactEntry {
    /// Returns the impacted declaration.
    #[must_use]
    pub fn impacted_symbol(&self) -> &SymbolId {
        &self.impacted_symbol
    }

    /// Returns all distinct causal explanations in deterministic order.
    #[must_use]
    pub fn causes(&self) -> &[ImpactCause] {
        &self.causes
    }

    /// Returns the distinct changed declarations causing this entry.
    #[must_use]
    pub fn changed_symbols(&self) -> Vec<&SymbolId> {
        let mut symbols =
            self.causes.iter().map(|cause| cause.explanation.changed_symbol()).collect::<Vec<_>>();
        symbols.sort();
        symbols.dedup();
        symbols
    }
}

/// Summary of bounded impact analysis.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImpactStatistics {
    changed_symbols: usize,
    impacted_symbols: usize,
    direct_impacts: usize,
    transitive_impacts: usize,
    max_depth: usize,
    truncated: bool,
}

impl ImpactStatistics {
    /// Returns the number of changed symbols used as analysis roots.
    #[must_use]
    pub const fn changed_symbols(&self) -> usize {
        self.changed_symbols
    }
    /// Returns the number of distinct impacted declarations.
    #[must_use]
    pub const fn impacted_symbols(&self) -> usize {
        self.impacted_symbols
    }
    /// Returns direct impact count, including direct non-call relationships.
    #[must_use]
    pub const fn direct_impacts(&self) -> usize {
        self.direct_impacts
    }
    /// Returns transitive caller count.
    #[must_use]
    pub const fn transitive_impacts(&self) -> usize {
        self.transitive_impacts
    }
    /// Returns the greatest observed traversal depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }
    /// Returns whether a configured bound discarded possible results.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Immutable, deterministically ordered impact results.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImpactSet {
    entries: Vec<ImpactEntry>,
    statistics: ImpactStatistics,
    #[serde(default)]
    evidence: EvidenceEnvelope,
}

impl ImpactSet {
    /// Returns impacted symbols in stable identity order.
    #[must_use]
    pub fn entries(&self) -> &[ImpactEntry] {
        &self.entries
    }
    /// Returns summary statistics.
    #[must_use]
    pub const fn statistics(&self) -> &ImpactStatistics {
        &self.statistics
    }

    /// Returns evidence state, provenance, and lineage for this impact set.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceEnvelope {
        &self.evidence
    }
    /// Returns whether no impacted declarations were found.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Explicit limits for one impact analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImpactOptions {
    max_depth: usize,
    max_results: usize,
}

impl Default for ImpactOptions {
    fn default() -> Self {
        Self { max_depth: 3, max_results: 1_000 }
    }
}

impl ImpactOptions {
    /// Creates bounded options.
    #[must_use]
    pub const fn new(max_depth: usize, max_results: usize) -> Self {
        Self { max_depth, max_results }
    }
    /// Returns the maximum call depth.
    #[must_use]
    pub const fn max_depth(&self) -> usize {
        self.max_depth
    }
    /// Returns the maximum distinct impacted symbols.
    #[must_use]
    pub const fn max_results(&self) -> usize {
        self.max_results
    }
}

/// Deterministic analyzer over immutable semantic snapshots.
#[derive(Clone, Copy, Debug, Default)]
pub struct ImpactAnalyzer {
    options: ImpactOptions,
}

impl ImpactAnalyzer {
    /// Creates an analyzer with conservative default bounds.
    #[must_use]
    pub const fn new() -> Self {
        Self { options: ImpactOptions::new(3, 1_000) }
    }

    /// Creates an analyzer with explicit traversal bounds.
    #[must_use]
    pub const fn with_options(options: ImpactOptions) -> Self {
        Self { options }
    }

    /// Returns configured analysis bounds.
    #[must_use]
    pub const fn options(&self) -> ImpactOptions {
        self.options
    }

    /// Analyzes changed declarations without mutating either snapshot or diff.
    #[must_use]
    pub fn analyze(
        &self,
        diff: &SemanticDiff,
        before: &SemanticIndexSnapshot,
        after: &SemanticIndexSnapshot,
    ) -> ImpactSet {
        let mut roots = BTreeMap::<SymbolId, Root>::new();
        let mut signature_roots = BTreeSet::new();
        for change in diff.symbols().iter().filter(|change| change.kind() != ChangeKind::Unchanged)
        {
            let (id, graph) = match change.kind() {
                ChangeKind::Removed => (change.before_id(), &before.graph()),
                ChangeKind::Added | ChangeKind::Modified => (change.after_id(), &after.graph()),
                ChangeKind::Unchanged => unreachable!(),
            };
            if let Some(id) = id {
                roots.entry(id.clone()).or_insert(Root { id: id.clone(), graph });
                if change.reasons().iter().any(|reason| {
                    matches!(
                        reason,
                        SymbolChangeReason::MethodSignatureChanged
                            | SymbolChangeReason::ReturnTypeChanged
                            | SymbolChangeReason::ParameterAdded
                            | SymbolChangeReason::ParameterRemoved
                            | SymbolChangeReason::ParameterTypeChanged
                    )
                }) {
                    signature_roots.insert(id.clone());
                }
            }
        }
        for relationship in diff.relationships() {
            let fact = relationship
                .fact()
                .after()
                .or(relationship.fact().before())
                .map(branchsense_semantic::SemanticFactRecord::fact);
            if let Some(source) = fact.and_then(source_symbol) {
                let graph = if relationship.fact().kind() == ChangeKind::Removed {
                    &before.graph()
                } else {
                    &after.graph()
                };
                roots.entry(source.clone()).or_insert(Root { id: source, graph });
            }
        }

        let mut entries = BTreeMap::<SymbolId, Vec<ImpactCause>>::new();
        let mut truncated = false;
        for root in roots.values() {
            self.traverse(root, &signature_roots, &mut entries, &mut truncated);
        }
        let entries = entries
            .into_iter()
            .map(|(impacted_symbol, mut causes)| {
                causes.sort_by_key(cause_key);
                causes.dedup();
                ImpactEntry { impacted_symbol, causes }
            })
            .collect::<Vec<_>>();
        let mut statistics = ImpactStatistics {
            changed_symbols: roots.len(),
            impacted_symbols: entries.len(),
            truncated,
            ..ImpactStatistics::default()
        };
        for entry in &entries {
            for cause in &entry.causes {
                statistics.max_depth = statistics.max_depth.max(cause.explanation.depth);
                if cause.explanation.depth == 1 {
                    statistics.direct_impacts += 1;
                }
                if cause.explanation.kind == ImpactKind::TransitiveCaller {
                    statistics.transitive_impacts += 1;
                }
            }
        }
        let evidence = impact_evidence(diff, before, after, &entries, statistics.truncated);
        ImpactSet { entries, statistics, evidence }
    }

    fn traverse(
        &self,
        root: &Root<'_>,
        signature_roots: &BTreeSet<SymbolId>,
        entries: &mut BTreeMap<SymbolId, Vec<ImpactCause>>,
        truncated: &mut bool,
    ) {
        let mut frontier = VecDeque::from([(root.id.clone(), Vec::<ImpactStep>::new(), 0usize)]);
        let mut visited = BTreeSet::from([root.id.clone()]);
        while let Some((current, path, depth)) = frontier.pop_front() {
            if depth >= self.options.max_depth {
                continue;
            }
            let node = GraphNodeId::Symbol(current.clone());
            let mut incoming = root.graph.incoming_relationships(&node);
            incoming.extend(
                root.graph
                    .edges()
                    .filter(|edge| unresolved_target_matches(root.graph, edge.target(), &current)),
            );
            incoming.sort_by(|left, right| left.id().cmp(right.id()));
            incoming.dedup_by(|left, right| left.id() == right.id());
            for edge in incoming {
                let Some(source) = symbol_node(root.graph, edge.source()) else { continue };
                let target = if let Some(target) = symbol_node(root.graph, edge.target()) {
                    target
                } else if unresolved_target_matches(root.graph, edge.target(), &current) {
                    current.clone()
                } else {
                    continue;
                };
                let Some(relationship) = relationship(edge.kind()) else { continue };
                let next_depth = depth + 1;
                let kind =
                    impact_kind(relationship, next_depth, signature_roots.contains(&root.id));
                let mut next_path = path.clone();
                next_path.push(ImpactStep::new(source.clone(), target, relationship));
                let explanation = ImpactExplanation {
                    changed_symbol: root.id.clone(),
                    relationship,
                    kind,
                    depth: next_depth,
                    path: ImpactPath::new(next_path.clone()),
                };
                let cause =
                    ImpactCause { explanation, relationship_fact: Some(edge.fact_id().clone()) };
                if source != root.id {
                    if !entries.contains_key(&source) && entries.len() >= self.options.max_results {
                        *truncated = true;
                    } else {
                        entries.entry(source.clone()).or_default().push(cause);
                    }
                }
                if edge.kind() == EdgeKind::Calls && visited.insert(source.clone()) {
                    frontier.push_back((source, next_path, next_depth));
                }
            }
        }
    }
}

fn impact_evidence(
    diff: &SemanticDiff,
    before: &SemanticIndexSnapshot,
    after: &SemanticIndexSnapshot,
    entries: &[ImpactEntry],
    truncated: bool,
) -> EvidenceEnvelope {
    let state = if truncated {
        EvidenceState::Truncated
    } else if entries.is_empty() {
        EvidenceState::NoEvidence
    } else {
        EvidenceState::Observed
    };
    let semantic_state = if diff.evidence().state() == EvidenceState::NoEvidence {
        EvidenceState::NoEvidence
    } else {
        diff.evidence().completeness().semantic()
    };
    let mut evidence = EvidenceEnvelope::derived_from(
        diff.evidence(),
        state,
        EvidenceCompleteness::new().with_semantic(semantic_state),
    );
    for entry in entries {
        let definition = after
            .graph()
            .find_symbol(entry.impacted_symbol())
            .or_else(|| before.graph().find_symbol(entry.impacted_symbol()))
            .and_then(GraphNode::definition);
        if let Some(definition) = definition {
            if let Ok(identity) = SemanticEntityIdentity::from_definition(definition) {
                let derived = EvidenceIdentity::semantic(EvidenceKind::Derived, &identity);
                let primary = EvidenceIdentity::semantic(EvidenceKind::Primary, &identity);
                evidence = evidence.with_identity(derived.clone()).with_link(EvidenceLink::new(
                    derived,
                    primary,
                    EvidenceRelation::DerivedFrom,
                ));
            }
        }
    }
    evidence
}

struct Root<'a> {
    id: SymbolId,
    graph: &'a SemanticGraph,
}

fn symbol_node(graph: &SemanticGraph, node: &GraphNodeId) -> Option<SymbolId> {
    match graph.node(node) {
        Some(GraphNode::Symbol { definition, .. }) => Some(definition.id().clone()),
        _ => None,
    }
}

fn unresolved_target_matches(graph: &SemanticGraph, node: &GraphNodeId, symbol: &SymbolId) -> bool {
    let Some(GraphNode::Unresolved { name }) = graph.node(node) else { return false };
    let Some(GraphNode::Symbol { definition, .. }) = graph.find_symbol(symbol) else {
        return false;
    };
    let Some(qualified_name) = definition.qualified_name() else { return false };
    let declaration_name = qualified_name
        .as_str()
        .split_once('(')
        .map_or_else(|| qualified_name.as_str(), |(prefix, _)| prefix);
    if name.as_str() != declaration_name {
        return false;
    }
    let matches = graph
        .nodes()
        .filter_map(GraphNode::definition)
        .filter_map(|definition| definition.qualified_name())
        .filter(|qualified| {
            qualified.as_str().split_once('(').map_or(qualified.as_str(), |(prefix, _)| prefix)
                == name.as_str()
        })
        .count();
    matches == 1
}

fn relationship(kind: EdgeKind) -> Option<ImpactRelationship> {
    match kind {
        EdgeKind::Calls => Some(ImpactRelationship::Calls),
        EdgeKind::References => Some(ImpactRelationship::References),
        EdgeKind::Implements => Some(ImpactRelationship::Implements),
        EdgeKind::Extends => Some(ImpactRelationship::Extends),
        EdgeKind::DependsOn => Some(ImpactRelationship::DependsOn),
        EdgeKind::Defines
        | EdgeKind::Contains
        | EdgeKind::Imports
        | EdgeKind::Returns
        | EdgeKind::Parameter
        | EdgeKind::Documents
        | EdgeKind::Annotates => None,
    }
}

fn impact_kind(relationship: ImpactRelationship, depth: usize, signature: bool) -> ImpactKind {
    match relationship {
        ImpactRelationship::Calls if signature && depth == 1 => ImpactKind::SignatureConsumer,
        ImpactRelationship::Calls if depth == 1 => ImpactKind::DirectCaller,
        ImpactRelationship::Calls => ImpactKind::TransitiveCaller,
        ImpactRelationship::References => ImpactKind::Reference,
        ImpactRelationship::Implements => ImpactKind::Implementation,
        ImpactRelationship::Extends => ImpactKind::Subtype,
        ImpactRelationship::DependsOn => ImpactKind::Dependency,
    }
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

fn cause_key(cause: &ImpactCause) -> (SymbolId, ImpactKind, usize, Vec<ImpactStep>) {
    (
        cause.explanation.changed_symbol.clone(),
        cause.explanation.kind,
        cause.explanation.depth,
        cause.explanation.path.steps.clone(),
    )
}

#[cfg(test)]
mod tests;

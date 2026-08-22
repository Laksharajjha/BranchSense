//! Backend-independent immutable graph snapshot implementation.

use std::collections::{BTreeMap, BTreeSet};

use branchsense_core::{DocumentId, RevisionId, SymbolId};
use branchsense_semantic::{
    AnnotationFact, DependencyKind, FactDelta, FactProvenance, ImportFact, ResolutionState,
    SemanticFact, SemanticFactRecord, SemanticFactSet, SymbolDefinition, SymbolReference,
    TypeReference, TypeRelation, TypeRelationFact,
};

use crate::{
    error::{GraphError, Result},
    model::{EdgeKind, GraphEdge, GraphEdgeId, GraphNode, GraphNodeId, GraphStatistics},
};

/// An immutable semantic graph snapshot.
#[derive(Clone, Debug, Default)]
pub struct SemanticGraph {
    revision_id: Option<RevisionId>,
    document_facts: BTreeMap<DocumentId, SemanticFactSet>,
    nodes: BTreeMap<GraphNodeId, GraphNode>,
    edges: BTreeMap<GraphEdgeId, GraphEdge>,
    symbol_index: BTreeMap<SymbolId, GraphNodeId>,
    qualified_index: BTreeMap<branchsense_core::QualifiedName, SymbolId>,
    document_nodes: BTreeMap<DocumentId, BTreeSet<GraphNodeId>>,
    outgoing: BTreeMap<GraphNodeId, BTreeSet<GraphEdgeId>>,
    incoming: BTreeMap<GraphNodeId, BTreeSet<GraphEdgeId>>,
}

impl SemanticGraph {
    /// Creates an empty graph snapshot.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates a graph containing one document's facts.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when fact identities, ownership, or endpoints
    /// are inconsistent.
    pub fn from_document_facts(
        document_id: DocumentId,
        revision_id: RevisionId,
        facts: SemanticFactSet,
    ) -> Result<Self> {
        Self::from_documents(
            revision_id,
            vec![branchsense_semantic::DocumentFactSet::new(document_id, facts)],
        )
    }

    /// Creates a graph from document-owned fact sets.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when fact identities, ownership, or endpoints
    /// are inconsistent.
    pub fn from_documents(
        revision_id: RevisionId,
        documents: Vec<branchsense_semantic::DocumentFactSet>,
    ) -> Result<Self> {
        let mut document_facts = BTreeMap::new();
        for document in documents {
            if document_facts
                .insert(document.document_id().clone(), document.facts().clone())
                .is_some()
            {
                return Err(GraphError::DuplicateNode {
                    node: GraphNodeId::Document(document.document_id().clone()),
                });
            }
        }
        Self::rebuild(Some(revision_id), document_facts)
    }

    /// Applies a document-scoped fact delta and returns a new snapshot.
    ///
    /// The current implementation rebuilds derived indexes from the retained
    /// document fact state. This is intentionally correct and deterministic;
    /// the delta API prevents downstream consumers from depending on that
    /// internal strategy and enables future copy-on-write optimization.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when the delta creates an invalid graph state.
    pub fn apply_delta(&self, delta: &FactDelta) -> Result<Self> {
        let document_id = delta.document_id().ok_or_else(|| GraphError::MissingEndpoint {
            edge_id: "delta-without-document".into(),
        })?;
        let mut document_facts = self.document_facts.clone();
        let mut records = document_facts.get(document_id).map_or_else(BTreeMap::new, |facts| {
            facts.facts().iter().map(|record| (record.id().clone(), record.clone())).collect()
        });

        for id in delta.removed() {
            records.remove(id);
        }
        for update in delta.updated() {
            records.insert(update.after().id().clone(), update.after().clone());
        }
        for record in delta.added() {
            records.insert(record.id().clone(), record.clone());
        }
        let mut facts = SemanticFactSet::new(records.into_values().collect());
        if let Some(previous) =
            document_facts.get(document_id).and_then(SemanticFactSet::provenance)
        {
            facts = facts.with_provenance(previous.clone());
        }
        document_facts.insert(document_id.clone(), facts);
        Self::rebuild(
            delta.revision_id().cloned().or_else(|| self.revision_id.clone()),
            document_facts,
        )
    }

    /// Atomically replaces all facts owned by one document.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when the replacement creates an invalid graph
    /// state.
    pub fn replace_document_facts(
        &self,
        document_id: DocumentId,
        revision_id: RevisionId,
        facts: SemanticFactSet,
    ) -> Result<Self> {
        let mut document_facts = self.document_facts.clone();
        document_facts.insert(document_id, facts);
        Self::rebuild(Some(revision_id), document_facts)
    }

    /// Atomically removes a document and all facts it owns.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when rebuilding the resulting snapshot fails.
    pub fn remove_document(
        &self,
        document_id: &DocumentId,
        revision_id: RevisionId,
    ) -> Result<Self> {
        let mut document_facts = self.document_facts.clone();
        document_facts.remove(document_id);
        Self::rebuild(Some(revision_id), document_facts)
    }

    /// Returns the graph revision, when one has been assigned.
    #[must_use]
    pub fn revision_id(&self) -> Option<&RevisionId> {
        self.revision_id.as_ref()
    }

    /// Returns a node by identity.
    #[must_use]
    pub fn node(&self, id: &GraphNodeId) -> Option<&GraphNode> {
        self.nodes.get(id)
    }

    /// Returns an edge by identity.
    #[must_use]
    pub fn edge(&self, id: &GraphEdgeId) -> Option<&GraphEdge> {
        self.edges.get(id)
    }

    /// Finds a declared symbol through the symbol index.
    #[must_use]
    pub fn find_symbol(&self, id: &SymbolId) -> Option<&GraphNode> {
        self.symbol_index.get(id).and_then(|node| self.nodes.get(node))
    }

    /// Returns outgoing edges for a node in deterministic order.
    #[must_use]
    pub fn outgoing_relationships(&self, id: &GraphNodeId) -> Vec<&GraphEdge> {
        self.outgoing.get(id).map_or_else(Vec::new, |edges| {
            edges.iter().filter_map(|edge| self.edges.get(edge)).collect()
        })
    }

    /// Returns incoming edges for a node in deterministic order.
    #[must_use]
    pub fn incoming_relationships(&self, id: &GraphNodeId) -> Vec<&GraphEdge> {
        self.incoming.get(id).map_or_else(Vec::new, |edges| {
            edges.iter().filter_map(|edge| self.edges.get(edge)).collect()
        })
    }

    /// Returns outgoing successor nodes for a node.
    #[must_use]
    pub fn successors(&self, id: &GraphNodeId) -> Vec<&GraphNode> {
        self.outgoing_relationships(id)
            .into_iter()
            .filter_map(|edge| self.nodes.get(edge.target()))
            .collect()
    }

    /// Returns incoming predecessor nodes for a node.
    #[must_use]
    pub fn predecessors(&self, id: &GraphNodeId) -> Vec<&GraphNode> {
        self.incoming_relationships(id)
            .into_iter()
            .filter_map(|edge| self.nodes.get(edge.source()))
            .collect()
    }

    /// Returns the fact set owned by a document.
    #[must_use]
    pub fn document_facts(&self, document_id: &DocumentId) -> Option<&SemanticFactSet> {
        self.document_facts.get(document_id)
    }

    /// Returns all nodes in deterministic identity order.
    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    /// Returns all edges in deterministic identity order.
    pub fn edges(&self) -> impl Iterator<Item = &GraphEdge> {
        self.edges.values()
    }

    /// Returns graph cardinality statistics.
    #[must_use]
    pub fn statistics(&self) -> GraphStatistics {
        let mut documents = 0;
        let mut symbols = 0;
        let mut external = 0;
        let mut unresolved = 0;
        for node in self.nodes.values() {
            match node {
                GraphNode::Document { .. } => documents += 1,
                GraphNode::Symbol { .. } => symbols += 1,
                GraphNode::External { .. } => external += 1,
                GraphNode::Unresolved { .. } => unresolved += 1,
            }
        }
        GraphStatistics::new(
            self.nodes.len(),
            self.edges.len(),
            documents,
            symbols,
            external,
            unresolved,
        )
    }

    fn rebuild(
        revision_id: Option<RevisionId>,
        document_facts: BTreeMap<DocumentId, SemanticFactSet>,
    ) -> Result<Self> {
        let mut normalized_documents = BTreeMap::new();
        for (document_id, facts) in document_facts {
            let mut records = BTreeMap::new();
            for record in facts.facts() {
                if let Some(existing) = records.get(record.id()) {
                    if existing != record {
                        return Err(GraphError::DuplicateFact {
                            fact_id: record.id().as_str().into(),
                        });
                    }
                } else {
                    records.insert(record.id().clone(), record.clone());
                }
            }
            let mut normalized = SemanticFactSet::new(records.into_values().collect());
            if let Some(provenance) = facts.provenance() {
                normalized = normalized.with_provenance(provenance.clone());
            }
            normalized_documents.insert(document_id, normalized);
        }
        let mut graph =
            Self { revision_id, document_facts: normalized_documents, ..Self::default() };
        let batches: Vec<(DocumentId, SemanticFactSet)> = graph
            .document_facts
            .iter()
            .map(|(document, facts)| (document.clone(), facts.clone()))
            .collect();
        for (document_id, facts) in &batches {
            graph.insert_node(GraphNode::Document { id: document_id.clone() })?;
            for record in facts.facts() {
                Self::validate_fact_document(document_id, record)?;
                if let SemanticFact::Definition(definition) = record.fact() {
                    graph.insert_symbol(definition, document_id)?;
                } else if let SemanticFact::Parameter(parameter) = record.fact() {
                    graph.insert_symbol(parameter.parameter(), document_id)?;
                }
            }
        }
        for (document_id, facts) in &batches {
            for record in facts.facts() {
                graph.insert_fact(document_id, facts.provenance(), record)?;
            }
        }
        Ok(graph)
    }

    fn validate_fact_document(document_id: &DocumentId, record: &SemanticFactRecord) -> Result<()> {
        let location_document =
            fact_location(record).map(|location| location.document_id().clone());
        if let Some(actual) = location_document {
            if &actual != document_id {
                return Err(GraphError::DocumentMismatch { actual, expected: document_id.clone() });
            }
        }
        Ok(())
    }

    fn insert_symbol(
        &mut self,
        definition: &SymbolDefinition,
        document_id: &DocumentId,
    ) -> Result<()> {
        let node =
            GraphNode::Symbol { definition: definition.clone(), document_id: document_id.clone() };
        let node_id = node.id();
        if let Some(existing) = self.nodes.get(&node_id) {
            if existing != &node {
                return Err(GraphError::DuplicateNode { node: node_id });
            }
        } else {
            self.insert_node(node)?;
        }
        if let GraphNodeId::Symbol(symbol) = node_id {
            self.symbol_index.insert(symbol, GraphNodeId::Symbol(definition.id().clone()));
            if let Some(qualified_name) = definition.qualified_name() {
                self.qualified_index.insert(qualified_name.clone(), definition.id().clone());
            }
        }
        Ok(())
    }

    fn insert_node(&mut self, node: GraphNode) -> Result<()> {
        let id = node.id();
        if self.nodes.contains_key(&id) {
            return Err(GraphError::DuplicateNode { node: id });
        }
        if let Some(document_id) = node.document_id() {
            self.document_nodes.entry(document_id.clone()).or_default().insert(id.clone());
        }
        self.nodes.insert(id, node);
        Ok(())
    }

    fn insert_fact(
        &mut self,
        document_id: &DocumentId,
        provenance: Option<&FactProvenance>,
        record: &SemanticFactRecord,
    ) -> Result<()> {
        match record.fact() {
            SemanticFact::Definition(definition) => self.add_edge(
                record,
                GraphNodeId::Document(document_id.clone()),
                GraphNodeId::Symbol(definition.id().clone()),
                EdgeKind::Defines,
                None,
                provenance,
            ),
            SemanticFact::Parameter(parameter) => self.add_edge(
                record,
                GraphNodeId::Symbol(parameter.callable().clone()),
                GraphNodeId::Symbol(parameter.parameter().id().clone()),
                EdgeKind::Parameter,
                None,
                provenance,
            ),
            SemanticFact::ReturnType(fact) => self.add_reference_edge(
                record,
                fact.callable(),
                fact.return_type(),
                EdgeKind::Returns,
                provenance,
            ),
            SemanticFact::Contains(fact) => self.add_edge(
                record,
                GraphNodeId::Symbol(fact.container().clone()),
                GraphNodeId::Symbol(fact.member().clone()),
                EdgeKind::Contains,
                None,
                provenance,
            ),
            SemanticFact::Call(fact) => self.add_reference_edge(
                record,
                fact.caller(),
                fact.callee(),
                EdgeKind::Calls,
                provenance,
            ),
            SemanticFact::Reference(fact) => self.add_reference_edge(
                record,
                fact.source(),
                fact.target(),
                EdgeKind::References,
                provenance,
            ),
            SemanticFact::TypeRelation(fact) => self.add_type_relation(record, fact, provenance),
            SemanticFact::Import(fact) => self.add_import(record, fact, provenance),
            SemanticFact::Documentation(fact) => {
                let target =
                    self.ensure_unresolved(branchsense_core::QualifiedName::new("documentation")?)?;
                self.add_edge(
                    record,
                    GraphNodeId::Symbol(fact.subject().clone()),
                    target,
                    EdgeKind::Documents,
                    None,
                    provenance,
                )
            }
            SemanticFact::Annotation(fact) => self.add_annotation(record, fact, provenance),
            SemanticFact::Dependency(fact) => self.add_reference_edge(
                record,
                fact.source(),
                fact.target(),
                dependency_edge_kind(fact.kind()),
                provenance,
            ),
        }
    }

    fn add_type_relation(
        &mut self,
        record: &SemanticFactRecord,
        fact: &TypeRelationFact,
        provenance: Option<&FactProvenance>,
    ) -> Result<()> {
        self.add_reference_edge(
            record,
            fact.source(),
            fact.target(),
            match fact.relation() {
                TypeRelation::Extends => EdgeKind::Extends,
                TypeRelation::Implements => EdgeKind::Implements,
            },
            provenance,
        )
    }

    fn add_import(
        &mut self,
        record: &SemanticFactRecord,
        fact: &ImportFact,
        provenance: Option<&FactProvenance>,
    ) -> Result<()> {
        let target = branchsense_core::QualifiedName::new(fact.target().as_str())?;
        let (target_id, resolution) =
            if let Some(symbol) = self.qualified_index.get(&target).cloned() {
                (GraphNodeId::Symbol(symbol.clone()), ResolutionState::Resolved(symbol))
            } else {
                (self.ensure_unresolved(target.clone())?, ResolutionState::Unresolved)
            };
        self.add_edge(
            record,
            GraphNodeId::Document(fact.document().clone()),
            target_id,
            EdgeKind::Imports,
            Some(resolution),
            provenance,
        )
    }

    fn add_annotation(
        &mut self,
        record: &SemanticFactRecord,
        fact: &AnnotationFact,
        provenance: Option<&FactProvenance>,
    ) -> Result<()> {
        let target = branchsense_core::QualifiedName::new(fact.annotation().name().as_str())?;
        let target_id = self.ensure_unresolved(target)?;
        self.add_edge(
            record,
            GraphNodeId::Symbol(fact.subject().clone()),
            target_id,
            EdgeKind::Annotates,
            Some(ResolutionState::Unresolved),
            provenance,
        )
    }

    fn add_reference_edge<R>(
        &mut self,
        record: &SemanticFactRecord,
        source: &SymbolId,
        reference: &R,
        kind: EdgeKind,
        provenance: Option<&FactProvenance>,
    ) -> Result<()>
    where
        R: ReferenceTarget,
    {
        let (target, resolution) = self.reference_target(reference)?;
        self.add_edge(
            record,
            GraphNodeId::Symbol(source.clone()),
            target,
            kind,
            Some(resolution),
            provenance,
        )
    }

    fn reference_target<R>(&mut self, reference: &R) -> Result<(GraphNodeId, ResolutionState)>
    where
        R: ReferenceTarget,
    {
        let resolution = match reference.resolution() {
            ResolutionState::Unresolved => self
                .qualified_index
                .get(reference.name())
                .map_or(ResolutionState::Unresolved, |symbol| {
                    ResolutionState::Resolved(symbol.clone())
                }),
            resolution => resolution.clone(),
        };
        let target = match &resolution {
            ResolutionState::Resolved(symbol) => GraphNodeId::Symbol(symbol.clone()),
            ResolutionState::External(external) => {
                let id = external.clone();
                let node = GraphNode::External { id: id.clone(), name: reference.name().clone() };
                self.ensure_node(node)?;
                GraphNodeId::External(id)
            }
            ResolutionState::Unresolved
            | ResolutionState::Ambiguous(_)
            | ResolutionState::Invalid { .. } => {
                self.ensure_unresolved(reference.name().clone())?
            }
        };
        if !self.nodes.contains_key(&target) {
            return Err(GraphError::MissingEndpoint { edge_id: "reference-target".into() });
        }
        Ok((target, resolution))
    }

    fn ensure_unresolved(&mut self, name: branchsense_core::QualifiedName) -> Result<GraphNodeId> {
        let id = GraphNodeId::Unresolved(name.clone());
        self.ensure_node(GraphNode::Unresolved { name })?;
        Ok(id)
    }

    fn ensure_node(&mut self, node: GraphNode) -> Result<()> {
        let id = node.id();
        if let Some(existing) = self.nodes.get(&id) {
            if existing != &node {
                return Err(GraphError::DuplicateNode { node: id });
            }
            return Ok(());
        }
        self.insert_node(node)
    }

    fn add_edge(
        &mut self,
        record: &SemanticFactRecord,
        source: GraphNodeId,
        target: GraphNodeId,
        kind: EdgeKind,
        resolution: Option<ResolutionState>,
        provenance: Option<&FactProvenance>,
    ) -> Result<()> {
        if !self.nodes.contains_key(&source) || !self.nodes.contains_key(&target) {
            return Err(GraphError::MissingEndpoint { edge_id: record.id().as_str().into() });
        }
        let id = GraphEdgeId::from_fact(record.id());
        if self.edges.contains_key(&id) {
            return Err(GraphError::DuplicateEdge { edge_id: id.to_string() });
        }
        let edge = GraphEdge::new(
            id.clone(),
            source.clone(),
            target.clone(),
            kind,
            record.id().clone(),
            resolution,
            provenance.cloned(),
        );
        self.outgoing.entry(source).or_default().insert(id.clone());
        self.incoming.entry(target).or_default().insert(id.clone());
        self.edges.insert(id, edge);
        Ok(())
    }
}

trait ReferenceTarget {
    fn name(&self) -> &branchsense_core::QualifiedName;
    fn resolution(&self) -> &ResolutionState;
}

impl ReferenceTarget for SymbolReference {
    fn name(&self) -> &branchsense_core::QualifiedName {
        self.name()
    }
    fn resolution(&self) -> &ResolutionState {
        self.resolution()
    }
}

impl ReferenceTarget for TypeReference {
    fn name(&self) -> &branchsense_core::QualifiedName {
        self.name()
    }
    fn resolution(&self) -> &ResolutionState {
        self.resolution()
    }
}

fn dependency_edge_kind(kind: DependencyKind) -> EdgeKind {
    match kind {
        DependencyKind::Call => EdgeKind::Calls,
        DependencyKind::Inheritance => EdgeKind::Extends,
        DependencyKind::Implementation => EdgeKind::Implements,
        _ => EdgeKind::DependsOn,
    }
}

fn fact_location(record: &SemanticFactRecord) -> Option<&branchsense_core::Location> {
    match record.fact() {
        SemanticFact::Definition(fact) => Some(fact.location()),
        SemanticFact::Parameter(fact) => Some(fact.parameter().location()),
        SemanticFact::Call(fact) => Some(fact.location()),
        SemanticFact::Reference(fact) => Some(fact.location()),
        SemanticFact::TypeRelation(fact) => Some(fact.location()),
        SemanticFact::Import(fact) => Some(fact.location()),
        SemanticFact::Annotation(fact) => Some(fact.location()),
        SemanticFact::Dependency(fact) => fact.location(),
        SemanticFact::ReturnType(_)
        | SemanticFact::Contains(_)
        | SemanticFact::Documentation(_) => None,
    }
}

//! Public graph node, edge, and statistics values.

use std::fmt;

use branchsense_core::{DocumentId, QualifiedName, SymbolId};
use branchsense_semantic::{FactId, FactProvenance, ResolutionState, SymbolDefinition};
use serde::{Deserialize, Serialize};

/// Stable identity of a graph node.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum GraphNodeId {
    /// A source document node.
    Document(DocumentId),
    /// A declared semantic symbol.
    Symbol(SymbolId),
    /// A symbol known to exist outside the indexed workspace.
    External(String),
    /// A reference whose target is not currently resolved.
    Unresolved(QualifiedName),
}

/// Broad category of a graph node.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum NodeKind {
    /// A source document.
    Document,
    /// A declared symbol.
    Symbol,
    /// An external dependency symbol.
    External,
    /// An unresolved or ambiguous symbolic target.
    Unresolved,
}

impl GraphNodeId {
    /// Returns the node category.
    #[must_use]
    pub const fn kind(&self) -> NodeKind {
        match self {
            Self::Document(_) => NodeKind::Document,
            Self::Symbol(_) => NodeKind::Symbol,
            Self::External(_) => NodeKind::External,
            Self::Unresolved(_) => NodeKind::Unresolved,
        }
    }
}

/// A semantic graph node with source ownership where available.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GraphNode {
    /// A document that owns source facts.
    Document {
        /// Document identity.
        id: DocumentId,
    },
    /// A declaration-backed symbol.
    Symbol {
        /// Stable symbol definition.
        definition: SymbolDefinition,
        /// Owning document.
        document_id: DocumentId,
    },
    /// A known external symbol.
    External {
        /// External identity.
        id: String,
        /// Display name retained from the reference.
        name: QualifiedName,
    },
    /// An unresolved or ambiguous target.
    Unresolved {
        /// Display name retained from the reference.
        name: QualifiedName,
    },
}

impl GraphNode {
    /// Returns this node's stable identity.
    #[must_use]
    pub fn id(&self) -> GraphNodeId {
        match self {
            Self::Document { id } => GraphNodeId::Document(id.clone()),
            Self::Symbol { definition, .. } => GraphNodeId::Symbol(definition.id().clone()),
            Self::External { id, .. } => GraphNodeId::External(id.clone()),
            Self::Unresolved { name } => GraphNodeId::Unresolved(name.clone()),
        }
    }

    /// Returns the broad node category.
    #[must_use]
    pub fn kind(&self) -> NodeKind {
        self.id().kind()
    }

    /// Returns the owning document for source-backed nodes.
    #[must_use]
    pub fn document_id(&self) -> Option<&DocumentId> {
        match self {
            Self::Document { id } => Some(id),
            Self::Symbol { document_id, .. } => Some(document_id),
            Self::External { .. } | Self::Unresolved { .. } => None,
        }
    }
}

/// Stable identity of a graph edge.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GraphEdgeId(String);

impl GraphEdgeId {
    /// Creates an edge identity from a non-empty value.
    pub fn new(value: impl Into<String>) -> branchsense_semantic::Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(branchsense_semantic::SemanticError::EmptyValue {
                kind: "graph edge identifier",
            });
        }
        Ok(Self(value))
    }

    /// Creates a deterministic edge identity derived from a fact.
    pub fn from_fact(fact: &FactId) -> Self {
        Self(format!("edge:{}", fact.as_str()))
    }

    /// Returns the serialized identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for GraphEdgeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Semantic relationship represented by a graph edge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum EdgeKind {
    /// A declaration belongs to a document.
    Defines,
    /// A symbol contains another symbol.
    Contains,
    /// A callable invokes another callable.
    Calls,
    /// A symbol references another symbol.
    References,
    /// A document imports a target.
    Imports,
    /// A type extends another type.
    Extends,
    /// A type implements another type.
    Implements,
    /// A symbol depends on a target.
    DependsOn,
    /// A callable returns a target type.
    Returns,
    /// A callable owns a parameter.
    Parameter,
    /// A symbol has documentation.
    Documents,
    /// A symbol has an annotation.
    Annotates,
}

/// A provenance-preserving semantic graph edge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphEdge {
    id: GraphEdgeId,
    source: GraphNodeId,
    target: GraphNodeId,
    kind: EdgeKind,
    fact_id: FactId,
    resolution: Option<ResolutionState>,
    provenance: Option<FactProvenance>,
}

impl GraphEdge {
    /// Creates an edge value.
    #[must_use]
    pub(crate) fn new(
        id: GraphEdgeId,
        source: GraphNodeId,
        target: GraphNodeId,
        kind: EdgeKind,
        fact_id: FactId,
        resolution: Option<ResolutionState>,
        provenance: Option<FactProvenance>,
    ) -> Self {
        Self { id, source, target, kind, fact_id, resolution, provenance }
    }

    /// Returns the edge identity.
    #[must_use]
    pub fn id(&self) -> &GraphEdgeId {
        &self.id
    }

    /// Returns the source node identity.
    #[must_use]
    pub fn source(&self) -> &GraphNodeId {
        &self.source
    }

    /// Returns the target node identity.
    #[must_use]
    pub fn target(&self) -> &GraphNodeId {
        &self.target
    }

    /// Returns the semantic edge kind.
    #[must_use]
    pub const fn kind(&self) -> EdgeKind {
        self.kind
    }

    /// Returns the source fact identity.
    #[must_use]
    pub fn fact_id(&self) -> &FactId {
        &self.fact_id
    }

    /// Returns resolution state when the edge targets a symbolic reference.
    #[must_use]
    pub fn resolution(&self) -> Option<&ResolutionState> {
        self.resolution.as_ref()
    }

    /// Returns source provenance when supplied by the fact batch.
    #[must_use]
    pub fn provenance(&self) -> Option<&FactProvenance> {
        self.provenance.as_ref()
    }
}

/// Counts exposed by a graph snapshot.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphStatistics {
    nodes: usize,
    edges: usize,
    documents: usize,
    symbols: usize,
    external: usize,
    unresolved: usize,
}

impl GraphStatistics {
    /// Creates graph statistics.
    #[must_use]
    pub(crate) const fn new(
        nodes: usize,
        edges: usize,
        documents: usize,
        symbols: usize,
        external: usize,
        unresolved: usize,
    ) -> Self {
        Self { nodes, edges, documents, symbols, external, unresolved }
    }

    /// Returns total nodes.
    #[must_use]
    pub const fn nodes(&self) -> usize {
        self.nodes
    }
    /// Returns total edges.
    #[must_use]
    pub const fn edges(&self) -> usize {
        self.edges
    }
    /// Returns document nodes.
    #[must_use]
    pub const fn documents(&self) -> usize {
        self.documents
    }
    /// Returns declared symbol nodes.
    #[must_use]
    pub const fn symbols(&self) -> usize {
        self.symbols
    }
    /// Returns external nodes.
    #[must_use]
    pub const fn external(&self) -> usize {
        self.external
    }
    /// Returns unresolved nodes.
    #[must_use]
    pub const fn unresolved(&self) -> usize {
        self.unresolved
    }
}

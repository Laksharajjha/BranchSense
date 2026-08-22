//! Backend-independent query result values.

use branchsense_core::{Location, QualifiedName, SymbolId};
use branchsense_graph::EdgeKind;
use branchsense_semantic::{FactId, FactProvenance, ResolutionState, SymbolKind};

/// A declared symbol returned from a query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySymbol {
    id: SymbolId,
    kind: SymbolKind,
    name: String,
    qualified_name: Option<QualifiedName>,
    location: Location,
}

impl QuerySymbol {
    pub(crate) fn from_definition(definition: &branchsense_semantic::SymbolDefinition) -> Self {
        Self {
            id: definition.id().clone(),
            kind: definition.kind(),
            name: definition.name().as_str().to_owned(),
            qualified_name: definition.qualified_name().cloned(),
            location: definition.location().clone(),
        }
    }
    /// Returns the stable symbol identity.
    #[must_use]
    pub fn id(&self) -> &SymbolId {
        &self.id
    }
    /// Returns the semantic symbol kind.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }
    /// Returns the short display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the qualified name, when supplied by extraction.
    #[must_use]
    pub fn qualified_name(&self) -> Option<&QualifiedName> {
        self.qualified_name.as_ref()
    }
    /// Returns the declaration location.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// A graph node returned by a relationship query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryNode {
    /// A declared source symbol.
    Symbol(QuerySymbol),
    /// An external dependency symbol.
    External {
        /// External identity.
        id: branchsense_semantic::ExternalSymbolId,
        /// Display name retained from the graph.
        name: QualifiedName,
    },
    /// An unresolved or ambiguous target.
    Unresolved {
        /// Display name retained from the graph.
        name: QualifiedName,
    },
    /// A source document.
    Document(branchsense_core::DocumentId),
}

/// A relationship returned by a query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationshipResult {
    source: QueryNode,
    target: QueryNode,
    kind: EdgeKind,
    fact_id: FactId,
    resolution: Option<ResolutionState>,
    provenance: Option<FactProvenance>,
}

impl RelationshipResult {
    pub(crate) fn new(
        source: QueryNode,
        target: QueryNode,
        edge: &branchsense_graph::GraphEdge,
    ) -> Self {
        Self {
            source,
            target,
            kind: edge.kind(),
            fact_id: edge.fact_id().clone(),
            resolution: edge.resolution().cloned(),
            provenance: edge.provenance().cloned(),
        }
    }
    /// Returns the relationship source.
    #[must_use]
    pub fn source(&self) -> &QueryNode {
        &self.source
    }
    /// Returns the relationship target.
    #[must_use]
    pub fn target(&self) -> &QueryNode {
        &self.target
    }
    /// Returns the relationship kind.
    #[must_use]
    pub const fn kind(&self) -> EdgeKind {
        self.kind
    }
    /// Returns the source fact identity.
    #[must_use]
    pub fn fact_id(&self) -> &FactId {
        &self.fact_id
    }
    /// Returns the resolution state when present.
    #[must_use]
    pub fn resolution(&self) -> Option<&ResolutionState> {
        self.resolution.as_ref()
    }
    /// Returns source provenance when present.
    #[must_use]
    pub fn provenance(&self) -> Option<&FactProvenance> {
        self.provenance.as_ref()
    }
}

/// A deterministic collection of query results.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QueryResult<T> {
    items: Vec<T>,
}

impl<T> QueryResult<T> {
    pub(crate) fn new(items: Vec<T>) -> Self {
        Self { items }
    }
    /// Returns result items in deterministic order.
    #[must_use]
    pub fn items(&self) -> &[T] {
        &self.items
    }
    /// Returns the number of result items.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    /// Returns whether no items matched.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    /// Consumes the result and returns its items.
    #[must_use]
    pub fn into_items(self) -> Vec<T> {
        self.items
    }

    /// Iterates over result items without transferring ownership.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }
}

impl<'a, T> IntoIterator for &'a QueryResult<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

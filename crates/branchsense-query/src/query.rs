//! Typed read-only query operations.

use std::collections::BTreeSet;

use branchsense_core::{DocumentId, QualifiedName, SymbolId};
use branchsense_graph::{EdgeKind, GraphNode, GraphNodeId, SemanticGraph};
use branchsense_semantic::SymbolKind;

use crate::{
    error::{QueryError, Result},
    result::{QueryNode, QueryResult, QuerySymbol, RelationshipResult},
};

/// Limits and filters shared by bounded queries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QueryOptions {
    limit: Option<usize>,
}

impl QueryOptions {
    /// Creates default options with no result limit.
    #[must_use]
    pub const fn new() -> Self {
        Self { limit: None }
    }
    /// Limits the number of returned items.
    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    fn take<T>(&self, mut items: Vec<T>) -> Vec<T> {
        if let Some(limit) = self.limit {
            items.truncate(limit);
        }
        items
    }
}

/// A read-only view over one immutable semantic graph snapshot.
#[derive(Debug)]
pub struct Query<'graph> {
    graph: &'graph SemanticGraph,
}

#[allow(clippy::missing_errors_doc)]
impl<'graph> Query<'graph> {
    /// Creates a query view over `graph`.
    #[must_use]
    pub const fn new(graph: &'graph SemanticGraph) -> Self {
        Self { graph }
    }

    /// Finds a declared symbol by stable ID.
    pub fn symbol(&self, id: &SymbolId) -> Result<QuerySymbol> {
        self.graph
            .find_symbol(id)
            .and_then(GraphNode::definition)
            .map(QuerySymbol::from_definition)
            .ok_or_else(|| QueryError::SymbolIdNotFound(id.clone()))
    }

    /// Resolves one exact qualified name, rejecting ambiguity.
    pub fn symbol_by_qualified_name(&self, name: &QualifiedName) -> Result<QuerySymbol> {
        let matches = self.symbols_by_name(name.as_str());
        match matches.len() {
            0 => Err(QueryError::SymbolNotFound { name: name.to_string() }),
            1 => Ok(matches.into_items().remove(0)),
            count => Err(QueryError::AmbiguousSymbol { name: name.to_string(), count }),
        }
    }

    /// Finds declarations whose short or qualified name exactly equals `name`.
    #[must_use]
    pub fn symbols_by_name(&self, name: &str) -> QueryResult<QuerySymbol> {
        let mut symbols = self
            .graph
            .nodes()
            .filter_map(GraphNode::definition)
            .filter(|definition| {
                definition.name().as_str() == name
                    || definition
                        .qualified_name()
                        .is_some_and(|qualified| qualified.as_str() == name)
            })
            .map(QuerySymbol::from_definition)
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| left.id().cmp(right.id()));
        QueryResult::new(symbols)
    }

    /// Finds declarations with an optional kind and document filter.
    #[must_use]
    pub fn symbols(
        &self,
        kind: Option<SymbolKind>,
        document: Option<&DocumentId>,
    ) -> QueryResult<QuerySymbol> {
        let mut symbols = self
            .graph
            .nodes()
            .filter_map(GraphNode::definition)
            .filter(|definition| kind.is_none_or(|wanted| definition.kind() == wanted))
            .filter(|definition| {
                document.is_none_or(|wanted| definition.location().document_id() == wanted)
            })
            .map(QuerySymbol::from_definition)
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| left.id().cmp(right.id()));
        QueryResult::new(symbols)
    }

    /// Returns callers represented by `Calls` edges targeting `symbol`.
    pub fn callers(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(symbol, false, &[EdgeKind::Calls], options)
    }

    /// Returns callees represented by `Calls` edges originating at `symbol`.
    pub fn callees(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(symbol, true, &[EdgeKind::Calls], options)
    }

    /// Returns all incoming reference edges for `symbol`.
    pub fn references(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(symbol, false, &[EdgeKind::References], options)
    }

    /// Returns types that explicitly implement `symbol`.
    pub fn implementations(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(symbol, false, &[EdgeKind::Implements], options)
    }

    /// Returns types that explicitly extend `symbol`.
    pub fn subtypes(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(symbol, false, &[EdgeKind::Extends], options)
    }

    /// Returns direct semantic dependency edges originating at `symbol`.
    ///
    /// Calls, inheritance, implementation, and explicit dependency facts are
    /// included because the graph records each as a dependency edge kind;
    /// ordinary references are intentionally excluded.
    pub fn dependencies(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(
            symbol,
            true,
            &[EdgeKind::DependsOn, EdgeKind::Calls, EdgeKind::Extends, EdgeKind::Implements],
            options,
        )
    }

    /// Returns direct semantic dependency edges targeting `symbol`.
    pub fn dependents(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.relationships(
            symbol,
            false,
            &[EdgeKind::DependsOn, EdgeKind::Calls, EdgeKind::Extends, EdgeKind::Implements],
            options,
        )
    }

    /// Returns symbols directly contained by `symbol`.
    pub fn contents(
        &self,
        symbol: &SymbolId,
        options: QueryOptions,
    ) -> Result<QueryResult<QuerySymbol>> {
        self.symbol(symbol)?;
        let items = self
            .graph
            .outgoing_relationships(&GraphNodeId::Symbol(symbol.clone()))
            .into_iter()
            .filter(|edge| edge.kind() == EdgeKind::Contains)
            .filter_map(|edge| self.graph.node(edge.target()).and_then(GraphNode::definition))
            .map(QuerySymbol::from_definition)
            .collect();
        Ok(QueryResult::new(options.take(items)))
    }

    /// Returns symbols directly contained by a package with this exact name.
    pub fn package_contents(
        &self,
        package: &QualifiedName,
        options: QueryOptions,
    ) -> Result<QueryResult<QuerySymbol>> {
        let package_symbol = self.symbol_by_qualified_name(package)?;
        if package_symbol.kind() != SymbolKind::Package {
            return Ok(QueryResult::new(Vec::new()));
        }
        self.contents(package_symbol.id(), options)
    }

    /// Traverses dependency edges up to `max_depth`, without revisiting nodes.
    pub fn dependency_tree(
        &self,
        symbol: &SymbolId,
        max_depth: usize,
        options: QueryOptions,
    ) -> Result<QueryResult<QuerySymbol>> {
        if max_depth == 0 {
            return Err(QueryError::InvalidDepth);
        }
        self.symbol(symbol)?;
        let mut visited = BTreeSet::from([GraphNodeId::Symbol(symbol.clone())]);
        let mut frontier = vec![GraphNodeId::Symbol(symbol.clone())];
        let mut found = Vec::new();
        for _ in 0..max_depth {
            let mut next = Vec::new();
            for current in frontier {
                for edge in self.graph.outgoing_relationships(&current) {
                    if !matches!(
                        edge.kind(),
                        EdgeKind::DependsOn
                            | EdgeKind::Calls
                            | EdgeKind::Extends
                            | EdgeKind::Implements
                    ) {
                        continue;
                    }
                    if visited.insert(edge.target().clone()) {
                        if let Some(GraphNode::Symbol { definition, .. }) =
                            self.graph.node(edge.target())
                        {
                            found.push(QuerySymbol::from_definition(definition));
                            next.push(edge.target().clone());
                        }
                    }
                }
            }
            frontier = next;
            if frontier.is_empty() {
                break;
            }
        }
        found.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(QueryResult::new(options.take(found)))
    }

    fn relationships(
        &self,
        symbol: &SymbolId,
        outgoing: bool,
        kinds: &[EdgeKind],
        options: QueryOptions,
    ) -> Result<QueryResult<RelationshipResult>> {
        self.symbol(symbol)?;
        let node_id = GraphNodeId::Symbol(symbol.clone());
        let edges = if outgoing {
            self.graph.outgoing_relationships(&node_id)
        } else {
            self.graph.incoming_relationships(&node_id)
        };
        let mut results = edges
            .into_iter()
            .filter(|edge| kinds.contains(&edge.kind()))
            .filter_map(|edge| {
                let source = self.graph.node(edge.source()).map(query_node)?;
                let target = self.graph.node(edge.target()).map(query_node)?;
                Some(RelationshipResult::new(source, target, edge))
            })
            .collect::<Vec<_>>();
        results.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
        Ok(QueryResult::new(options.take(results)))
    }
}

fn query_node(node: &GraphNode) -> QueryNode {
    match node {
        GraphNode::Document { id } => QueryNode::Document(id.clone()),
        GraphNode::Symbol { definition, .. } => {
            QueryNode::Symbol(QuerySymbol::from_definition(definition))
        }
        GraphNode::External { id, name } => {
            QueryNode::External { id: id.clone(), name: name.clone() }
        }
        GraphNode::Unresolved { name } => QueryNode::Unresolved { name: name.clone() },
    }
}

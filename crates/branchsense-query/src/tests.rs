use branchsense_core::{RevisionId, SymbolId};
use branchsense_graph::SemanticGraph;

use crate::{Query, QueryError, QueryOptions};

fn graph() -> SemanticGraph {
    SemanticGraph::from_documents(RevisionId::new("revision:test").expect("revision"), Vec::new())
        .expect("empty graph")
}

#[test]
fn empty_snapshot_has_deterministic_empty_results() {
    let snapshot = graph();
    let query = Query::new(&snapshot);
    assert!(query.symbols_by_name("Missing").is_empty());
    assert!(query.symbols(None, None).is_empty());
}

#[test]
fn missing_symbol_and_invalid_depth_are_structured_errors() {
    let snapshot = graph();
    let query = Query::new(&snapshot);
    let id = SymbolId::new("symbol:missing").expect("symbol ID");
    assert!(matches!(query.symbol(&id), Err(QueryError::SymbolIdNotFound(_))));
    assert!(matches!(
        query.dependency_tree(&id, 0, QueryOptions::new()),
        Err(QueryError::InvalidDepth)
    ));
}

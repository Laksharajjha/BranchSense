use branchsense_core::{DocumentId, RevisionId};
use branchsense_semantic::SemanticFactSet;

use crate::{GraphNodeId, SemanticGraph};

#[test]
fn empty_document_graph_contains_only_document_node() {
    let document = DocumentId::new("src/Main.java").expect("document ID");
    let graph = SemanticGraph::from_document_facts(
        document.clone(),
        RevisionId::new("revision:one").expect("revision ID"),
        SemanticFactSet::default(),
    )
    .expect("graph builds");

    assert_eq!(graph.statistics().nodes(), 1);
    assert_eq!(graph.statistics().edges(), 0);
    assert!(graph.node(&GraphNodeId::Document(document)).is_some());
}

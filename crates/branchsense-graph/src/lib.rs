//! Immutable semantic graph snapshots derived from [`branchsense_semantic`].
//!
//! `branchsense-graph` is deliberately a repository-local semantic store, not
//! a parser, resolver, Git integration, query language, or branch-analysis
//! engine. Language adapters emit facts; this crate preserves those facts as
//! typed nodes and edges with explicit resolution state and source provenance.
//!
//! A graph update constructs a new [`SemanticGraph`] from the previous
//! document fact state. Existing readers retain the old value while the new
//! snapshot is built. The current implementation favors correctness and
//! deterministic behavior over incremental internal mutation; the public
//! [`FactDelta`] and document replacement APIs leave room for a later indexed
//! copy-on-write implementation without exposing a backend.
//!
//! # Example
//!
//! ```
//! use branchsense_core::{DocumentId, RevisionId};
//! use branchsense_graph::SemanticGraph;
//! use branchsense_semantic::SemanticFactSet;
//!
//! let graph = SemanticGraph::from_document_facts(
//!     DocumentId::new("src/Main.java").expect("document ID"),
//!     RevisionId::new("revision:one").expect("revision ID"),
//!     SemanticFactSet::default(),
//! )
//! .expect("empty document graph");
//! assert_eq!(graph.statistics().nodes(), 1);
//! ```

#![forbid(unsafe_code)]

mod error;
mod graph;
mod model;

pub use error::{GraphError, Result};
pub use graph::SemanticGraph;
pub use model::{
    EdgeKind, GraphEdge, GraphEdgeId, GraphNode, GraphNodeId, GraphStatistics, NodeKind,
};

pub use branchsense_semantic::{DocumentFactSet, FactDelta, FactUpdate};

#[cfg(test)]
mod tests;

//! Graph validation errors.

use branchsense_core::{DocumentId, RevisionId};
use thiserror::Error;

use crate::GraphNodeId;

/// Errors returned while constructing or updating a semantic graph.
#[derive(Debug, Error)]
pub enum GraphError {
    /// The same fact identity was supplied with different payloads.
    #[error("fact {fact_id} was supplied more than once with different payloads")]
    DuplicateFact {
        /// Conflicting semantic fact identity.
        fact_id: String,
    },
    /// A graph node identity was defined more than once with different data.
    #[error("graph node {node:?} was defined more than once")]
    DuplicateNode {
        /// Conflicting node identity.
        node: GraphNodeId,
    },
    /// A graph edge identity was defined more than once.
    #[error("graph edge {edge_id} was defined more than once")]
    DuplicateEdge {
        /// Conflicting edge identity.
        edge_id: String,
    },
    /// A fact referenced a document different from its owning batch.
    #[error("fact for document {actual} was submitted in document batch {expected}")]
    DocumentMismatch {
        /// Document found in fact provenance or location.
        actual: DocumentId,
        /// Document supplied by the update operation.
        expected: DocumentId,
    },
    /// An edge endpoint could not be represented by the graph.
    #[error("graph edge {edge_id} has no valid endpoint")]
    MissingEndpoint {
        /// Edge identity with the invalid endpoint.
        edge_id: String,
    },
    /// A delta belongs to a different revision than the graph update.
    #[error("graph revision {expected} does not match delta revision {actual}")]
    RevisionMismatch {
        /// Revision expected by the graph.
        expected: RevisionId,
        /// Revision supplied by the delta.
        actual: RevisionId,
    },
    /// A graph update could not construct a required semantic value.
    #[error("semantic graph value construction failed: {0}")]
    Semantic(#[from] branchsense_semantic::SemanticError),
    /// A core identity or location could not be constructed.
    #[error("core graph value construction failed: {0}")]
    Core(#[from] branchsense_core::ModelError),
}

/// Result type used by graph operations.
pub type Result<T> = std::result::Result<T, GraphError>;

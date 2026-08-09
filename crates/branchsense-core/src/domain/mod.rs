//! Aggregate-level semantic domain values.

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{entities::SemanticEntity, ids::RevisionId, relationships::SemanticRelationship};

/// Immutable entity and relationship collection for one semantic revision.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SemanticSnapshot {
    entities: Vec<SemanticEntity>,
    relationships: Vec<SemanticRelationship>,
}

impl SemanticSnapshot {
    /// Creates a snapshot from already validated domain values.
    pub fn new(entities: Vec<SemanticEntity>, relationships: Vec<SemanticRelationship>) -> Self {
        Self { entities, relationships }
    }
    pub fn entities(&self) -> &[SemanticEntity] {
        &self.entities
    }
    pub fn relationships(&self) -> &[SemanticRelationship] {
        &self.relationships
    }
}

/// Revision-pinned semantic model aggregate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SemanticModel {
    revision_id: RevisionId,
    snapshot: SemanticSnapshot,
}

impl SemanticModel {
    /// Creates a model pinned to one workspace revision.
    pub fn new(revision_id: RevisionId, snapshot: SemanticSnapshot) -> Self {
        Self { revision_id, snapshot }
    }
    pub fn revision_id(&self) -> &RevisionId {
        &self.revision_id
    }
    pub fn snapshot(&self) -> &SemanticSnapshot {
        &self.snapshot
    }
}

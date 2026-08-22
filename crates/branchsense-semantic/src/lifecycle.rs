//! Fact replacement, delta, and snapshot contracts.

use std::collections::{BTreeMap, BTreeSet};

use branchsense_core::{DocumentId, RevisionId};
use serde::{Deserialize, Serialize};

use crate::{FactId, SemanticError, SemanticFactRecord, SemanticFactSet, SnapshotIdentity};

/// One fact changed from an earlier value to a newer value.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactUpdate {
    before: SemanticFactRecord,
    after: SemanticFactRecord,
}

impl FactUpdate {
    /// Creates an update for one stable fact identity.
    #[must_use]
    pub fn new(before: SemanticFactRecord, after: SemanticFactRecord) -> Self {
        Self { before, after }
    }

    /// Returns the previous fact value.
    #[must_use]
    pub fn before(&self) -> &SemanticFactRecord {
        &self.before
    }

    /// Returns the replacement fact value.
    #[must_use]
    pub fn after(&self) -> &SemanticFactRecord {
        &self.after
    }
}

/// Document-scoped changes between two semantic fact sets.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactDelta {
    document_id: Option<DocumentId>,
    revision_id: Option<RevisionId>,
    added: Vec<SemanticFactRecord>,
    removed: Vec<FactId>,
    updated: Vec<FactUpdate>,
}

impl FactDelta {
    /// Computes a deterministic replacement delta for one document.
    ///
    /// A missing previous set means that every current fact is added. A
    /// missing current set can be represented by passing an empty set, which
    /// removes every fact from the previous document.
    #[must_use]
    pub fn between(
        document_id: DocumentId,
        revision_id: RevisionId,
        previous: Option<&SemanticFactSet>,
        current: &SemanticFactSet,
    ) -> Self {
        let previous_by_id = previous.map_or_else(BTreeMap::new, |set| {
            set.facts().iter().map(|record| (record.id().clone(), record)).collect()
        });
        let current_by_id: BTreeMap<FactId, &SemanticFactRecord> =
            current.facts().iter().map(|record| (record.id().clone(), record)).collect();

        let mut delta = Self {
            document_id: Some(document_id),
            revision_id: Some(revision_id),
            ..Self::default()
        };

        for (id, record) in &current_by_id {
            match previous_by_id.get(id) {
                None => delta.added.push((*record).clone()),
                Some(previous) if *previous != *record => {
                    delta.updated.push(FactUpdate::new((*previous).clone(), (*record).clone()));
                }
                Some(_) => {}
            }
        }
        for id in previous_by_id.keys() {
            if !current_by_id.contains_key(id) {
                delta.removed.push(id.clone());
            }
        }
        delta
    }

    /// Computes a full document deletion delta.
    #[must_use]
    pub fn delete(
        document_id: DocumentId,
        revision_id: RevisionId,
        previous: &SemanticFactSet,
    ) -> Self {
        Self::between(document_id, revision_id, Some(previous), &SemanticFactSet::default())
    }

    /// Returns the affected document, when this is a document-scoped delta.
    #[must_use]
    pub fn document_id(&self) -> Option<&DocumentId> {
        self.document_id.as_ref()
    }

    /// Returns the revision that produced this delta.
    #[must_use]
    pub fn revision_id(&self) -> Option<&RevisionId> {
        self.revision_id.as_ref()
    }

    /// Returns newly added facts.
    #[must_use]
    pub fn added(&self) -> &[SemanticFactRecord] {
        &self.added
    }

    /// Returns removed fact identities.
    #[must_use]
    pub fn removed(&self) -> &[FactId] {
        &self.removed
    }

    /// Returns facts whose stable identity remained but whose value changed.
    #[must_use]
    pub fn updated(&self) -> &[FactUpdate] {
        &self.updated
    }

    /// Returns whether the two fact sets were identical.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.updated.is_empty()
    }

    /// Returns the number of affected fact identities.
    #[must_use]
    pub fn changed_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.updated.len()
    }
}

/// Facts belonging to one source document in a snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentFactSet {
    document_id: DocumentId,
    facts: SemanticFactSet,
}

impl DocumentFactSet {
    /// Creates a document-owned fact set.
    #[must_use]
    pub fn new(document_id: DocumentId, facts: SemanticFactSet) -> Self {
        Self { document_id, facts }
    }

    /// Returns the owning document identity.
    #[must_use]
    pub fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the document's immutable facts.
    #[must_use]
    pub fn facts(&self) -> &SemanticFactSet {
        &self.facts
    }
}

/// Immutable, revision-pinned semantic fact snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactSnapshot {
    identity: SnapshotIdentity,
    documents: Vec<DocumentFactSet>,
}

impl FactSnapshot {
    /// Creates a deterministic snapshot and rejects duplicate documents.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::DuplicateDocument`] when a document appears
    /// more than once.
    pub fn new(
        identity: SnapshotIdentity,
        documents: Vec<DocumentFactSet>,
    ) -> Result<Self, SemanticError> {
        let mut seen = BTreeSet::new();
        for document in &documents {
            if !seen.insert(document.document_id().clone()) {
                return Err(SemanticError::DuplicateDocument {
                    document: document.document_id().clone(),
                });
            }
        }
        let mut documents = documents;
        documents.sort_by(|left, right| left.document_id().cmp(right.document_id()));
        Ok(Self { identity, documents })
    }

    /// Returns the repository/workspace/revision identity.
    #[must_use]
    pub fn identity(&self) -> &SnapshotIdentity {
        &self.identity
    }

    /// Returns document fact sets in canonical document-identity order.
    #[must_use]
    pub fn documents(&self) -> &[DocumentFactSet] {
        &self.documents
    }
}

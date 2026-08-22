//! Immutable semantic change values.

use std::path::PathBuf;

use branchsense_core::SymbolId;
use branchsense_semantic::{FactId, SemanticFactRecord, SymbolDefinition};
use serde::{Deserialize, Serialize};

/// Classification shared by document, fact, symbol, and relationship changes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ChangeKind {
    /// The value exists only in the after snapshot.
    Added,
    /// The value exists only in the before snapshot.
    Removed,
    /// The stable subject exists in both snapshots with different content.
    Modified,
    /// The value is identical in both snapshots.
    Unchanged,
}

/// A document-level semantic change.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentChange {
    path: PathBuf,
    kind: ChangeKind,
    before_hash: Option<String>,
    after_hash: Option<String>,
}

impl DocumentChange {
    pub(crate) fn new(
        path: PathBuf,
        kind: ChangeKind,
        before_hash: Option<String>,
        after_hash: Option<String>,
    ) -> Self {
        Self { path, kind, before_hash, after_hash }
    }

    /// Returns the repository-relative document path.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Returns the document change classification.
    #[must_use]
    pub const fn kind(&self) -> ChangeKind {
        self.kind
    }

    /// Returns the before content hash, when the document existed.
    #[must_use]
    pub fn before_hash(&self) -> Option<&str> {
        self.before_hash.as_deref()
    }

    /// Returns the after content hash, when the document exists.
    #[must_use]
    pub fn after_hash(&self) -> Option<&str> {
        self.after_hash.as_deref()
    }
}

/// A lower-level fact change owned by one document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactChange {
    document: PathBuf,
    id: FactId,
    kind: ChangeKind,
    before: Option<SemanticFactRecord>,
    after: Option<SemanticFactRecord>,
}

impl FactChange {
    pub(crate) fn new(
        document: PathBuf,
        id: FactId,
        kind: ChangeKind,
        before: Option<SemanticFactRecord>,
        after: Option<SemanticFactRecord>,
    ) -> Self {
        Self { document, id, kind, before, after }
    }

    /// Returns the repository-relative owning document.
    #[must_use]
    pub fn document(&self) -> &std::path::Path {
        &self.document
    }

    /// Returns the stable fact identity.
    #[must_use]
    pub fn id(&self) -> &FactId {
        &self.id
    }

    /// Returns the fact change classification.
    #[must_use]
    pub const fn kind(&self) -> ChangeKind {
        self.kind
    }

    /// Returns the before fact, when present.
    #[must_use]
    pub fn before(&self) -> Option<&SemanticFactRecord> {
        self.before.as_ref()
    }

    /// Returns the after fact, when present.
    #[must_use]
    pub fn after(&self) -> Option<&SemanticFactRecord> {
        self.after.as_ref()
    }
}

/// A reason attached to a modified symbol.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum SymbolChangeReason {
    /// The declaration kind or identity-bearing name changed.
    DefinitionChanged,
    /// A callable's parameter-list spelling changed.
    MethodSignatureChanged,
    /// A callable return type changed.
    ReturnTypeChanged,
    /// A callable gained a parameter.
    ParameterAdded,
    /// A callable lost a parameter.
    ParameterRemoved,
    /// A callable parameter type changed.
    ParameterTypeChanged,
    /// Visibility changed.
    VisibilityChanged,
    /// One or more modifiers changed.
    ModifierChanged,
    /// A containing symbol changed.
    ContainerChanged,
    /// Documentation changed.
    DocumentationChanged,
    /// An annotation changed.
    AnnotationChanged,
    /// A field's type dependency changed.
    FieldTypeChanged,
    /// A superclass changed.
    SuperclassChanged,
    /// An implemented interface was added.
    InterfaceAdded,
    /// An implemented interface was removed.
    InterfaceRemoved,
}

/// A declaration-level change, paired conservatively across snapshots.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SymbolChange {
    before_id: Option<SymbolId>,
    after_id: Option<SymbolId>,
    kind: ChangeKind,
    before: Option<SymbolDefinition>,
    after: Option<SymbolDefinition>,
    reasons: Vec<SymbolChangeReason>,
}

impl SymbolChange {
    pub(crate) fn new(
        before_id: Option<SymbolId>,
        after_id: Option<SymbolId>,
        kind: ChangeKind,
        before: Option<SymbolDefinition>,
        after: Option<SymbolDefinition>,
        reasons: Vec<SymbolChangeReason>,
    ) -> Self {
        Self { before_id, after_id, kind, before, after, reasons }
    }

    /// Returns the before symbol identity, when present.
    #[must_use]
    pub fn before_id(&self) -> Option<&SymbolId> {
        self.before_id.as_ref()
    }

    /// Returns the after symbol identity, when present.
    #[must_use]
    pub fn after_id(&self) -> Option<&SymbolId> {
        self.after_id.as_ref()
    }

    /// Returns the symbol change classification.
    #[must_use]
    pub const fn kind(&self) -> ChangeKind {
        self.kind
    }

    /// Returns the before declaration, when present.
    #[must_use]
    pub fn before(&self) -> Option<&SymbolDefinition> {
        self.before.as_ref()
    }

    /// Returns the after declaration, when present.
    #[must_use]
    pub fn after(&self) -> Option<&SymbolDefinition> {
        self.after.as_ref()
    }

    /// Returns deterministic reasons for a modified declaration.
    #[must_use]
    pub fn reasons(&self) -> &[SymbolChangeReason] {
        &self.reasons
    }
}

/// Semantic relationship category represented by a fact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum RelationshipKind {
    /// A containment relationship.
    Contains,
    /// A call relationship.
    Call,
    /// A symbol reference.
    Reference,
    /// An import relationship.
    Import,
    /// An inheritance or implementation relationship.
    TypeRelation,
    /// A dependency relationship.
    Dependency,
}

/// A relationship change derived from semantic relationship facts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RelationshipChange {
    fact: FactChange,
    relationship: RelationshipKind,
}

impl RelationshipChange {
    pub(crate) fn new(fact: FactChange, relationship: RelationshipKind) -> Self {
        Self { fact, relationship }
    }

    /// Returns the relationship category.
    #[must_use]
    pub const fn relationship(&self) -> RelationshipKind {
        self.relationship
    }

    /// Returns the underlying fact change.
    #[must_use]
    pub fn fact(&self) -> &FactChange {
        &self.fact
    }
}

/// Summary counts for one semantic diff.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DiffStatistics {
    documents_added: usize,
    documents_removed: usize,
    documents_modified: usize,
    documents_unchanged: usize,
    symbols_added: usize,
    symbols_removed: usize,
    symbols_modified: usize,
    symbols_unchanged: usize,
    relationships_added: usize,
    relationships_removed: usize,
    relationships_modified: usize,
    facts_added: usize,
    facts_removed: usize,
    facts_modified: usize,
    facts_unchanged: usize,
}

impl DiffStatistics {
    pub(crate) fn count_document(&mut self, kind: ChangeKind) {
        match kind {
            ChangeKind::Added => self.documents_added += 1,
            ChangeKind::Removed => self.documents_removed += 1,
            ChangeKind::Modified => self.documents_modified += 1,
            ChangeKind::Unchanged => self.documents_unchanged += 1,
        }
    }
    pub(crate) fn count_symbol(&mut self, kind: ChangeKind) {
        match kind {
            ChangeKind::Added => self.symbols_added += 1,
            ChangeKind::Removed => self.symbols_removed += 1,
            ChangeKind::Modified => self.symbols_modified += 1,
            ChangeKind::Unchanged => self.symbols_unchanged += 1,
        }
    }
    pub(crate) fn count_fact(&mut self, kind: ChangeKind) {
        match kind {
            ChangeKind::Added => self.facts_added += 1,
            ChangeKind::Removed => self.facts_removed += 1,
            ChangeKind::Modified => self.facts_modified += 1,
            ChangeKind::Unchanged => self.facts_unchanged += 1,
        }
    }
    pub(crate) fn count_relationship(&mut self, kind: ChangeKind) {
        match kind {
            ChangeKind::Added => self.relationships_added += 1,
            ChangeKind::Removed => self.relationships_removed += 1,
            ChangeKind::Modified => self.relationships_modified += 1,
            ChangeKind::Unchanged => {}
        }
    }

    /// Returns added documents.
    #[must_use]
    pub const fn documents_added(&self) -> usize {
        self.documents_added
    }
    /// Returns removed documents.
    #[must_use]
    pub const fn documents_removed(&self) -> usize {
        self.documents_removed
    }
    /// Returns modified documents.
    #[must_use]
    pub const fn documents_modified(&self) -> usize {
        self.documents_modified
    }
    /// Returns unchanged documents.
    #[must_use]
    pub const fn documents_unchanged(&self) -> usize {
        self.documents_unchanged
    }
    /// Returns added symbols.
    #[must_use]
    pub const fn symbols_added(&self) -> usize {
        self.symbols_added
    }
    /// Returns removed symbols.
    #[must_use]
    pub const fn symbols_removed(&self) -> usize {
        self.symbols_removed
    }
    /// Returns modified symbols.
    #[must_use]
    pub const fn symbols_modified(&self) -> usize {
        self.symbols_modified
    }
    /// Returns unchanged symbols.
    #[must_use]
    pub const fn symbols_unchanged(&self) -> usize {
        self.symbols_unchanged
    }
    /// Returns added relationships.
    #[must_use]
    pub const fn relationships_added(&self) -> usize {
        self.relationships_added
    }
    /// Returns removed relationships.
    #[must_use]
    pub const fn relationships_removed(&self) -> usize {
        self.relationships_removed
    }
    /// Returns modified relationships.
    #[must_use]
    pub const fn relationships_modified(&self) -> usize {
        self.relationships_modified
    }
    /// Returns added facts.
    #[must_use]
    pub const fn facts_added(&self) -> usize {
        self.facts_added
    }
    /// Returns removed facts.
    #[must_use]
    pub const fn facts_removed(&self) -> usize {
        self.facts_removed
    }
    /// Returns modified facts.
    #[must_use]
    pub const fn facts_modified(&self) -> usize {
        self.facts_modified
    }
    /// Returns unchanged facts.
    #[must_use]
    pub const fn facts_unchanged(&self) -> usize {
        self.facts_unchanged
    }
}

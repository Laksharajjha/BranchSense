//! Canonical language-independent semantic facts for `BranchSense`.
//!
//! A parser describes syntax: nodes, tokens, and recovery artifacts. A
//! semantic adapter describes meaning: a type is defined, a method belongs to
//! a type, a call targets a symbol, or one declaration implements another.
//! This crate is the vocabulary for those facts.
//!
//! The crate deliberately contains no parser nodes, syntax tokens, language
//! implementations, graph containers, Git concepts, or persistence. Language
//! adapters translate their private syntax representations into
//! [`SemanticFact`] values. A future graph builder can then index those facts,
//! while diffing and Git subsystems can compare fact sets without depending on
//! a parser technology.
//!
//! # Semantic facts, not syntax
//!
//! A Java method declaration and a Rust function item can have very different
//! syntax while both producing a [`SymbolDefinition`] with a callable
//! [`SymbolKind`]. An unresolved call can still be represented with a
//! [`SymbolReference`] and resolved later without changing the fact's shape.
//!
//! # Identity and locations
//!
//! Symbol identity is supplied by the semantic adapter through the stable
//! [`branchsense_core::SymbolId`] type. Names and locations retain useful
//! provenance, but consumers should use IDs—not display names—as references.
//! Locations point back to parser-owned documents through the core location
//! value object; they do not expose syntax trees.
//!
//! # Graph construction boundary
//!
//! [`SemanticFactSet`] is only an immutable transport collection. It has no
//! adjacency, traversal, mutation, or storage behavior. A later graph crate
//! may consume one fact set, resolve references, and produce graph deltas
//! without introducing graph concerns into this vocabulary crate.
//!
//! Fact sets may carry [`FactProvenance`]. [`FactDelta`] compares the facts
//! owned by one document and reports additions, removals, and stable-identity
//! updates. [`FactSnapshot`] groups document fact sets under a repository,
//! workspace, and revision identity without becoming a graph or persistence
//! layer. These contracts let a future store replace one document atomically
//! while keeping readers on immutable snapshots.
//!
//! [`ResolutionState`] makes unresolved, ambiguous, external, and invalid
//! references distinct from resolved symbols. Consumers must not treat an
//! unresolved reference as a graph edge to a known symbol.
//!
//! # Example
//!
//! ```
//! use branchsense_core::{DocumentId, Location, Name, Position, Range, SymbolId};
//! use branchsense_semantic::{SymbolDefinition, SymbolKind};
//!
//! let document = DocumentId::new("src/payment.rs").expect("document ID");
//! let location = Location::new(
//!     document,
//!     Range::new(Position::new(0, 0, 0), Position::new(0, 7, 7))
//!         .expect("ordered range"),
//! );
//! let payment = SymbolDefinition::new(
//!     SymbolId::new("symbol:payment").expect("symbol ID"),
//!     SymbolKind::Type,
//!     Name::new("Payment").expect("symbol name"),
//!     location,
//! );
//! assert_eq!(payment.kind(), SymbolKind::Type);
//! ```

#![forbid(unsafe_code)]

mod error;
mod evaluation;
mod evidence;
mod facts;
mod identity;
mod ids;
mod lifecycle;
mod provenance;
mod values;

pub use error::{Result, SemanticError};
pub use evaluation::{
    DatasetSchemaVersion, EvalOutcome, EvalRecord, EvalRepositoryIdentity, EvalRevision,
    LabelProvenance, OutcomeConfidence, PredictedOrdinalAssessment,
};
pub use evidence::{
    AbstentionDecision, CompletenessIssue, CompletenessScope, CompletenessSource,
    EvidenceCompleteness, EvidenceEnvelope, EvidenceIdentity, EvidenceKind, EvidenceLedger,
    EvidenceLink, EvidenceRelation, EvidenceState, ObservationIdentity,
};
pub use facts::{
    AnnotationFact, CallFact, ContainsFact, DependencyFact, DocumentationFact, ImportFact,
    ParameterFact, ReferenceFact, ReturnTypeFact, SemanticFact, SemanticFactRecord,
    SymbolDefinition, TypeRelation, TypeRelationFact,
};
pub use identity::{IdentityMatch, SemanticEntityIdentity, canonical_identity, revision_local_id};
pub use ids::FactId;
pub use lifecycle::{DocumentFactSet, FactDelta, FactSnapshot, FactUpdate};
pub use provenance::{
    AnalysisProvenance, ConfigurationFingerprint, ContentHash, FactProvenance, ProducerIdentity,
    SnapshotIdentity,
};
pub use values::{
    Annotation, AnnotationArgument, AnnotationValue, DependencyKind, Documentation,
    ExternalSymbolId, ReferenceKind, ResolutionState, SemanticFactSet, SymbolKind, SymbolReference,
    TypeReference,
};

#[cfg(test)]
mod tests;

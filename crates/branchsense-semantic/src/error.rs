//! Validation errors for semantic values and snapshots.

use branchsense_core::DocumentId;

use thiserror::Error;

/// Errors returned when constructing semantic values.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum SemanticError {
    /// A required semantic value was empty or whitespace-only.
    #[error("{kind} cannot be empty")]
    EmptyValue {
        /// The kind of value that was empty.
        kind: &'static str,
    },
    /// A snapshot contained more than one fact set for the same document.
    #[error("snapshot contains duplicate document {document}")]
    DuplicateDocument {
        /// The duplicated document identity.
        document: DocumentId,
    },
}

/// Result type used by semantic value constructors.
pub type Result<T> = std::result::Result<T, SemanticError>;

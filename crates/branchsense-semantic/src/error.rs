//! Validation errors for semantic values.

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
}

/// Result type used by semantic value constructors.
pub type Result<T> = std::result::Result<T, SemanticError>;

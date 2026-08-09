//! Errors produced while constructing semantic model values.

use thiserror::Error;

/// Errors specific to semantic model validation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ModelError {
    /// A typed identifier or name cannot be empty.
    #[error("{kind} cannot be empty")]
    EmptyValue {
        /// The kind of value that was empty.
        kind: &'static str,
    },
    /// A range end precedes its start.
    #[error("range end must not precede range start")]
    InvalidRange,
    /// A workspace root must be absolute.
    #[error("workspace root must be an absolute path: {path}")]
    RelativeWorkspaceRoot {
        /// The invalid path supplied by the caller.
        path: String,
    },
}

/// Errors emitted by the core crate.
#[derive(Debug, Error)]
pub enum CoreError {
    /// A workspace root must be absolute to avoid ambiguous identity.
    #[error("workspace root must be an absolute path: {path}")]
    RelativeWorkspaceRoot {
        /// The invalid path supplied by the caller.
        path: String,
    },
}

/// The standard result type for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

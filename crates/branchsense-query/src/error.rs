//! Query errors.

use branchsense_core::SymbolId;
use thiserror::Error;

/// Errors returned by semantic queries.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum QueryError {
    /// The requested symbol does not exist in the snapshot.
    #[error("symbol `{name}` was not found")]
    SymbolNotFound {
        /// Exact name that was requested.
        name: String,
    },
    /// Exact name resolution returned more than one symbol.
    #[error("symbol `{name}` is ambiguous ({count} matches)")]
    AmbiguousSymbol {
        /// Exact name that was requested.
        name: String,
        /// Number of matching declarations.
        count: usize,
    },
    /// A symbol ID was requested but does not exist.
    #[error("symbol `{0}` was not found")]
    SymbolIdNotFound(SymbolId),
    /// A query depth was outside the supported range.
    #[error("query depth must be at least one")]
    InvalidDepth,
}

/// Result alias for query operations.
pub type Result<T> = std::result::Result<T, QueryError>;

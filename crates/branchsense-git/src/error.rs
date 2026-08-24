//! Error contracts for Git operations.

/// Errors raised by read-only Git operations.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// The requested path is not inside a discoverable repository.
    #[error("Git repository discovery failed: {0}")]
    Discovery(String),
    /// The repository operation failed.
    #[error("Git operation failed: {0}")]
    Operation(String),
    /// Semantic indexing failed before a Git snapshot could be published.
    #[error("Git semantic indexing failed: {0}")]
    Index(#[source] branchsense_index::IndexError),
    /// The repository has no commit at the requested symbolic location.
    #[error("Git reference `{0}` does not resolve to a commit")]
    NotACommit(String),
    /// A Git object identifier could not be represented by the domain model.
    #[error("invalid Git object identifier: {0}")]
    InvalidObjectId(String),
    /// A Git commit contained metadata that could not be decoded.
    #[error("invalid Git commit metadata: {0}")]
    InvalidMetadata(String),
}

impl GitError {
    pub(crate) fn operation(error: impl std::fmt::Display) -> Self {
        Self::Operation(error.to_string())
    }
}

/// Result alias for Git operations.
pub type Result<T, E = GitError> = std::result::Result<T, E>;

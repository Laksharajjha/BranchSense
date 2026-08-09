//! CLI-specific error contracts.

use thiserror::Error;

/// Errors produced by the `BranchSense` command-line interface.
#[derive(Debug, Error)]
pub enum CliError {
    /// Logging could not be initialized for this process.
    #[error("failed to initialize logging: {0}")]
    Logging(String),
    /// A requested command could not complete.
    #[error("{0}")]
    Command(String),
}

/// The standard result type for CLI operations.
pub type Result<T> = std::result::Result<T, CliError>;

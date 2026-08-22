//! Source discovery errors.

use std::path::PathBuf;

use thiserror::Error;

/// Errors that prevent repository source discovery from starting.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// The requested path could not be inspected.
    #[error("cannot inspect repository path `{path}`: {source}")]
    Io {
        /// Path that could not be inspected.
        path: PathBuf,
        /// Underlying filesystem error.
        #[source]
        source: std::io::Error,
    },
    /// The requested path is neither a file nor a directory.
    #[error("repository path `{0}` is not a file or directory")]
    InvalidRoot(PathBuf),
    /// A source file is outside the discovered root.
    #[error("source path `{path}` is outside repository root `{root}`")]
    OutsideRoot {
        /// Discovered source path.
        path: PathBuf,
        /// Canonical repository root.
        root: PathBuf,
    },
}

/// Result alias for discovery operations.
pub type Result<T> = std::result::Result<T, DiscoveryError>;

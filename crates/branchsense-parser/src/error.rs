//! Parser and registry error contracts.

use std::{io, path::PathBuf};

use branchsense_core::Language;
use thiserror::Error;

/// Errors returned while parsing a document.
#[derive(Debug, Error)]
pub enum ParseError {
    /// The source file could not be read.
    #[error("failed to read source file {path}: {source}")]
    Io {
        /// The source path that could not be read.
        path: PathBuf,
        /// The underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// The source exceeds the configured limit.
    #[error("source is {actual_bytes} bytes, exceeding the configured limit of {max_bytes} bytes")]
    SourceTooLarge {
        /// Actual UTF-8 source size in bytes.
        actual_bytes: usize,
        /// Configured maximum source size in bytes.
        max_bytes: usize,
    },
    /// The parser does not support the requested language or operation.
    #[error("unsupported parser operation for {language:?}: {operation}")]
    Unsupported {
        /// Language associated with the operation.
        language: Language,
        /// Operation that is not supported.
        operation: &'static str,
    },
    /// An adapter received input for another language.
    #[error("parser language mismatch: expected {expected:?}, received {actual:?}")]
    LanguageMismatch {
        /// Language expected by the adapter.
        expected: Language,
        /// Language reported by the input or tree.
        actual: Language,
    },
    /// An adapter-specific parse failure.
    #[error("parser adapter failure: {message}")]
    Adapter {
        /// Human-readable adapter failure context.
        message: String,
    },
}

/// Errors returned by parser registration and lookup.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// No parser has been registered for a language.
    #[error("no parser registered for {0:?}")]
    NotRegistered(Language),
    /// A language already has a parser registration.
    #[error("a parser is already registered for {0:?}")]
    AlreadyRegistered(Language),
    /// Creating a parser from an adapter failed.
    #[error("failed to register parser adapter: {0}")]
    Adapter(#[source] ParseError),
}

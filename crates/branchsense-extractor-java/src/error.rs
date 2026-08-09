//! Errors that prevent semantic extraction from starting.

use branchsense_core::Language;
use thiserror::Error;

/// Errors returned when a document cannot be used by the Java extractor.
#[derive(Debug, Error)]
pub enum ExtractionError {
    /// The document was produced for another language.
    #[error("Java extraction received {actual:?} document")]
    LanguageMismatch {
        /// Language reported by the parsed document.
        actual: Language,
    },
    /// The document does not contain the Java adapter's syntax tree.
    #[error("parsed document does not contain a Java syntax tree")]
    SyntaxTreeMismatch,
    /// The parser did not retain source text required for semantic names.
    #[error("parsed document does not retain source text")]
    SourceUnavailable,
    /// A required semantic value could not be constructed.
    #[error("semantic value construction failed: {0}")]
    Semantic(#[from] branchsense_semantic::SemanticError),
    /// A core identity or source location could not be constructed.
    #[error("core value construction failed: {0}")]
    Core(#[from] branchsense_core::ModelError),
}

/// Result type for extractor startup and document validation failures.
pub type Result<T> = std::result::Result<T, ExtractionError>;

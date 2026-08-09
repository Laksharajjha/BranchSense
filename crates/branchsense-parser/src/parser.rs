//! Parser and language-adapter traits.

#![allow(clippy::missing_errors_doc)]

use std::{future::Future, path::Path, pin::Pin, sync::Arc};

use branchsense_core::Language;

use crate::{
    configuration::ParserConfiguration,
    document::{DocumentVersion, ParseInput, ParsedDocument, TextEdit},
    error::ParseError,
    result::ParseResult,
};

/// A boxed future used by the async-ready parser contract.
pub type ParseFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ParseResult, ParseError>> + Send + 'a>>;

/// Language-neutral parser implementation contract.
pub trait Parser: Send + Sync {
    /// Returns the language handled by this parser.
    fn language(&self) -> Language;

    /// Returns immutable parser configuration.
    fn configuration(&self) -> &ParserConfiguration;

    /// Parses a source file from disk.
    fn parse(&self, path: &Path) -> Result<ParseResult, ParseError> {
        let source = std::fs::read_to_string(path)
            .map_err(|source| ParseError::Io { path: path.to_path_buf(), source })?;
        let input = ParseInput::new(path, source, DocumentVersion::initial());
        input.validate(self.configuration().max_source_bytes())?;
        self.parse_source(input)
    }

    /// Parses supplied source text, including unsaved editor content.
    fn parse_source(&self, input: ParseInput) -> Result<ParseResult, ParseError>;

    /// Applies edits to a previous parsed document when supported.
    fn parse_incremental(
        &self,
        previous: &ParsedDocument,
        edits: &[TextEdit],
    ) -> Result<ParseResult, ParseError> {
        let _ = (previous, edits);
        Err(ParseError::Unsupported { language: self.language(), operation: "incremental parsing" })
    }

    /// Returns an async-ready invocation using the synchronous implementation by default.
    fn parse_async(&self, input: ParseInput) -> ParseFuture<'_> {
        Box::pin(async move { self.parse_source(input) })
    }
}

/// Factory contract used by language adapters to register parser instances.
pub trait LanguageAdapter: Send + Sync {
    /// Returns the language provided by this adapter.
    fn language(&self) -> Language;

    /// Constructs a parser with registry-owned configuration.
    fn create_parser(
        &self,
        configuration: &ParserConfiguration,
    ) -> Result<Arc<dyn Parser>, ParseError>;
}

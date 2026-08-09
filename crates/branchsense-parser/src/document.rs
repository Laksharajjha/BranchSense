//! Parser inputs, immutable parsed documents, and incremental edits.

use std::{
    any::Any,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use branchsense_core::{Language, Range};

use crate::error::ParseError;

/// Monotonic version of a document supplied to a parser.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentVersion(u64);

impl DocumentVersion {
    /// The first version of a document.
    #[must_use]
    pub const fn initial() -> Self {
        Self(0)
    }

    /// Creates a version from a sequence number.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the sequence number.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Returns the next version, saturating at the maximum representable value.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Source text and identity supplied to a parser.
#[derive(Clone, Debug)]
pub struct ParseInput {
    path: PathBuf,
    source: Arc<str>,
    version: DocumentVersion,
}

impl ParseInput {
    /// Creates a parser input from source text.
    #[must_use]
    pub fn new(
        path: impl Into<PathBuf>,
        source: impl Into<Arc<str>>,
        version: DocumentVersion,
    ) -> Self {
        Self { path: path.into(), source: source.into(), version }
    }

    /// Returns the input path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the input source.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the input version.
    #[must_use]
    pub const fn version(&self) -> DocumentVersion {
        self.version
    }

    pub(crate) fn validate(&self, max_source_bytes: usize) -> Result<(), ParseError> {
        if self.source.len() > max_source_bytes {
            return Err(ParseError::SourceTooLarge {
                actual_bytes: self.source.len(),
                max_bytes: max_source_bytes,
            });
        }
        Ok(())
    }
}

/// Opaque syntax-tree boundary implemented by a language adapter.
pub trait SyntaxTree: Any + fmt::Debug + Send + Sync {
    /// Returns the language that produced this tree.
    fn language(&self) -> Language;

    /// Exposes the adapter-owned tree for an explicitly coordinated downcast.
    fn as_any(&self) -> &dyn Any;
}

/// Immutable parsed representation of one document revision.
#[derive(Clone)]
pub struct ParsedDocument {
    path: PathBuf,
    language: Language,
    source: Arc<str>,
    version: DocumentVersion,
    syntax_tree: Arc<dyn SyntaxTree>,
}

impl fmt::Debug for ParsedDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ParsedDocument")
            .field("path", &self.path)
            .field("language", &self.language)
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
}

impl ParsedDocument {
    /// Creates a parsed document from an adapter-owned syntax tree.
    pub fn new(input: ParseInput, syntax_tree: Arc<dyn SyntaxTree>, retain_source: bool) -> Self {
        let source = if retain_source { input.source.clone() } else { Arc::from("") };
        Self {
            path: input.path,
            language: syntax_tree.language(),
            source,
            version: input.version,
            syntax_tree,
        }
    }

    /// Returns the parsed path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the opaque syntax tree.
    #[must_use]
    pub fn syntax_tree(&self) -> &dyn SyntaxTree {
        self.syntax_tree.as_ref()
    }

    /// Returns the source text, or an empty string when retention is disabled.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the document version.
    #[must_use]
    pub const fn version(&self) -> DocumentVersion {
        self.version
    }

    /// Returns the language that produced this document.
    #[must_use]
    pub const fn language(&self) -> Language {
        self.language
    }
}

/// A text replacement used for incremental parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEdit {
    range: Range,
    replacement: String,
}

impl TextEdit {
    /// Creates a replacement over a source range.
    #[must_use]
    pub fn new(range: Range, replacement: impl Into<String>) -> Self {
        Self { range, replacement: replacement.into() }
    }

    /// Returns the replaced source range.
    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    /// Returns replacement text.
    #[must_use]
    pub fn replacement(&self) -> &str {
        &self.replacement
    }
}

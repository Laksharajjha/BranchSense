//! Parse diagnostics and successful parse results.

use branchsense_core::Range;
use serde::{Deserialize, Serialize};

use crate::document::ParsedDocument;

/// Severity of a parser diagnostic.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DiagnosticSeverity {
    /// The parser found a syntax error.
    Error,
    /// The parser found a recoverable issue.
    Warning,
    /// Informational parser output.
    Info,
}

/// A source-relative parser diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParseDiagnostic {
    severity: DiagnosticSeverity,
    message: String,
    range: Option<Range>,
}

impl ParseDiagnostic {
    /// Creates a parser diagnostic.
    #[must_use]
    pub fn new(
        severity: DiagnosticSeverity,
        message: impl Into<String>,
        range: Option<Range>,
    ) -> Self {
        Self { severity, message: message.into(), range }
    }
    /// Returns diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }
    /// Returns diagnostic text.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Returns the optional source range.
    #[must_use]
    pub const fn range(&self) -> Option<Range> {
        self.range
    }
}

/// Successful parse output, including recoverable diagnostics.
#[derive(Clone, Debug)]
pub struct ParseResult {
    document: ParsedDocument,
    diagnostics: Vec<ParseDiagnostic>,
}

impl ParseResult {
    /// Creates a successful parse result.
    #[must_use]
    pub fn new(document: ParsedDocument, diagnostics: Vec<ParseDiagnostic>) -> Self {
        Self { document, diagnostics }
    }
    /// Returns the parsed document.
    #[must_use]
    pub fn document(&self) -> &ParsedDocument {
        &self.document
    }
    /// Consumes the result and returns its parsed document.
    #[must_use]
    pub fn into_document(self) -> ParsedDocument {
        self.document
    }
    /// Returns recoverable diagnostics in adapter order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }
    /// Returns whether any diagnostic is an error.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }
}

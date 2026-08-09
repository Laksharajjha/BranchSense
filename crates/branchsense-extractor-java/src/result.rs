//! Extraction output and recoverable diagnostics.

use branchsense_core::Location;
use branchsense_semantic::SemanticFactSet;
use serde::{Deserialize, Serialize};

/// Severity of an extraction diagnostic.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ExtractionSeverity {
    /// Extraction could not interpret one syntax region but continued.
    Error,
    /// Extraction made a conservative choice for an incomplete construct.
    Warning,
}

/// Structured, source-relative extraction diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExtractionDiagnostic {
    severity: ExtractionSeverity,
    message: String,
    location: Option<Location>,
}

impl ExtractionDiagnostic {
    /// Creates an extraction diagnostic.
    #[must_use]
    pub fn new(
        severity: ExtractionSeverity,
        message: impl Into<String>,
        location: Option<Location>,
    ) -> Self {
        Self { severity, message: message.into(), location }
    }

    /// Returns diagnostic severity.
    #[must_use]
    pub const fn severity(&self) -> ExtractionSeverity {
        self.severity
    }

    /// Returns the diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the optional source location.
    #[must_use]
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

/// Semantic facts and recoverable diagnostics for one parsed document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractionResult {
    facts: SemanticFactSet,
    diagnostics: Vec<ExtractionDiagnostic>,
}

impl ExtractionResult {
    pub(crate) fn new(facts: SemanticFactSet, diagnostics: Vec<ExtractionDiagnostic>) -> Self {
        Self { facts, diagnostics }
    }

    /// Returns the immutable extracted fact set.
    #[must_use]
    pub fn facts(&self) -> &SemanticFactSet {
        &self.facts
    }

    /// Returns recoverable extraction diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[ExtractionDiagnostic] {
        &self.diagnostics
    }

    /// Returns whether extraction reported an error diagnostic.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|diagnostic| diagnostic.severity == ExtractionSeverity::Error)
    }
}

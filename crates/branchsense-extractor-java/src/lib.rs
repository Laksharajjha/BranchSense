//! Deterministic semantic extraction for Java parsed documents.
//!
//! `branchsense-java` owns parsing and exposes only an adapter-owned,
//! Tree-sitter-free node query surface. This crate consumes that surface and
//! emits the language-independent facts from `branchsense-semantic`.
//!
//! # Mapping rules
//!
//! Java constructs map to facts as follows:
//!
//! - `package_declaration` becomes a [`branchsense_semantic::SymbolKind::Package`]
//!   definition.
//! - `import_declaration` becomes an [`branchsense_semantic::ImportFact`].
//! - Class, interface, enum, method, constructor, field, parameter, and enum
//!   constant declarations become [`branchsense_semantic::SymbolDefinition`]
//!   values with the corresponding [`branchsense_semantic::SymbolKind`].
//! - `class` inheritance and interface implementation become
//!   [`branchsense_semantic::TypeRelationFact`] values.
//! - Nested declarations become [`branchsense_semantic::ContainsFact`] values.
//! - Java modifiers map to the shared core visibility and modifier values.
//! - Javadoc comments become [`branchsense_semantic::DocumentationFact`] values
//!   and are also retained on the corresponding definition.
//! - Java annotations become [`branchsense_semantic::AnnotationFact`] values
//!   and are also retained on the corresponding definition.
//! - Method invocations become [`branchsense_semantic::CallFact`] values with
//!   unresolved or name-based targets when type resolution is unavailable.
//!
//! Extraction is intentionally not resolution. A reference preserves its
//! qualified spelling and can be resolved by a future workspace semantic pass.
//! The extractor does not build graphs, indexes, or Git state.
//!
//! # Recovery
//!
//! Tree-sitter recovery nodes are converted to structured diagnostics. They do
//! not abort extraction; valid declarations surrounding malformed syntax are
//! still emitted whenever their ranges and names are available.

#![forbid(unsafe_code)]

mod error;
mod extractor;
mod result;

pub use error::{ExtractionError, Result};
pub use extractor::JavaExtractor;
pub use result::{ExtractionDiagnostic, ExtractionResult, ExtractionSeverity};

#[cfg(test)]
mod tests;

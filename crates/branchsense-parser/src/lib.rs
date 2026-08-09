//! Language-agnostic parser contracts for `BranchSense`.
//!
//! This crate intentionally has no syntax-tree implementation and no parser
//! generator dependency. Language adapters implement [`Parser`] and expose
//! their implementation through [`ParserRegistry`].

#![forbid(unsafe_code)]

pub mod configuration;
pub mod document;
pub mod error;
pub mod parser;
pub mod registry;
pub mod result;

pub use configuration::ParserConfiguration;
pub use document::{DocumentVersion, ParseInput, ParsedDocument, SyntaxTree, TextEdit};
pub use error::{ParseError, RegistryError};
pub use parser::{LanguageAdapter, ParseFuture, Parser};
pub use registry::ParserRegistry;
pub use result::{DiagnosticSeverity, ParseDiagnostic, ParseResult};

#[cfg(test)]
mod tests;

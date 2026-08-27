//! Language adapter framework for `BranchSense`.
//!
//! This crate isolates language-specific parser implementations from the
//! parser abstraction. It contains no grammar, parser generator, semantic
//! extraction, graph, or Git implementation.

#![forbid(unsafe_code)]

pub mod adapter;
pub mod capabilities;
pub mod error;
pub mod lifecycle;
pub mod metadata;
pub mod parser_bridge;
pub mod registry;
pub mod version;

pub use adapter::{AdapterConfig, LanguageAdapter};
pub use capabilities::{Capabilities, Capability, FeatureRequest, NegotiatedFeatures};
pub use error::{AdapterError, RegistryError};
pub use lifecycle::{AdapterSession, AdapterState};
pub use metadata::AdapterMetadata;
pub use parser_bridge::ParserAdapterBridge;
pub use registry::AdapterRegistry;
pub use version::{ADAPTER_API_VERSION, Version, VersionRange};

#[cfg(test)]
mod tests;

//! Repository-aware source discovery for `BranchSense`.
//!
//! This crate starts the repository indexing boundary without moving parser or
//! semantic responsibilities into the CLI. Discovery is deterministic,
//! symlink-safe, and configurable. Later indexing stages consume its relative
//! Java source paths and can build one immutable graph snapshot for a project.
#![forbid(unsafe_code)]

mod discovery;
mod error;

pub use discovery::{DiscoveredFile, DiscoveryOptions, DiscoveryResult, SourceDiscovery};
pub use error::{DiscoveryError, Result};

#[cfg(test)]
mod tests;

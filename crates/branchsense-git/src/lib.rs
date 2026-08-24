//! Read-only Git repository primitives for `BranchSense`.
//!
//! This crate translates gitoxide values into BranchSense-owned domain types.
//! No public API exposes `gix` implementation types, and no operation mutates
//! refs, the index, or the working tree.
#![forbid(unsafe_code)]

mod error;
mod repository;
mod revision;
mod snapshot;

pub use error::{GitError, Result};
pub use repository::{GitRef, GitRefKind, GitRepository, MergeBaseResult, RepositoryIdentity};
pub use revision::{GitCommitId, GitRevision, GitSignature, GitTreeId};
pub use snapshot::{GitSemanticSnapshot, GitSnapshotIndexer};

#[cfg(test)]
mod tests;

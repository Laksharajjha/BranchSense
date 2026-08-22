//! Deterministic semantic comparison of immutable `BranchSense` snapshots.
//!
//! [`SemanticDiffer`] compares two [`branchsense_index::SemanticIndexSnapshot`]
//! values without parsing source, invoking Git, or mutating either snapshot.
//! It reports document, fact, symbol, and relationship changes in stable order.
//!
//! A semantic diff is deliberately different from a textual diff. Source line
//! movement is ignored, while changes to declarations, signatures, references,
//! and other emitted semantic facts are retained. Symbol matching is
//! conservative: a method whose identity includes a changed signature can be
//! paired with its old declaration only when its kind and signature-independent
//! qualified name provide an unambiguous match. Rename detection is not
//! attempted.
//!
//! Snapshot persistence and a user-facing `diff` command are intentionally
//! deferred. The current index owns immutable in-memory snapshots, which keeps
//! this crate independent from storage and Git concerns.
#![forbid(unsafe_code)]

mod change;
mod diff;

pub use change::{
    ChangeKind, DiffStatistics, DocumentChange, FactChange, RelationshipChange, RelationshipKind,
    SymbolChange, SymbolChangeReason,
};
pub use diff::{SemanticDiff, SemanticDiffer};

#[cfg(test)]
mod tests;

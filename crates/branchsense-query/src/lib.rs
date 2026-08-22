//! Read-only, typed semantic queries over [`branchsense_graph::SemanticGraph`].
//!
//! The query engine is deliberately a thin view over an immutable graph
//! snapshot. It owns no graph, parser, index, or cache. Results are copied
//! into stable value types so callers never depend on the graph backend.
#![forbid(unsafe_code)]

mod error;
mod query;
mod result;

pub use error::{QueryError, Result};
pub use query::{Query, QueryOptions};
pub use result::{QueryNode, QueryResult, QuerySymbol, RelationshipResult};

#[cfg(test)]
mod tests;

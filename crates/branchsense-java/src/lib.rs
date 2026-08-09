//! Java language adapter for `BranchSense`.
//!
//! Tree-sitter is intentionally confined to this crate. Consumers receive
//! only the generic parser interfaces and the adapter-owned statistics value.

#![forbid(unsafe_code)]

mod adapter;
mod parser;
mod syntax_tree;

pub use adapter::JavaAdapter;
pub use parser::JavaParser;
pub use syntax_tree::{JavaSyntaxTree, TreeStatistics};

#[cfg(test)]
mod tests;

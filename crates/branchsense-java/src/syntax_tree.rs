//! Opaque Java syntax-tree wrapper and adapter-neutral statistics.

use std::{any::Any, fmt};

use branchsense_core::Language;
use branchsense_parser::SyntaxTree;
use tree_sitter::{Node, Tree};

/// Statistics computed from a Java syntax tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeStatistics {
    node_count: usize,
    depth: usize,
}

impl TreeStatistics {
    /// Returns the total number of nodes, including the root node.
    #[must_use]
    pub const fn node_count(self) -> usize {
        self.node_count
    }

    /// Returns the maximum root-to-leaf depth, with the root at depth one.
    #[must_use]
    pub const fn depth(self) -> usize {
        self.depth
    }
}

/// Adapter-owned Java syntax tree hidden behind the generic parser interface.
pub struct JavaSyntaxTree {
    tree: Tree,
}

impl JavaSyntaxTree {
    pub(crate) fn new(tree: Tree) -> Self {
        Self { tree }
    }

    /// Computes structural statistics without exposing Tree-sitter types.
    #[must_use]
    pub fn statistics(&self) -> TreeStatistics {
        let (node_count, depth) = statistics(self.tree.root_node(), 1);
        TreeStatistics { node_count, depth }
    }

    pub(crate) fn clone_tree(&self) -> Tree {
        self.tree.clone()
    }
}

impl fmt::Debug for JavaSyntaxTree {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("JavaSyntaxTree").field("statistics", &self.statistics()).finish()
    }
}

impl SyntaxTree for JavaSyntaxTree {
    fn language(&self) -> Language {
        Language::Java
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn statistics(node: Node<'_>, depth: usize) -> (usize, usize) {
    let mut cursor = node.walk();
    let mut node_count = 1;
    let mut max_depth = depth;
    for child in node.children(&mut cursor) {
        let (child_count, child_depth) = statistics(child, depth + 1);
        node_count += child_count;
        max_depth = max_depth.max(child_depth);
    }
    (node_count, max_depth)
}

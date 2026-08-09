//! Opaque Java syntax-tree wrapper and adapter-neutral statistics.

use std::{any::Any, fmt};

use branchsense_core::Language;
use branchsense_parser::SyntaxTree;
use tree_sitter::{Node, Tree};

/// A read-only Java syntax node exposed without leaking Tree-sitter types.
///
/// The node is valid only while the borrowed [`JavaSyntaxTree`] remains alive.
/// Language-specific consumers should use its semantic properties and source
/// ranges rather than depending on parser implementation details.
#[derive(Clone, Copy)]
pub struct JavaNode<'tree> {
    node: Node<'tree>,
}

impl fmt::Debug for JavaNode<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JavaNode")
            .field("kind", &self.kind())
            .field("start_byte", &self.start_byte())
            .field("end_byte", &self.end_byte())
            .finish()
    }
}

impl JavaNode<'_> {
    /// Returns the grammar-independent node kind supplied by the Java adapter.
    #[must_use]
    pub fn kind(&self) -> &str {
        self.node.kind()
    }

    /// Returns whether the parser marked this node as an error.
    #[must_use]
    pub fn is_error(&self) -> bool {
        self.node.is_error()
    }

    /// Returns whether the parser inserted this missing node during recovery.
    #[must_use]
    pub fn is_missing(&self) -> bool {
        self.node.is_missing()
    }

    /// Returns the inclusive start byte offset in the source document.
    #[must_use]
    pub fn start_byte(&self) -> usize {
        self.node.start_byte()
    }

    /// Returns the exclusive end byte offset in the source document.
    #[must_use]
    pub fn end_byte(&self) -> usize {
        self.node.end_byte()
    }

    /// Returns the zero-based start line and column.
    #[must_use]
    pub fn start_position(&self) -> (usize, usize) {
        let point = self.node.start_position();
        (point.row, point.column)
    }

    /// Returns the zero-based exclusive end line and column.
    #[must_use]
    pub fn end_position(&self) -> (usize, usize) {
        let point = self.node.end_position();
        (point.row, point.column)
    }

    /// Returns the number of direct children, including anonymous syntax nodes.
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.node.child_count()
    }

    /// Returns the number of direct named children.
    #[must_use]
    pub fn named_child_count(&self) -> usize {
        self.node.named_child_count()
    }

    /// Returns a direct child by zero-based index.
    #[must_use]
    pub fn child(&self, index: usize) -> Option<Self> {
        u32::try_from(index).ok().and_then(|index| self.node.child(index)).map(|node| Self { node })
    }

    /// Returns a direct named child by zero-based named-child index.
    #[must_use]
    pub fn named_child(&self, index: usize) -> Option<Self> {
        u32::try_from(index)
            .ok()
            .and_then(|index| self.node.named_child(index))
            .map(|node| Self { node })
    }

    /// Returns a direct child associated with a grammar field name.
    #[must_use]
    pub fn child_by_field_name(&self, name: &str) -> Option<Self> {
        self.node.child_by_field_name(name).map(|node| Self { node })
    }
}

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

    /// Returns the root node through the adapter-owned query surface.
    #[must_use]
    pub fn root_node(&self) -> JavaNode<'_> {
        JavaNode { node: self.tree.root_node() }
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

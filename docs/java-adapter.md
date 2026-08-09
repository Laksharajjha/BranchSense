# Java Adapter

`branchsense-java` is the first concrete language implementation. It is the
only crate in the current workspace that imports `tree-sitter` or
`tree-sitter-java`.

## Runtime behavior

`JavaAdapter` publishes incremental parsing and diagnostics capabilities. Its
session creates a thread-safe `JavaParser`, which returns the generic
`ParsedDocument` and hides the Tree-sitter `Tree` behind `JavaSyntaxTree`.
Consumers may request adapter-owned `TreeStatistics` without importing
Tree-sitter.

Malformed Java returns a recovered syntax tree and structured diagnostics when
Tree-sitter can recover. Incremental edits reuse the previous Java tree and
advance `DocumentVersion`; invalid UTF-8-boundary edits return a parser error.

Semantic extraction, graph construction, Git analysis, and BCS are deliberately
outside this adapter milestone.

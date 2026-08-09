# Parser Abstraction

`branchsense-parser` defines the boundary between the semantic engine and
language adapters. It owns no grammar and has no dependency on Tree-sitter or
any other parser generator.

## Contract

- `Parser` handles file-backed parsing, source-buffer parsing, and the default
  incremental operation.
- `ParsedDocument` is immutable and exposes path, language, source, version,
  and an opaque `SyntaxTree`.
- `SyntaxTree` is implemented by an adapter. The generic layer can inspect its
  language and support explicit adapter-owned downcasts, but cannot depend on
  a concrete tree representation.
- `ParseResult` carries the parsed document and recoverable diagnostics.
- `ParserConfiguration` carries immutable limits and behavior switches.
- `LanguageAdapter` constructs configured parser instances.
- `ParserRegistry` is cloneable, thread-safe, instance-owned state; it has no
  global registration or singleton lifecycle.

## Incremental and asynchronous behavior

`TextEdit` and `DocumentVersion` define the stable incremental input contract.
Adapters override `Parser::parse_incremental` when they support incremental
trees; the default returns an explicit unsupported-operation error rather than
silently reparsing.

`Parser::parse_async` returns a boxed `Send` future and defaults to invoking
the synchronous implementation. Adapters can replace that implementation when
their parser runtime is asynchronous. The abstraction does not require a
particular executor.

## Language boundary

The parser crate supports `Java`, `Kotlin`, `Go`, `Rust`, `TypeScript`, and
`Python` through the shared `Language` value. No language parser is included in
this milestone. Tree-sitter belongs in a future language adapter crate and must
not leak into this public contract.

# Repository Indexing

`branchsense-index` is the repository-aware orchestration layer. It composes
source discovery, the existing Java parser, the existing Java extractor, and
the existing semantic graph without moving language-specific logic into the
CLI.

## Discovery

`SourceDiscovery` canonicalizes the requested root and returns Java files as
repository-relative paths in deterministic order. The default ignored
directory basenames are `target`, `build`, `.gradle`, `.idea`, and `.git`.
They can be removed or extended through `DiscoveryOptions`. Symlinked files
and directories are skipped rather than followed, preventing traversal loops
and path-identity ambiguity.

## Identities and provenance

The current pre-Git identity is deliberately path-scoped:

- `RepositoryId`, `WorkspaceId`, and `ProjectId` are derived from the
  canonical root path.
- `DocumentId` is the repository-relative path.
- Each document receives a deterministic content hash and provenance linking
  it to repository, workspace, project, and index revision identities.

Path-scoped identities keep two separate filesystem checkouts from colliding.
Git-backed repository identity is intentionally deferred until the Git
discovery milestone.

## Lifecycle

`RepositoryIndex::index(path, previous)` performs one complete pass:

1. Discover and sort Java files.
2. Read and hash each source.
3. Reuse unchanged document facts.
4. Parse and extract changed or new documents.
5. Apply document replacement and deletion to a graph snapshot.
6. Publish one `SemanticIndexSnapshot` only after the pass completes.

Readers holding the previous snapshot are unaffected. The current graph
backend rebuilds derived indexes during document replacement; the indexer
avoids reparsing unchanged files, while copy-on-write graph optimization
remains future work.

## Diagnostics

Malformed Java syntax is recoverable: parser diagnostics are counted and
partial facts are still indexed. Fatal read, parse-start, and extraction
errors are associated with their relative file path and do not abort other
files. The report separates discovered, indexed, unchanged, skipped, parse
diagnostic, and extraction diagnostic counts.

## Cross-file semantics

The repository graph contains facts from every discovered document and query
APIs operate over that one graph. The current Java extractor emits unresolved
method calls and type references when classpath and scope information is not
available. The indexer preserves those states and never equates symbols by
short name alone. Deterministic cross-file resolution requires import, scope,
and Java classpath modeling and is the next semantic analysis milestone.

## CLI

```text
branchsense index .
branchsense callers --project . billing.PaymentService.process()
branchsense callees --project . billing.PaymentService.process()
```

The project query commands build one repository graph for the invocation;
persistent local index storage is not part of this milestone.

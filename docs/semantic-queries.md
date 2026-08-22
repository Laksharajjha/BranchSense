# Semantic Queries

`branchsense-query` is a read-only view over one immutable
`branchsense-graph::SemanticGraph` snapshot. It does not parse source, own a
second graph, mutate indexes, or resolve syntax. The graph remains the source
of truth.

## API

Construct a query view with `Query::new(&graph)`. The API is typed Rust rather
than a query language:

- `symbol` and `symbol_by_qualified_name` perform exact identity lookup.
- `symbols_by_name` and `symbols` provide exact deterministic enumeration.
- `callers`, `callees`, and `references` return relationship results.
- `implementations` and `subtypes` expose explicit type edges.
- `dependencies` and `dependents` include calls, inheritance,
  implementation, and explicit dependency edges, but not ordinary references.
- `contents` and `package_contents` expose direct containment.
- `dependency_tree` performs bounded, cycle-safe dependency traversal.

Results are copied into backend-independent values. They expose symbol IDs,
kinds, names, qualified names, locations, edge kinds, fact IDs, resolution
state, and provenance where the graph has those values. Petgraph or another
storage backend never appears in the public API.

## Resolution

Qualified-name lookup is exact. A missing name returns `SymbolNotFound`; more
than one exact match returns `AmbiguousSymbol`. Query results do not hide
unresolved, ambiguous, invalid, or external graph targets. A relationship
whose target is not resolved is returned with a `QueryNode::Unresolved` or
`QueryNode::External` and retains its `ResolutionState`.

The current Java extractor intentionally emits unresolved call targets when
cross-document resolution is not yet available. The query layer preserves
that fact rather than fabricating callers or symbol identities.

## Determinism and traversal

The graph uses ordered indexes. Query results are additionally ordered by
stable symbol or fact identity. Bounded traversal uses a visited set, expands
only semantic dependency edge kinds, removes duplicates, and applies an
optional result limit after deterministic ordering.

## CLI

The first CLI surface operates on a single Java source graph because a
workspace graph loader is not part of this milestone:

```text
branchsense callers --file PaymentService.java billing.PaymentService.process()
branchsense callees --file PaymentService.java billing.PaymentService.process()
branchsense references --file PaymentService.java billing.PaymentService.process()
branchsense implementations --file Repository.java billing.Repository
branchsense dependencies --file PaymentService.java billing.PaymentService.process()
```

Repository-wide loading, cross-file symbol resolution, Git revisions, and
impact analysis are later milestones rather than implicit CLI behavior.

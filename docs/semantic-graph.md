# Semantic Graph

`branchsense-graph` is the first repository-level semantic graph for
BranchSense. It consumes `SemanticFactSet` and `FactDelta` from
`branchsense-semantic`; it does not parse source, resolve classpaths, inspect
Git, or predict branch collisions.

## Model

Definitions and parameters become symbol nodes. Every document receives a
document node. Resolved references target declared symbol nodes. External
references receive external nodes, while unresolved, ambiguous, and invalid
references receive unresolved nodes and retain their original
`ResolutionState` on the edge.

Fact variants become typed edges: definitions, containment, calls,
references, imports, inheritance, implementation, dependencies, return types,
parameters, documentation, and annotations. Each edge retains its source
`FactId` and optional `FactProvenance`.

## Snapshots and updates

`SemanticGraph` is an immutable snapshot value. `replace_document_facts` and
`remove_document` clone the document fact state, rebuild indexes, and return a
new snapshot. Existing readers can continue using the old graph while the new
one is constructed; no partially updated state is observable.

`apply_delta` accepts the canonical document-scoped `FactDelta`. The current
implementation rebuilds derived indexes from retained document facts rather
than claiming copy-on-write performance. This is deliberate: correctness and
determinism are established first, while the public delta boundary permits a
later incremental implementation.

## Indexes and backend boundary

The implementation uses ordered standard-library maps and sets for node,
edge, symbol, document-ownership, outgoing-edge, and incoming-edge indexes.
Ordered collections provide deterministic traversal and serialization without
exposing a graph backend. `petgraph` is not used and no backend type appears in
the public API.

## Future consumers

A future query engine can consume immutable graph snapshots for definitions,
references, callers, callees, containment, and impact analysis. Git
intelligence can compare revision-pinned snapshots and apply document deltas.
Neither subsystem is part of this milestone.

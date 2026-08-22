# Semantic State Foundation

`branchsense-semantic` is the canonical ingestion model for the future
semantic graph. Language extractors emit `SemanticFactSet`; graph storage will
consume fact deltas and publish immutable fact snapshots. The older entity and
relationship types in `branchsense-core` remain domain values for workspace
and aggregate modeling. They are not replaced by a second graph store here.

## Identity

`RepositoryId` distinguishes repositories independently of their filesystem
mount path. `WorkspaceId`, `ProjectId`, `DocumentId`, and `RevisionId` identify
the corresponding scope. Current Java symbol IDs remain path-derived for
backward compatibility; future workspace orchestration can provide canonical
document identities and rename continuity without changing fact shapes.

No perfect rename detection is claimed by this milestone.

## Provenance

`FactProvenance` records the repository, workspace, optional project, document,
revision, source content hash, producing adapter/extractor identity, and an
optional extraction configuration fingerprint. These values explain where a
fact batch came from and prevent consumers from treating facts from different
revisions as interchangeable.

## Lifecycle

`FactDelta::between` compares facts by `FactId` and reports `added`, `removed`,
and `updated` records. A stable ID with a changed payload is an update. A
missing document is represented by `FactDelta::delete`. Equal fact sets produce
an empty delta. This is a document replacement contract, not a graph update
implementation.

## Resolution

`ResolutionState` distinguishes `Resolved`, `Unresolved`, `Ambiguous`,
`External`, and `Invalid` references. Unresolved and ambiguous references are
not resolved graph edges. Resolution remains a later workspace analysis stage;
the Java extractor does not perform classpath or cross-file resolution yet.

## Snapshots

`FactSnapshot` groups immutable document fact sets under a
`SnapshotIdentity`, which pins repository, workspace, optional project, and
revision. It rejects duplicate document entries and provides deterministic
serialization through stable value types. It has no adjacency, traversal,
query, persistence, or global mutable state.

The future graph can consume a previous snapshot plus document-scoped
`FactDelta` values, then publish its own immutable graph snapshot. That
implementation is intentionally deferred.

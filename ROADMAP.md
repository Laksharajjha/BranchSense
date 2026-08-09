# BranchSense Roadmap

## Milestone 1 — Repository foundation

- Cargo workspace and dependency policy
- Core domain crate and CLI binary
- Structured logging and typed error boundaries
- CI, formatting, linting, documentation, and contribution process
- `branchsense --help` and `branchsense version`

## Milestone 2 — Workspace identity and Git discovery

- Repository discovery through `gix`
- Canonical workspace/session lifecycle
- Configuration loading and diagnostic reporting

## Milestone 3 — Java syntax foundation

- Tree-sitter Java adapter ✅
- Incremental document parsing and syntax diagnostics ✅
- Parser conformance fixtures and large-file benchmark ✅

## Milestone 4 — Semantic graph foundation

- Stable symbol/dependency graph schema
- Transactional graph deltas and snapshots
- Incremental invalidation indexes

## Milestone 5 — Query and observability surface

- Versioned local query protocol
- CLI graph inspection and diagnostic bundles
- Performance benchmarks for warm incremental updates

## Future milestones

Editor clients, collaboration replication, and BCS-C consumers are deferred
until the local semantic engine is proven correct, incremental, and observable.

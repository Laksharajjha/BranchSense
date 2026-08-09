# BranchSense Roadmap

## Milestone 1 — Repository foundation ✅

- Cargo workspace and dependency policy
- Core domain crate and CLI binary
- Structured logging and typed error boundaries
- CI, formatting, linting, documentation, and contribution process
- `branchsense --help` and `branchsense version`

## Milestone 2 — Parser and adapter foundation ✅

- Language-neutral parser contracts and parsed-document model
- Adapter metadata, capability negotiation, lifecycle, and registry
- Incremental parsing and diagnostics interfaces

## Milestone 3 — Java syntax foundation ✅

- Tree-sitter Java adapter ✅
- Incremental document parsing and syntax diagnostics ✅
- Parser conformance fixtures and large-file benchmark ✅

## Milestone 4 — Semantic vocabulary ✅

- Language-independent symbol definitions and typed references
- Calls, imports, type relations, annotations, documentation, and dependencies
- Identified immutable fact records with serde support

## Milestone 5 — Java semantic extraction ✅

- Java declarations, imports, parameters, fields, annotations, and documentation
- Inheritance, interface implementation, containment, and call facts
- Partial extraction with structured malformed-source diagnostics

## Milestone 6 — Workspace identity and Git discovery

- Repository discovery through `gix`
- Canonical workspace/session lifecycle
- Configuration loading and diagnostic reporting

## Milestone 7 — Semantic graph foundation

- Stable symbol/dependency graph schema
- Transactional graph deltas and snapshots
- Incremental invalidation indexes

## Milestone 8 — Query and observability surface

- Versioned local query protocol
- CLI graph inspection and diagnostic bundles
- Performance benchmarks for warm incremental updates

## Future milestones

Editor clients, collaboration replication, and BCS-C consumers are deferred
until the local semantic engine is proven correct, incremental, and observable.

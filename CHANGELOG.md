# Changelog

All notable changes to BranchSense are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and releases use semantic versioning with an explicit alpha phase.

## [Unreleased]

### Planned

- Workspace identity and Git repository discovery.
- Graph impact queries and repository-wide query loading.

### Added

- `branchsense-index` with deterministic Java source discovery, ignored
  directory handling, repository-wide graph construction, structured
  diagnostics, and content-based incremental reuse.
- `branchsense index <path>` and `--project` query execution over one
  repository graph snapshot.
- `branchsense-query` with exact symbol lookup, relationship queries, and
  bounded deterministic traversal.
- Semantic query CLI commands with explicit `--file` source input.
- First immutable semantic graph crate consuming `SemanticFactSet` and `FactDelta`.
- Typed graph nodes and edges preserving unresolved, ambiguous, and external references.
- Atomic document replacement and deletion APIs with deterministic indexes.
- `branchsense inspect --graph` for graph statistics.

## [0.1.0-alpha] - 2026-08-09

The first public alpha establishes the local parsing foundation for BranchSense.
The APIs are experimental and may change before the first stable release.

### Added

- Immutable semantic domain model with strongly typed identifiers and values.
- Language-neutral parser abstraction with parsed documents, diagnostics,
  configuration, and incremental-edit contracts.
- Language adapter framework with capability discovery, lifecycle management,
  feature negotiation, and version compatibility.
- Tree-sitter-backed Java adapter with error recovery and incremental parsing.
- `branchsense parse <file>` for Java parse diagnostics and tree statistics.
- Unit tests, fixture tests, integration tests, and a large-source benchmark.
- Rust workspace quality gates, CI, contributor documentation, and roadmap.

### Not included

- Semantic extraction or symbol/dependency graph construction.
- Git analysis, merge-conflict prediction, BCS, networking, or editor clients.

The alpha APIs are experimental and may change without a compatibility
guarantee. See the project roadmap for planned milestones.

[Unreleased]: https://github.com/Laksharajjha/BranchSense/compare/v0.1.0-alpha...HEAD
[0.1.0-alpha]: https://github.com/Laksharajjha/BranchSense/releases/tag/v0.1.0-alpha

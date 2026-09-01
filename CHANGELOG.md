# Changelog

All notable changes to BranchSense are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and releases use semantic versioning with an explicit alpha phase.

## [Unreleased]

### Changed

- Added a shared analytical evidence envelope with explicit state,
  completeness, provenance, identities, and lineage.
- Preserved overload signatures in canonical semantic identities.
- Marked bounded history and responsibility analyses as truncated when their
  configured commit window excludes additional history.
- Added offline CLI integration coverage for branch analysis, history, and
  responsibility redaction.

### Planned

- Persistent semantic snapshots and richer Git provenance.
- BCS prediction after evidence-contract hardening and validation.

### Added

- `branchsense-ownership` for bounded, read-only contributor responsibility
  evidence derived from Git commit authors.
- Separate semantic symbol and document responsibility scopes with contribution
  counts, shares, recent contributors, concentration, and commit provenance.
- `branchsense ownership --repo --revision --max-commits` with optional `--json`
  output. Results are historical evidence, not ownership certainty.

- `branchsense-impact` for bounded, deterministic semantic impact analysis
  with structured causal explanations, direct and transitive caller traversal,
  and truncation statistics.
- `branchsense impact --repo --before --after` for Git-backed impact inspection.
- `branchsense-overlap` for deterministic semantic overlap evidence between two
  branch changes relative to their common merge base.
- `branchsense overlap --repo --base --branch-a --branch-b` for Git-backed
  branch overlap inspection.
- `branchsense-collision` for deterministic semantic collision factors,
  bounded evidence scoring, severity, and structured explanations.
- `branchsense analyze --repo --base --branch-a --branch-b` for concise
  collision assessment from Git-backed semantic snapshots.
- `branchsense-history` for bounded, revision-pinned frequency, recency,
  semantic co-change, and separate file co-change evidence.
- `branchsense history --repo --revision --max-commits` with optional `--json`
  output for historical analysis.

- `branchsense-diff` for deterministic document, fact, symbol, and relationship
  comparison between immutable semantic index snapshots.
- Structured callable signature-change reasons, including parameter and return
  type changes, without claiming unsupported rename detection.
- `branchsense-index` with deterministic Java source discovery, ignored
  directory handling, repository-wide graph construction, structured
  diagnostics, and content-based incremental reuse.
- Exact fully qualified cross-file imports and references resolve through the
  graph symbol index; short-name and scope-dependent references remain explicit.
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

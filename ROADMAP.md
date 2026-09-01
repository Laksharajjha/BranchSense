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

## Milestone 5.5 — Semantic state foundation ✅

- Canonical fact provenance and repository/workspace/revision identity
- Document-scoped fact deltas and immutable fact snapshots
- Explicit unresolved, ambiguous, external, and invalid reference states

## Milestone 6 — Repository semantic indexing ✅

- Deterministic Java source discovery with configurable ignored directories ✅
- Multi-file parser, extractor, and semantic graph indexing ✅
- Content-based unchanged reuse and document replacement/deletion ✅
- Atomic repository snapshot publication and structured diagnostics ✅

## Milestone 7 — Workspace identity and Git discovery ✅

- Repository discovery through `gix` ✅
- Read-only revision, ref, branch, and merge-base resolution ✅
- Git-tree Java snapshot indexing ✅
- Persistent snapshots and canonical workspace/session lifecycle

## Milestone 8 — Semantic graph foundation ✅

- Stable symbol/dependency graph schema ✅
- Immutable graph snapshots and document replacement ✅
- Deterministic node, edge, symbol, and ownership indexes ✅
- Incremental invalidation indexes

## Milestone 9 — Semantic change analysis ✅

- Deterministic document, fact, symbol, and relationship diffs ✅
- Conservative callable signature-change classification ✅
- Immutable snapshot comparison with stable ordering and statistics ✅

## Milestone 10 — Query and observability surface 🚧

- Versioned local query protocol
- Cross-file Java scope and classpath resolution
- CLI graph inspection and diagnostic bundles
- Performance benchmarks for warm incremental updates

## Milestone 11 — Semantic impact analysis ✅

- Deterministic impact sets derived from semantic diffs
- Bounded direct and transitive caller analysis
- Structured causal explanations and truncation reporting
- Git-backed `branchsense impact` inspection

## Milestone 12 — Branch impact analysis ✅

- Compare two branch deltas relative to a unique common merge base.
- Produce deterministic direct, impact, shared-impact, and cross-impact candidates.
- Preserve bounded causal paths and report truncation.
- Provide Git-backed `branchsense overlap` inspection.

## Milestone 13 — Semantic collision assessment ✅

- Classify semantic overlap into explicit collision evidence factors.
- Produce deterministic evidence strength and severity without probability claims.
- Preserve signature, removal, transitive, and causal-path explanations.
- Provide Git-backed `branchsense analyze` inspection.

## Milestone 14 — Historical signals ✅

- Bounded read-only Git ancestry traversal.
- Revision-pinned change frequency and recency evidence.
- Semantic symbol co-change with explicit file-level co-change fallback.
- Deterministic JSON-compatible historical signal output.
- Git-backed `branchsense history` inspection.
- Explicitly defer unreliable conflict reconstruction.

## Milestone 15 — Ownership and code responsibility signals ✅

- Bounded, read-only Git author contribution evidence.
- Separate semantic symbol and document-level responsibility scopes.
- Conservative author identity normalization and deterministic contribution shares.
- Recent contributors, top-contributor concentration, and supporting commit provenance.
- Git-backed `branchsense ownership` inspection with JSON output.
- Keep historical signals and collision assessment independent.

## Next — Evidence envelope integration

- Propagate one evidence envelope through diff, impact, overlap, collision,
  history, and responsibility results.
- Preserve overload-safe canonical identities and explicit evidence lineage.
- Add adversarial identity, truncation, privacy, and CLI integration coverage.
- Keep BCS design separate until evidence contracts are complete.

## Future milestones

- Git-backed repository and revision infrastructure is implemented locally;
  persistence and richer Git provenance remain future work.
Editor clients, collaboration replication, and BCS-C consumers are deferred
until the local semantic engine is proven correct, incremental, and observable.

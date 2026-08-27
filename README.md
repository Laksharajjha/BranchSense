# BranchSense

<p align="center">
  <img src="assets/branchsense-logo.svg" alt="BranchSense" width="720">
</p>

**Semantic intelligence for Git branches.**

BranchSense understands relationships between your code so it can reason about
changes beyond ordinary line-based Git diffs. Its long-term goal is to identify
potential semantic conflicts before a merge; today it provides the local
semantic foundation for that work.

[![CI](https://github.com/Laksharajjha/BranchSense/actions/workflows/ci.yml/badge.svg)](https://github.com/Laksharajjha/BranchSense/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/built_with-Rust-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/Laksharajjha/BranchSense?include_prereleases)](https://github.com/Laksharajjha/BranchSense/releases)

BranchSense is not an AI coding assistant, IDE extension, or merge tool. It is
an open-source semantic analysis engine that currently focuses on Java and
Git-backed repository snapshots.

## The Problem

Traditional Git tells you which lines changed. BranchSense records what those
lines mean and which declarations are connected to them.

```text
Before: PaymentService.process(User)
After:  PaymentService.process(User, Currency)

PaymentController.submit()
        │ calls
        ▼
PaymentService.process(...)
```

The second view exposes a dependency that a line diff does not explain. That
information powers semantic diffs and bounded impact analysis today, and is the
input for future branch-overlap and collision prediction.

## What Works Today

- [x] Java parsing with syntax recovery and incremental parser sessions
- [x] Language-independent semantic facts and Java extraction
- [x] Repository-wide Java indexing and immutable semantic graph snapshots
- [x] Semantic queries for callers, callees, references, implementations, and dependencies
- [x] Deterministic semantic diffs between snapshots
- [x] Read-only Git repository, revision, ref, and merge-base inspection
- [x] Git-backed semantic snapshots loaded directly from commit trees
- [x] Bounded semantic impact analysis with structured explanations
- [x] Deterministic branch overlap candidates from a common Git merge base
- [x] Deterministic semantic collision assessment with explainable evidence
- [x] Bounded historical semantic evidence and co-change analysis
- [x] Bounded historical contributor responsibility evidence
- [ ] Branch collision prediction / BCS — future work
- [ ] VS Code or other IDE integration — future work
- [ ] Collaboration server — future work

## Quick Start

Prerequisites: stable Rust with Cargo and a Java source file. The current
development workflow runs the CLI through Cargo.

```sh
git clone https://github.com/Laksharajjha/BranchSense.git
cd BranchSense
cargo run --bin branchsense -- --help
cargo run --bin branchsense -- inspect crates/branchsense-extractor-java/tests/fixtures/SpringApplication.java
```

The inspection command prints package, type, method, field, relationship, fact,
and diagnostic counts without printing the syntax tree.

See [GETTING_STARTED.md](GETTING_STARTED.md) for the complete first session.

## Five-Minute Example

Use the repository's Java fixture to exercise the current local pipeline:

```sh
# Parse one Java file and print tree statistics.
cargo run --bin branchsense -- parse crates/branchsense-java/tests/fixtures/Hello.java

# Extract semantic facts and inspect their counts.
cargo run --bin branchsense -- inspect crates/branchsense-extractor-java/tests/fixtures/SpringApplication.java

# Build a repository-wide semantic graph.
cargo run --bin branchsense -- index crates/branchsense-extractor-java/tests/fixtures

# Query a repository or a single Java file.
cargo run --bin branchsense -- callers payment.PaymentService.process --file path/to/Payment.java

# Compare two Git revisions.
cargo run --bin branchsense -- diff --repo . --before main --after feature/payment

# Analyze symbols affected by the semantic change.
cargo run --bin branchsense -- impact --repo . --before main --after feature/payment

# Compare two branches relative to their common base.
cargo run --bin branchsense -- overlap --repo . --base main \
  --branch-a feature/payment --branch-b feature/checkout

# Assess the strength of semantic collision evidence.
cargo run --bin branchsense -- analyze --repo . --base main \
  --branch-a feature/payment --branch-b feature/checkout

# Inspect bounded historical semantic evidence.
cargo run --bin branchsense -- history --repo . --revision main --max-commits 500

# Inspect historical contributor responsibility evidence.
cargo run --bin branchsense -- ownership --repo . --revision main --max-commits 500
```

Representative output from `inspect` has this shape:

```text
Package: 1
Types: 2
Methods: 3
Fields: 1
Relationships: 4
Fact Count: 12
Extraction Time: ...
Syntax Diagnostics: 0
Extraction Diagnostics: 0
```

The exact counts and timings depend on the file. The `impact` command reports
changed symbols, impacted symbols, traversal depth, truncation, and structured
causes such as `SignatureConsumer` and `TransitiveCaller`.

## How It Works

```text
Source code
    ↓
Semantic understanding
    ↓
Semantic graph
    ↓
Semantic changes
    ↓
Impact analysis
    ↓
Future branch collision detection
```

The current product boundary ends at deterministic branch overlap evidence.
Probabilistic collision prediction, IDE warnings, and collaboration workflows
are not yet implemented. The `overlap` command reports semantic evidence only;
it does not assign a risk score.

## Architecture

```text
Java source
    ↓
Parser and language adapter
    ↓
Semantic extraction
    ↓
Semantic facts
    ↓
Immutable semantic graph
    ↓
Repository index
    ↓
Semantic diff
    ↓
Git-backed snapshot
    ↓
Semantic impact analysis
    ↓
Branch overlap evidence
    ↓
Semantic collision assessment
    ↓
Historical evidence
```

- The parser produces language-neutral parsed documents.
- The Java adapter isolates Tree-sitter from the rest of the system.
- Extraction translates syntax into canonical semantic facts.
- The graph stores declarations and relationships as immutable snapshots.
- The index builds deterministic repository-wide Java snapshots.
- The diff compares semantic state rather than source lines.
- Git loads exact revisions without checking out or modifying the worktree.
- Impact analysis follows supported graph relationships with explicit bounds.
- Overlap analysis compares two branch deltas from one merge base and preserves
  direct, impact, shared-impact, and cross-impact evidence.

See the full [architecture document](ARCHITECTURE.md) for boundaries and design
rationale.

## Documentation Map

```text
[Getting Started] → [Semantic Model] → [Graph] → [Index]
                                      ↓
                         [Git] → [Diff] → [Impact]
                                      ↓
                                [Contributing]
```

- [Getting Started](GETTING_STARTED.md)
- [Parser abstraction](docs/parser.md)
- [Java adapter](docs/java-adapter.md)
- [Semantic state](docs/semantic-state.md)
- [Semantic graph](docs/semantic-graph.md)
- [Repository indexing](docs/repository-indexing.md)
- [Git integration](docs/git-integration.md)
- [Semantic diff](docs/semantic-diff.md)
- [Semantic impact analysis](docs/semantic-impact.md)
- [Branch overlap analysis](docs/branch-overlap.md)
- [Semantic collision engine](docs/collision-engine.md)
- [Historical signals](docs/historical-signals.md)
- [Architecture](ARCHITECTURE.md)
- [Roadmap](ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Security](SECURITY.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [License](LICENSE)

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo build --workspace --release
```

## Performance

The current impact benchmark measures the full analysis stage over indexed
fixtures. On one local run with Criterion's ten-sample configuration:

| Fixture | Median range |
| --- | ---: |
| 1 changed symbol | 4.46 µs |
| 10 changed symbols | 292 µs |
| 100 changed symbols | 48.3 ms |

These are development baselines, not product guarantees. Run
`cargo bench -p branchsense-impact --bench impact_analysis` on your hardware
before drawing performance conclusions.

## Roadmap

```text
Semantic understanding  ✅
        ↓
Git understanding       ✅
        ↓
Impact analysis         ✅
        ↓
Branch overlap          NEXT
        ↓
Collision prediction    FUTURE
        ↓
IDE warning             FUTURE
```

The detailed status and sequencing are maintained in [ROADMAP.md](ROADMAP.md).

## License

BranchSense is released under the [Apache License 2.0](LICENSE).

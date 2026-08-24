# Getting Started

This guide is for developers who know basic Git and Java and want to see what
BranchSense does today.

## What Is BranchSense?

BranchSense is a local semantic intelligence engine for Git repositories. It
parses Java, extracts declarations and relationships, builds immutable semantic
graphs, compares repository snapshots, and reports code that may be affected by
a semantic change.

It is not a merge tool and does not yet predict branch collisions.

## What Problem Does It Solve?

Git can show that a method signature changed. BranchSense can additionally
identify callers, references, implementations, and bounded transitive callers
that may need review.

```text
PaymentService.process(User)
        becomes
PaymentService.process(User, Currency)

PaymentController.submit() → PaymentService.process(...)
```

## Prerequisites

- Rust 1.85 or newer with Cargo
- Git for repository-backed commands
- Java source files for analysis

No Java compiler, database, server, IDE extension, or AI service is required.

## Build BranchSense

```sh
git clone https://github.com/Laksharajjha/BranchSense.git
cd BranchSense
cargo build --workspace
```

During this alpha, invoke the binary through Cargo:

```sh
cargo run --bin branchsense -- --help
```

## Run Your First Analysis

Parse the small Java fixture:

```sh
cargo run --bin branchsense -- parse crates/branchsense-java/tests/fixtures/Hello.java
```

The command reports language, total syntax-tree nodes, tree depth, parse
duration, and syntax diagnostics.

Extract semantic facts from a Java file:

```sh
cargo run --bin branchsense -- inspect crates/branchsense-extractor-java/tests/fixtures/SpringApplication.java
```

The output reports package, type, method, field, relationship, and fact counts.

## Index a Repository

Index Java sources below a path:

```sh
cargo run --bin branchsense -- index path/to/java-repository
```

The report includes discovered, indexed, unchanged, and skipped files, parse
and extraction diagnostics, graph node and edge counts, and duration.

## Try Semantic Queries

Query callers from one file:

```sh
cargo run --bin branchsense -- callers payment.PaymentService.process \
  --file path/to/PaymentService.java
```

Query an indexed project instead:

```sh
cargo run --bin branchsense -- callers payment.PaymentService.process \
  --project path/to/java-repository
```

Related commands are `callees`, `references`, `implementations`, and
`dependencies`. Symbols must use the fully qualified names available in the
semantic graph.

## Try Semantic Diff

Compare two Git revisions without checking either one out:

```sh
cargo run --bin branchsense -- diff \
  --repo path/to/repository \
  --before main \
  --after feature/payment
```

The report summarizes changed documents, symbols, facts, and relationships.

## Try Impact Analysis

Run bounded impact analysis over the same two revisions:

```sh
cargo run --bin branchsense -- impact \
  --repo path/to/repository \
  --before main \
  --after feature/payment
```

The report contains changed symbols, impacted symbols, maximum traversal depth,
truncation status, and causal impact kinds such as `DirectCaller`,
`SignatureConsumer`, and `TransitiveCaller`.

## Inspect Git State

BranchSense can inspect repository metadata without mutating the worktree:

```sh
cargo run --bin branchsense -- git info path/to/repository
cargo run --bin branchsense -- git branches path/to/repository
cargo run --bin branchsense -- git merge-base main feature/payment \
  --path path/to/repository
```

## Understand the Architecture

```text
Java source → parser → semantic facts → graph → index
                                      ↓
                              semantic diff
                                      ↓
                              impact analysis
```

Read the [architecture overview](ARCHITECTURE.md), then the detailed guides
for [Git integration](docs/git-integration.md),
[semantic diff](docs/semantic-diff.md), and
[semantic impact](docs/semantic-impact.md).

## Current Limitations

- Java is the only implemented language adapter.
- Classpath-wide type resolution is incomplete.
- Reflection, generated code, and data-flow effects are not modeled.
- Snapshots are in-memory and are not persisted.
- Branch overlap, collision prediction, BCS, IDE integration, and collaboration
  services are not implemented.

## What's Next

The next milestone is branch impact analysis: comparing the impact sets of two
branches to produce semantic overlap candidates. See [ROADMAP.md](ROADMAP.md).

## Where to Go Next

- [Semantic model](docs/semantic-state.md)
- [Semantic graph](docs/semantic-graph.md)
- [Repository indexing](docs/repository-indexing.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

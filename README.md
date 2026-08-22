# BranchSense

BranchSense is an open-source semantic intelligence engine for Git repositories.
It maintains a live model of symbols, dependencies, and project structure so
future tools can identify integration risks before they become merge conflicts.

It is not an AI coding assistant, editor extension, or merge tool.

## Status

The project is in an early alpha. The current milestone provides the semantic
domain model, parser and language-adapter contracts, the first Tree-sitter-backed
Java parser, canonical semantic facts, Java semantic extraction, and the first
semantic graph snapshots, repository indexing, semantic queries, and
deterministic snapshot diffing. Git inspection and conflict prediction remain
future work.

## Quick start

Install stable Rust using [rustup](https://rustup.rs), then run:

```sh
cargo run --bin branchsense -- --help
cargo run --bin branchsense -- version
cargo run --bin branchsense -- parse crates/branchsense-java/tests/fixtures/Hello.java
cargo run --bin branchsense -- inspect crates/branchsense-extractor-java/tests/fixtures/SpringApplication.java
```

## Workspace

| Crate | Responsibility |
| --- | --- |
| `branchsense-core` | Immutable semantic entities, strongly typed identifiers, value objects, relationships, and domain aggregates. |
| `branchsense-parser` | Language-neutral parser traits, parsed-document abstraction, incremental edit contract, diagnostics, configuration, and registry. |
| `branchsense-language` | Language adapter metadata, capabilities, lifecycle sessions, compatibility checks, negotiation, and registry. |
| `branchsense-java` | Tree-sitter-backed Java parser, recovery diagnostics, incremental parsing, and tree statistics. |
| `branchsense-semantic` | Language-independent semantic facts, symbol definitions, references, relationships, and fact batches. |
| `branchsense-extractor-java` | Java syntax-to-semantic-fact extraction with recovery diagnostics and benchmarks. |
| `branchsense-graph` | Immutable semantic graph snapshots, typed relationships, indexes, and document updates. |
| `branchsense-query` | Read-only typed semantic queries over graph snapshots. |
| `branchsense-index` | Deterministic Java discovery and repository-wide graph indexing. |
| `branchsense-diff` | Deterministic comparison of semantic index snapshots. |
| `branchsense` | The CLI binary, argument parsing, command dispatch, and process logging. |

The semantic model is organized into explicit domain boundaries:

```text
crates/branchsense-core/src/
├── domain/          aggregate-level snapshots and revision-pinned models
├── entities/        workspace, project, document, declaration, and dependency entities
├── errors/          typed validation errors
├── ids/             strongly typed identifiers
├── relationships/   typed semantic relationships
├── traits/          small domain capabilities such as identity and location
└── value_objects/   immutable positions, ranges, names, visibility, and language values
```

The repository indexing milestone adds deterministic multi-file Java
discovery, coherent graph snapshots, incremental document reuse, and project
queries. The semantic-diff milestone compares those snapshots without Git. Git
intelligence, conflict prediction, and AI remain future work.

The root manifest centrally pins the selected ecosystem: `tree-sitter` for
incremental parsing, `gix` (gitoxide) for Git access, `petgraph` as the initial
graph backend, `tokio` for runtime scheduling, and `serde`, `tracing`, and
`clap` for shared infrastructure.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

`RUST_LOG` controls structured diagnostic logging. Logs are written to stderr;
command output is written to stdout.

## Design documents

- [Architecture](ARCHITECTURE.md)
- [Branch Collision Score draft](BRANCH_COLLISION_SCORE.md)
- [BCS-C research revision](BCS_REVIEW_AND_REVISION.md)
- [Error handling strategy](docs/error-handling.md)
- [Parser abstraction](docs/parser.md)
- [Language adapter framework](docs/language-adapter.md)
- [Java semantic extractor](docs/java-extractor.md)
- [Semantic state foundation](docs/semantic-state.md)
- [Semantic graph](docs/semantic-graph.md)
- [Semantic queries](docs/semantic-queries.md)
- [Repository indexing](docs/repository-indexing.md)
- [Semantic diff](docs/semantic-diff.md)
- [Roadmap](ROADMAP.md)
- [Contributing guide](CONTRIBUTING.md)

## License

BranchSense is released under the [Apache License 2.0](LICENSE).

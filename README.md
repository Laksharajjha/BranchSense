# BranchSense

BranchSense is an open-source semantic intelligence engine for Git repositories.
It maintains a live model of symbols, dependencies, and project structure so
future tools can identify integration risks before they become merge conflicts.

It is not an AI coding assistant, editor extension, or merge tool.

## Status

The project is in Milestone 1: a production-oriented Rust workspace, CLI
foundation, quality gates, and public project governance. Parsing, Git
inspection, graph maintenance, and prediction are deliberately not implemented
in this milestone.

## Quick start

Install stable Rust using [rustup](https://rustup.rs), then run:

```sh
cargo run --bin branchsense -- --help
cargo run --bin branchsense -- version
```

## Workspace

| Crate | Responsibility |
| --- | --- |
| `branchsense-core` | Immutable semantic entities, strongly typed identifiers, value objects, relationships, and domain aggregates. |
| `branchsense-parser` | Language-neutral parser traits, parsed-document abstraction, incremental edit contract, diagnostics, configuration, and registry. |
| `branchsense-language` | Language adapter metadata, capabilities, lifecycle sessions, compatibility checks, negotiation, and registry. |
| `branchsense-java` | Tree-sitter-backed Java parser, recovery diagnostics, incremental parsing, and tree statistics. |
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

This milestone intentionally stops at the semantic model. It does not parse
source, inspect Git, mutate a graph, expose networking, or integrate an editor.

The root manifest centrally pins the selected ecosystem: `tree-sitter` for
incremental parsing, `gix` (gitoxide) for Git access, `petgraph` as the initial
graph backend, `tokio` for runtime scheduling, and `serde`, `tracing`, and
`clap` for shared infrastructure. Parser, Git, and graph dependencies remain
unlinked until their implementation milestones; this keeps Milestone 1 small
and avoids shipping unused behavior.

## Development

```sh
cargo fmt --check
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
- [Roadmap](ROADMAP.md)
- [Contributing guide](CONTRIBUTING.md)

## License

BranchSense is released under the [MIT License](LICENSE).

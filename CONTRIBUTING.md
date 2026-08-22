# Contributing to BranchSense

Thank you for contributing to BranchSense. The project favors small,
well-tested changes that preserve clear module boundaries and deterministic
behavior.

## Before opening a pull request

1. Read `ARCHITECTURE.md` and keep the kernel independent of adapters.
2. Keep public types and error contracts documented.
3. Run the quality checks from the repository root:

   ```sh
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
   cargo build --workspace --release
   cargo bench -p branchsense-java --bench java_parse --no-run
   cargo bench -p branchsense-extractor-java --bench java_extract --no-run
   cargo bench -p branchsense-graph --bench graph_repository --no-run
   cargo bench -p branchsense-query --bench semantic_queries --no-run
   cargo bench -p branchsense-index --bench repository_index --no-run
   ```

BranchSense targets Rust `1.85.0` or newer. Pull requests also run an MSRV
check, a `cargo-audit` vulnerability scan, cargo-deny advisory/license/source
policy checks, and GitHub dependency review. These checks require no local
secrets; install `cargo-audit` and `cargo-deny` locally if you want to run the
security checks before opening a pull request.

4. Explain the user-visible behavior, architectural impact, and validation in
   the pull request description.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) so the history
and release notes remain easy to scan. Prefer a focused commit with a type and
scope, for example:

```text
feat(java): add Tree-sitter Java parser adapter
fix(cli): report recovered Java syntax errors
docs: document the alpha release
```

Keep implementation, tests, and documentation for one change together when
that makes the commit easier to review. Avoid mixing unrelated refactors into
feature commits.

## Release checklist

Release candidates must have a clean working tree, passing CI, updated
`CHANGELOG.md`, verified package metadata and license, and a signed or
annotated version tag created by a maintainer. Alpha releases should state
their limitations explicitly and must not imply API stability.

## Rust conventions

- Use stable Rust only and honor the workspace minimum Rust version.
- Prefer traits at package boundaries and concrete types within a package.
- Keep crates dependency-directed; adapter crates must not leak into core.
- Return typed errors at library boundaries. The CLI may translate them into
  concise diagnostics for users.
- Do not introduce `unsafe` code. The workspace forbids it.
- Keep logging structured with `tracing`; do not print diagnostics to stdout.

## Scope discipline

Do not add parsing, Git inspection, graph mutation, editor integration, or
conflict prediction ahead of their roadmap milestones. A focused foundation is
more valuable than speculative interfaces.

## Reporting issues

Report reproducible defects with the BranchSense version, operating system,
Rust version, command, expected behavior, actual behavior, and a minimal
repository fixture when possible. Do not include private source code or
credentials.

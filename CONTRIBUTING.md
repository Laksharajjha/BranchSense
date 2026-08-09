# Contributing to BranchSense

Thank you for contributing to BranchSense. The project favors small,
well-tested changes that preserve clear module boundaries and deterministic
behavior.

## Before opening a pull request

1. Read `ARCHITECTURE.md` and keep the kernel independent of adapters.
2. Keep public types and error contracts documented.
3. Run the quality checks from the repository root:

   ```sh
   cargo fmt --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

4. Explain the user-visible behavior, architectural impact, and validation in
   the pull request description.

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

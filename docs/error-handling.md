# Error Handling Strategy

BranchSense uses typed, contextual errors at every public boundary.

## Libraries

Library crates define their own error enum with `thiserror` and expose a crate
`Result<T>` alias. Error variants describe actionable failure categories and
retain structured context such as paths, identifiers, and revisions. Libraries
do not print errors, terminate the process, or depend on a global logger.

Cross-crate error conversion is explicit. A boundary should translate an error
only when it can add useful context or hide an implementation detail; otherwise
the original typed error should be preserved.

## Binary

The CLI returns a typed error from `main`. It writes successful command output
to stdout and diagnostics to stderr through `tracing`. This keeps output safe
for scripts and makes diagnostic filtering controllable with `RUST_LOG`.

## Invariants

- Use `Result` for recoverable failures.
- Reserve panics for violated internal invariants that represent programming
  defects, never for invalid user input or repository state.
- Include a source error when wrapping an I/O, parser, Git, or transport
  failure.
- Do not leak source contents, credentials, or private paths into telemetry.
- Keep error variants stable when they cross a public protocol boundary.

This policy supports deterministic local behavior today and preserves the
context required for future editor and collaboration adapters.

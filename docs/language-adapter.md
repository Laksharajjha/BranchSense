# Language Adapter Framework

`branchsense-language` is the host boundary for language-specific parser
implementations. It depends on `branchsense-parser` but does not depend on a
parser generator or a language grammar.

## Registration

`AdapterRegistry` is an instance-owned, cloneable registry backed by a
thread-safe read/write map. Registration validates the adapter's declared
framework API range and rejects duplicate languages. There is no global
registry, static initialization, or singleton state.

```text
registry.register(adapter)
registry.adapter(Language::Java)
```

The returned adapter publishes its `AdapterMetadata`, `Capabilities`, and
implementation `Version`. A later host can call `start` to create an
`AdapterSession`, whose parser is the generic `branchsense-parser` interface.

## Capabilities

Capabilities are a compact typed bitset covering incremental parsing, semantic
extraction, type and symbol resolution, cross-file analysis, dependency
analysis, formatting, and diagnostics. `FeatureRequest` separates required
features from preferred features. Negotiation fails only when a required
capability is unavailable; preferred gaps remain observable through
`NegotiatedFeatures`.

## Lifecycle and compatibility

Adapters publish a semantic implementation version and a framework API
`VersionRange`. The registry validates compatibility before storing an adapter.
Sessions are explicitly started and shut down through `AdapterSession`; parser
resources are never managed by global state.

No Java, Kotlin, Rust, or other concrete adapter is included in this milestone.

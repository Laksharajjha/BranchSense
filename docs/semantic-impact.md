# Semantic Impact Analysis

Semantic diff answers **what changed**. Semantic impact analysis answers
**what may be affected by those changes**. The `branchsense-impact` crate
connects the two without parsing source, invoking Git, or mutating snapshots.

## Model

`ImpactAnalyzer` consumes a `SemanticDiff`, the before snapshot, and the after
snapshot. It returns an immutable `ImpactSet` containing one `ImpactEntry` per
impacted declaration. Each entry contains one or more `ImpactCause` values so
multiple changed symbols retain their independent causal paths.

Every cause records the changed symbol, impact kind, traversal depth, graph
relationship, relationship fact identity, and an ordered `ImpactPath`.

## Direction and bounds

Call edges are traversed backwards: an edge `caller → callee` makes the caller
an impact of a changed callee. Direct callers are depth one; callers reached
through another call edge are transitive callers. References, implementations,
subtypes, and explicit dependencies are reported as direct impacts and are not
followed transitively.

Analysis is bounded by `ImpactOptions::max_depth` and `max_results`. The result
reports truncation rather than silently presenting an incomplete result.
Results and causes are sorted by stable symbol, relationship, depth, and path
identity, so repeated analysis of the same snapshots is identical.

## Signature changes

Signature, return-type, and parameter changes classify direct call impacts as
`SignatureConsumer`. Removed declarations are analyzed against the before
graph; added and modified declarations use the after graph. Newly added
symbols without incoming evidence do not create fabricated impacts.

Java extraction may emit unresolved call names when overload or receiver
resolution is unavailable. Impact analysis conservatively matches an unresolved
method name to a declaration only when its qualified declaration prefix is
unambiguous in the selected graph. It does not infer arbitrary textual links.

## CLI

The current Git-backed inspection command is:

```sh
branchsense impact --repo . --before main --after feature/payment
```

It builds two in-memory Git semantic snapshots, computes a semantic diff, and
prints structured impact summaries. Snapshot persistence and branch-overlap
analysis are intentionally outside this milestone.

## Limitations

Impact analysis currently relies on the relationships emitted by the semantic
graph. Classpath-wide resolution, data-flow effects, reflection, generated
code, and branch-to-branch overlap are not modeled. The next milestone will
compare impact sets from two branches; it will not be part of this crate.

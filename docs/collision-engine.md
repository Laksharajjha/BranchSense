# Semantic collision engine

The collision engine is the first deterministic assessment layer above branch
overlap. It answers a narrower question than Git merge prediction:

> How strong is the available semantic evidence that two branch changes may
> interfere during integration?

It does not claim that a merge will fail and it does not estimate a
probability. Its `evidence_score` is an ordinal value from 0 to 100 used to
compare evidence strength under explicit rules.

## Pipeline

```text
Git revisions
  → semantic snapshots
  → SemanticDiff
  → ImpactSet
  → OverlapSet
  → CollisionAssessment
```

`branchsense-collision` consumes only `OverlapSet`. It does not parse source,
read Git history, traverse graphs, contact a service, or use AI. This keeps the
assessment reusable by the CLI and future editor or server consumers.

## Factors and rules

The analyzer retains every applicable factor but scores each unique overlap
pair once. The strongest factor for that pair supplies its contribution.
Redundant representations therefore explain the result without inflating it.

| Factor | Strength | Rule |
| --- | ---: | --- |
| `SameSymbolChanged` | 80 | Both branches modify one stable symbol. |
| `ChangedSymbolImpact` | 65 / 45 / 30 | Branch A changes a symbol reached by branch B at depth 1 / 2 / 3+. |
| `ReverseChangedSymbolImpact` | 65 / 45 / 30 | The reverse directional relationship. |
| `SharedImpact` | 47 / 37 / 30 | Both branches affect the same downstream target at depth 1 / 2 / 3+. |
| `TransitiveImpact` | 30 | The retained causal path is deeper than one relationship. |
| `SignatureInteraction` | 85 | Impact evidence identifies a signature consumer. |
| `RemovalInteraction` | 90 | A changed symbol is removed and the other branch has dependency evidence. |

These values are evidence strengths, not probabilities. They are intentionally
small, explicit rules that can be evaluated and changed as empirical evidence
arrives. Historical and responsibility evidence are available as independent
inputs for future BCS work; this crate does not combine them.

## Severity

Severity is derived from the capped evidence score:

| Score | Severity |
| ---: | --- |
| 0 | `None` |
| 1–29 | `Informational` |
| 30–59 | `Low` |
| 60–79 | `Medium` |
| 80–100 | `High` |

There is no `Critical` level yet. The current semantic evidence cannot justify
that stronger claim. A high assessment means strong semantic interaction, not a
guaranteed conflict.

## Specialized interactions

Signature interactions are detected from the existing impact classification
`SignatureConsumer`; the collision engine does not infer signatures from text.
Removal interactions use declaration change metadata preserved by the overlap
layer and require dependency evidence from the impact layer.

Transitive evidence retains the original `ImpactPath` values. A direct shared
symbol therefore scores more strongly than a distant path, while the path is
still available for inspection.

## Explanations and output

`CollisionAssessment` is serializable with the workspace's existing serde
conventions. It contains severity, evidence score, factors, structured causal
evidence, deterministic summaries, and truncation statistics.

```sh
branchsense analyze --repo . --base main \
  --branch-a feature/payment --branch-b feature/checkout
```

The command prints branch revisions, changed-symbol counts, overlap count,
severity, score, factors, symbols, targets, and impact depths. It explicitly
labels the result as semantic evidence rather than merge-failure probability.

## Benchmark baseline

The release-profile Criterion benchmark was run locally with 100 samples on
the synthetic serialized fixtures:

| Case | Median |
| --- | ---: |
| Empty assessment | 29.8 ns |
| 1 direct impact | 1.18 µs |
| 10 direct impacts | 10.3 µs |
| 100 direct impacts | 118 µs |
| 10 deep impacts | 18.0 µs |

These are development baselines for the assessment stage only, not product
performance guarantees. They exclude Git, parsing, indexing, extraction, and
impact analysis.

## Limitations and next step

The engine is conservative and only sees semantic facts emitted by the current
Java pipeline. Unresolved references, incomplete extraction, missing classpath
resolution, and bounded impact traversal can reduce recall; the assessment
reports upstream truncation rather than hiding it. There is no calibration
against actual merge outcomes yet.

The next major milestone is historical and ownership evidence: prior conflict
frequency, co-change patterns, ownership concentration, and symbol hotness.
Those signals should be validated independently before designing the final BCS
model. IDE integration should follow that research, not replace it.

# Branch overlap analysis

Branch overlap analysis compares two independent branch changes relative to
one common merge base. It is the first BranchSense feature that reasons about
two branches at once, but it deliberately stops short of risk scoring or
conflict prediction.

## Pipeline

```text
Git revisions A and B
        ↓
Unique common merge base
        ↓
Semantic snapshots for base, A, and B
        ↓
SemanticDiff(base, A) and SemanticDiff(base, B)
        ↓
Bounded ImpactSet A and ImpactSet B
        ↓
SemanticOverlapAnalyzer
        ↓
OverlapSet with structured evidence
```

The library does not invoke Git, parse source, or traverse graphs. Those
responsibilities remain in the existing Git, index, diff, and impact crates.
This makes the analyzer usable with persisted snapshots and future language
adapters without changing its API.

## Overlap kinds

- `DirectChange`: both branches changed the same stable symbol.
- `ImpactChange`: one branch changed a symbol reached by the other branch's
  impact set.
- `SharedImpact`: both branches impact the same downstream symbol.
- `CrossImpact`: each branch's changed symbol impacts the other branch's
  changed symbol.

Every entry preserves branch A and B changed symbols, one or more targets, the
impact classification, relationship, depth, graph fact identity when
available, and the complete `ImpactPath`. Distinct causal paths are retained;
duplicate evidence is removed deterministically.

## Bounds and semantics

`SemanticOverlapAnalyzer` accepts `OverlapOptions` with a maximum evidence
depth and result count. Upstream impact truncation is carried into the result,
so consumers cannot mistake a bounded result for proof that no other overlap
exists. Results are sorted by overlap kind and strongly typed symbol identity.

The CLI requires `--base` to resolve to the unique Git merge base of
`--branch-a` and `--branch-b`. Missing, unrelated, or ambiguous merge bases
are reported as errors. Branch-to-branch comparison is intentionally not used:
both deltas must be measured from the same base.

```sh
branchsense overlap --repo . --base main \
  --branch-a feature/payment --branch-b feature/checkout
```

## Not a risk score

Overlap is observable semantic evidence. It does not estimate probability,
rank developers, or claim that a Git merge will conflict. Future collision
prediction work may consume this evidence, historical merge outcomes, and
explicit policy while leaving this deterministic vocabulary stable.

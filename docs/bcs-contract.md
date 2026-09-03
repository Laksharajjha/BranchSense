# BCS Contract

Branch Collision Score (BCS) is a future BranchSense consumer of semantic
evidence. This document defines the contract before scoring is implemented.

## Version-one target

BCS v1 answers:

> How strongly does the available semantic evidence indicate that two
> concurrent branch evolutions deserve integration review?

The first result is deterministic, explainable, and ordinal. Its conceptual
bands are `None`, `Low`, `Moderate`, `High`, `Critical`, and `Indeterminate`.
`Indeterminate` is not a risk band: it means that trustworthy evidence was not
available for a normal assessment.

BCS v1 is not a probability of a Git conflict, build failure, runtime defect,
or merge success. `CollisionAssessment::evidence_score()` is ordinal collision
evidence strength and must never be serialized or described as probability,
percentage, confidence, or likelihood. `HistoricalSignals` and
`ResponsibilitySignals` are contextual observations, not calibrated weights or
causal explanations.

## Evidence identity

An underlying observation is identified independently from its evidence role.
`EvidenceKind` describes whether an observation is primary, supporting, or
derived. `EvidenceRelation` describes `Supports`, `DerivedFrom`, and
`Corroborates`. Derived observations belong to the same causal family for
deduplication. Corroborating observations remain separate and explainable.

The `EvidenceLedger` provides deterministic identity and lineage deduplication.
BCS must not count a diff, impact, overlap, and collision assessment as four
independent observations when lineage shows they are one causal chain.

## Abstention

`NoEvidence` means the requested analysis completed and found no applicable
evidence; it does not mean safe. `Unavailable`, `Unsupported`, `Unresolved`,
`Ambiguous`, `Truncated`, and `Failed` remain visible and are never converted
to zero evidence.

BCS may continue with warnings when a non-critical source document is
incomplete and is outside the analyzed changed subgraph. It must return
`Indeterminate` when a required branch, merge base, collision analysis, or
central changed symbol cannot be analyzed reliably. Ambiguous and unresolved
references require abstention when they affect the primary interaction path.

Every result must expose evidence state, completeness, warnings, provenance,
and the reason for abstention or reduced confidence.

## Probability boundary

A future calibrated layer may consume the deterministic evidence
representation:

```text
semantic evidence → deterministic ordinal assessment → optional calibration → probability
```

Calibration is a separate, versioned contract. A raw ordinal score must never
be exposed as a probability field.


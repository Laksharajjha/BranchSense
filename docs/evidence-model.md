# Evidence model

BranchSense separates observations from conclusions. A parser, graph, Git
history walk, or attribution pass first produces a raw observation. The
analysis layer retains that observation as evidence with an explicit
availability state and provenance.

```text
raw observation
    ↓
evidence state and payload
    ↓
EvidenceIdentity
    ↓
primary / supporting / derived relationship
    ↓
future aggregation
```

## Availability

`branchsense-semantic::EvidenceState` distinguishes `Observed`, `NoEvidence`,
`Unavailable`, `Unsupported`, `Unresolved`, `Ambiguous`, `Truncated`, and
`Failed`. `NoEvidence` means the requested analysis completed and found
nothing. It must not be substituted for unavailable or unsupported analysis,
and neither state implies low collision risk.

`EvidenceCompleteness` records the state of semantic, historical, and
responsibility domains independently. This lets a future consumer identify
partial analysis without interpreting absent records as negative evidence.

## Identity and provenance

`SemanticEntityIdentity` correlates declarations conservatively across
revisions using repository-relative document path, symbol kind, and a
signature-independent qualified name. Opaque `SymbolId` values remain local to
one revision. `AnalysisProvenance` records repository and revision context,
branch merge-base context, configuration, bounded history windows, and producer
versions without depending on Git implementation types.

`EvidenceIdentity` identifies the underlying causal subject and related
entities. Related values are sorted and deduplicated for deterministic
cross-subsystem comparison.

## Evidence relationships

- **Primary** evidence is directly observed by an analysis pass.
- **Supporting** evidence explains or strengthens a primary observation.
- **Derived** evidence is deterministically produced from other evidence.

For example, a changed method is primary evidence; a direct caller path is
supporting evidence; a branch overlap derived from those paths is derived
evidence. Future aggregation must preserve these relationships so one causal
fact is not counted as several independent observations.

## Scope

This document defines contracts for future aggregation only. BCS does not yet
exist in the repository. No score, probability, calibration, or BCS CLI is
implemented here.

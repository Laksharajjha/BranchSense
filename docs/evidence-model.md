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
revisions using repository-relative document path, symbol kind, and the
adapter-supplied qualified name, including overload signatures. Opaque
`SymbolId` values remain local to one revision. `AnalysisProvenance` records repository and revision context,
branch merge-base context, configuration, bounded history windows, and producer
versions without depending on Git implementation types.

`EvidenceIdentity` identifies the underlying causal subject and related
entities. The shared `EvidenceLedger` deduplicates identical identities and
exact lineage links with deterministic ordering. It never merges independent
observations merely because they name the same subject.

## Evidence relationships

- **Primary** evidence is directly observed by an analysis pass.
- **Supporting** evidence explains or strengthens a primary observation.
- **Derived** evidence is deterministically produced from other evidence.

For example, a changed method is primary evidence; a direct caller path is
supporting evidence; a branch overlap derived from those paths is derived
evidence; an independent historical co-change can corroborate the method.
Future aggregation must preserve these relationships so one causal fact is not
counted as several independent observations while independent observations
remain visible.

The link direction is `from` observation to `to` source observation. Thus a
derived impact links to its changed-symbol evidence with `DerivedFrom`, while
historical evidence links to a semantic observation with `Corroborates`.

## Scope

This document defines contracts for future aggregation only. BCS does not yet
exist in the repository. No score, probability, calibration, or BCS CLI is
implemented here.

## Analytical result envelope

`EvidenceEnvelope` is the result-level contract used by semantic diffs,
impact sets, branch overlap, collision assessments, historical signals, and
responsibility signals. It carries result state, domain completeness, analysis
provenance, stable evidence identities, and explicit lineage links.
`EvidenceLink` distinguishes derived evidence from its source without claiming
that two observations are the same observation.

Semantic identities retain declaration signatures when the adapter provides
them, so overloaded declarations are not collapsed merely because their
display names match. Identity remains conservative: renames, moves, package
changes, and unresolved or ambiguous references do not acquire continuity by
guesswork.

# Branch Collision Score (BCS)

## 1. What BCS Is
BCS is a deterministic, ordinal, explainable scalar that classifies the severity of semantic collisions and shared responsibility between two branches. It aggregates evidence from multiple analytical dimensions and places it into explicit bands (None, Low, Moderate, High, Critical) using a stable, versioned policy.

## 2. What BCS Is Not
BCS is **not** a probability, confidence, or likelihood score. It does not predict merge outcomes or use machine learning. It is a strictly deterministic aggregation of verifiable facts.

## 3. Evidence Sources
- **Direct Collision:** Semantic mutations impacting the same signatures.
- **Shared Impact:** Transitive impacts landing on identical system boundaries.
- **Structural Overlap:** Basic AST and file-level overlaps.
- **Historical Co-change:** Evidence that the involved components are frequently modified together.
- **Responsibility Concentration:** Evidence of shared authorship profiles.

## 4. Normalization
Each subsystem's raw analysis (e.g., `CollisionAssessment`, `ImpactSet`) is adapted into a `BcsNormalizedEvidence` struct. This isolation guarantees the scoring engine remains decoupled from subsystem implementation details while retaining strict provenance and identities.

## 5. Evidence Ledger & Deduplication
Raw evidence often overlaps (e.g., a direct collision is also structurally an overlap). The `EvidenceLedger` uses `ObservationIdentity` to ensure duplicate evidence facts are squashed correctly based on their maximum observed strength, eliminating double-counting risks.

## 6. Abstention Gate
Before scoring, BCS enforces an abstention gate via `AbstentionDecision`:
- Clean evidence yields `Proceed`.
- Truncated evidence yields `Warn` (analysis proceeds but is flagged).
- Unavailable, Unsupported, Failed, Ambiguous, or Unresolved evidence yields an `Indeterminate` assessment.

## 7. BcsPolicyV1 & Scoring Rules
The policy isolates scoring math. Factors map distinct evidence into numerical contributions.
- Collision: * 5 multiplier
- Shared Impact: * 2 multiplier
- History / Ownership / Overlap: * 1 multiplier (baseline signal)
The total bounds at a defined cap (100).

## 8. Ordinal Bands
Thresholds are structurally hardcoded for V1:
- `None` (0)
- `Low` (>= 10)
- `Moderate` (>= 30)
- `High` (>= 60)
- `Critical` (>= 90)
- `Indeterminate` (Fallback for unsafe evidence limits)

## 9. Explanation Model
The `BcsExplanation` provides human-readable traces enumerating exactly which factors fired and why the final band was chosen.

## 10. Determinism Guarantees
All collections in BCS are strictly ordered (`BTreeMap`, `BTreeSet`, `Vec`). A given snapshot state will identically yield the same exact score every single time.

## 11. Current Limitations & Calibration
V1 thresholds (10, 30, 60, 90) are design-based heuristics, not empirically derived rules. The system is designed for deterministic stability first. Calibration against large datasets (using `EvalRecord`) is the focus of the subsequent evaluation milestone.

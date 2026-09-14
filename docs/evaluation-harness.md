# Evaluation Harness

The Evaluation Harness (`branchsense-evaluation`) securely executes the BCS pipeline against massive arrays of historical Git branches to measure descriptive agreement with real-world outcomes.

## 1. Dataset Format
Datasets are strictly encoded as JSONL. Each line maps perfectly to an `EvaluationCase`. This supports massive stream-processing without catastrophic full-file parse failures.

## 2. Evaluation Case Structure
Each case requires a unique `case_id`, `repository` context, `base_revision`, the two branch tips (`branch_a_revision`, `branch_b_revision`), and the three-way `merge_base`. It securely carries an `observed_outcome`.

## 3. Ground-Truth Separation
The `BcsEngine` absolutely never sees the `observed_outcome`. Outcomes are segregated inside the harness loop. The engine is only fed raw Git topologies.

## 4. Dataset Validation
Before executing, the loader guarantees:
- Case IDs are strictly unique.
- Branch refs are non-empty.
- Schema versions are compatible.

## 5. Runner Architecture
The `DatasetRunner` orchestrates the extraction of semantic data through the `SemanticIndexer` and `branchsense_diff` purely in-memory. It wraps the execution to capture abstentions, panics, or unavailable repositories safely without mutating the disk.

## 6. Determinism
Execution is strictly single-threaded (or deterministically batched). Order of JSONL dictates execution order. BTreeMap ensures identical serialization behavior.

## 7. Read-only Git Behavior
The harness explicitly disables checkout, staging, or mutation. It strictly resolves content-addressed hashes.

## 8. Abstention Handling
If history is truncated or evidence is ambiguous, the engine outputs `Indeterminate`. The harness records this as `EvaluationDiagnostic::Abstained` rather than incorrectly punishing the model for a missing outcome.

## 9. Evaluation Metrics
Metrics are restricted to: Band Distribution, Diagnostic distributions, and boolean Agreement tables (e.g., did `Critical` correlate to `semantic_integration_issue`?).

## 10. Limitations
The evaluation assumes the provided ground-truth dataset is 100% correct.

## 11. Why This Is NOT Calibration
This harness merely *measures* the distribution. It does not train, adjust, or statistically fit the `BcsPolicyV1` weights based on the outcome.

## 12. Future Calibration
A completely separate ML or statistical script could read the JSON output of this harness to run regressions, proposing new fixed heuristics for `BcsPolicyV2`.

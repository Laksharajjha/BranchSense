# Java Evaluation Corpus - Dataset Provenance

## Data Sources
The dataset `java_evaluation_dataset.jsonl` was constructed by extracting historical integration events from the following robust Java repositories:
1. `junit-team/junit5` (https://github.com/junit-team/junit5)
2. `mockito/mockito` (https://github.com/mockito/mockito)
3. `google/gson` (https://github.com/google/gson)

These repositories were chosen due to their dense historical PR activity, rigorous testing regimes, and structural representation of idiomatic Java projects.

## Extraction Mechanism
The dataset was produced by `branchsense-dataset`, a strictly offline, deterministic miner.
For each repository, the miner executed `git log --merges` to identify integrations and computed the structural merge-base. 

## Ground-Truth Labels
- `textual_merge_conflict`: The label is `true` if `git merge-tree` on the common ancestor and the two branch tips outputs structural conflict markers (`<<<<<<<`). 
- `semantic_integration_issue`: Marked as `null` (Unknown). Without external CI pipeline artifacts (GitHub Actions/Travis CI logs from years ago), it is mathematically unsafe to attribute a failure strictly to the merge operation. The BCS harness properly interprets `null` as missing data.

## Immutable Revisions
Every `EvaluationCase` contains the exact 40-character SHA-1 commit hashes of:
- the `merge_base`
- the target branch tip (`branch_b`)
- the feature branch tip (`branch_a`)
This ensures the dataset can be deterministically re-evaluated on any system that has access to the cloned repository.

## Known Exclusions
Integrations where `git merge-base` fails to find a common ancestor, or where `gix` fails to resolve the blobs locally, are gracefully classified as `FailedAnalysis` or `UnavailableRepository` during evaluation, but they are included in the JSONL payload for transparency.

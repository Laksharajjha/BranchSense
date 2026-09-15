# Dataset Generation Methodology

## 1. Candidate Selection
Candidates are selected deterministically by scanning the target repository for merge commits using `git log --merges`. Each merge commit provides:
- Branch B (target / mainline): The first parent.
- Branch A (feature / topic): The second parent.
- Merge Base: The output of `git merge-base`.

## 2. Ground-Truth Definitions
Outcomes are explicitly extracted from historical execution artifacts, not inferred from semantic analysis.
- **Textual Merge Conflict:** Extracted via `git merge-tree`. If the output indicates conflicting chunks (`<<<<<<<`), the label is true.
- **Semantic Integration Issue:** A cleanly merged textual integration that results in a verifiable CI failure (build or test) directly on the merge commit, but not on the constituent parents.
- **Clean Integration:** Clean textually and clean in CI.

## 3. Provenance Requirements
Every `EvaluationCase` must retain:
- Repository URL and optional hint.
- Exact SHA-1 revisions for base, branch A, branch B, and merge base.
- A deterministic `case_id` derived from the repository identity and the merge commit hash.
- Diagnostic context regarding unavailability.

## 4. Unknown/Unavailable Handling
If a ground-truth label (e.g., CI outcome) cannot be reliably verified due to expired logs or missing external integrations, the corresponding `EvalOutcome` field MUST be left as `None`. We never substitute synthetic values. The BCS framework is explicitly designed to handle partial labels cleanly during calibration.

## 5. Determinism Guarantees
- Cases are sorted deterministically by their string-formatted `case_id`.
- The git abstractions use strictly pinned SHAs (not branch names).
- JSONL serialization follows a stable struct layout provided by `serde`.

## 6. Leakage Prevention
The generation of `EvalOutcome` labels is structurally isolated from BCS analysis. As proven by `isolation_test.rs`, the `branchsense-evaluation` runner does not consume `EvalOutcome` at any point during `engine.assess()`. The observed outcome is only appended to the `EvaluationResult` strictly after scoring is complete.

## 7. Known Limitations (Exploratory Phase)
This initial dataset generator is currently an exploratory prototype rather than a calibrated benchmark.
- **No CI Fetching:** The script currently relies purely on local Git state and lacks integration with GitHub/GitLab APIs to fetch historical build/test logs. Hence, semantic integration issues are currently left as `None` (unavailable).
- **Language Mismatch:** Running this tool against Rust repositories (like BranchSense itself) yields `Indeterminate` abstentions because the underlying BCS semantic pipeline is currently strictly configured for Java.

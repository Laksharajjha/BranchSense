# Java Evaluation Corpus & Diagnostic Report

## 1. Methodology and Provenance
We successfully utilized `branchsense-dataset` to extract 292 historical integrations from two major Java open-source repositories: `mockito/mockito` (150 cases) and `google/gson` (142 cases). The generator extracts base, branch A, and branch B revisions from merge commits, and runs an isolated `git merge-tree` to independently observe textual integration conflicts.

### Semantic Integration Issues Attribution Rule
A core methodological rule for this empirical dataset is that **semantic integration issues cannot be automatically deduced merely by observing a CI failure post-merge.** A CI failure on the main branch post-merge might be caused by flaky tests, infrastructure outages, or independent defects in one of the branches. 

To formally establish a semantic integration issue, the failure must be:
1. **Verifiable on the merge commit** (or the exact artifact produced by the merge).
2. **Absent on both parent commits** (the target branch prior to merge, and the feature branch prior to merge).
3. **Causally linked to the branch interaction** (e.g., a test failure in one branch's test suite caused by a behavioral change in the other branch's code).

Because the evaluation generator is strictly an offline Git tool and lacks API credentials to fetch historical CI logs (like GitHub Actions outputs), the semantic labels (`build_failure`, `test_failure`, `semantic_integration_issue`) are explicitly marked as `null` (Unavailable) across the entire corpus. **We strictly preserve unknown outcomes rather than fabricating labels.**

## 2. Diagnostics
## 3. Dataset Generator Regression Tests
Two critical deterministic behaviors of the generator are proven via tests:
1. **JSONL Determinism:** The generated JSON representation and case sort order are stable for the same inputs.
2. **Outcome Isolation:** Tested in `crates/branchsense-evaluation/tests/isolation_test.rs`. The `observed_outcome` property does not participate in the `BcsEngine` execution flow. Modifying the target outcome (e.g., asserting a SemanticIntegrationIssue instead of Clean) yields the exact same calculated `BcsAssessment`, mathematically preventing BCS from tuning itself implicitly to the ground truth during generation.

## 4. Frozen-Policy Findings
The evaluation was executed against the completely frozen `BcsPolicyV1` configuration (Thresholds: 10/30/60/90, and Multipliers: Collision × 5, Impact × 2, History × 1, Ownership × 1, Overlap × 1). No parameters were calibrated.

(Aggregate outputs will be appended below from the raw JSON result logs.)

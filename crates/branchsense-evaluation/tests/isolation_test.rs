//! BCS Data Leakage Isolation Test

use branchsense_semantic::{DatasetSchemaVersion, EvalOutcome, EvalRepositoryIdentity, EvalRevision};
use branchsense_evaluation::model::EvaluationCase;
use branchsense_evaluation::runner::run_evaluation_case;
use std::path::Path;

#[test]
fn test_outcome_isolation_real_commit() {
    let repo_path = Path::new(".");
    let repo_id = EvalRepositoryIdentity::new("test-repo", None::<String>);
    
    // We use actual commits from this repository
    let outcome_clean = EvalOutcome::new().with_textual_merge_conflict(false);
    let outcome_conflict = EvalOutcome::new().with_textual_merge_conflict(true).with_semantic_integration_issue(true);
    
    let base_rev = EvalRevision::new("361c07b00a1978f01bb015e96983388d7ac895b1");
    let branch_a = EvalRevision::new("4c7093268826ff5f062eff8d0caf26c55f6dd880");
    let branch_b = EvalRevision::new("8c321ce04af7164541690efa95a68314e2da2c57");

    let case_clean = EvaluationCase::new(
        "test-clean",
        DatasetSchemaVersion::current(),
        repo_id.clone(),
        base_rev.clone(),
        branch_a.clone(),
        branch_b.clone(),
        base_rev.clone(),
        outcome_clean,
    );

    let case_conflict = EvaluationCase::new(
        "test-conflict",
        DatasetSchemaVersion::current(),
        repo_id.clone(),
        base_rev.clone(),
        branch_a.clone(),
        branch_b.clone(),
        base_rev.clone(),
        outcome_conflict,
    );

    let result_clean = run_evaluation_case(&case_clean, repo_path);
    let result_conflict = run_evaluation_case(&case_conflict, repo_path);

    // Assert that the assessments are IDENTICAL despite wildly different outcomes.
    assert_eq!(result_clean.assessment(), result_conflict.assessment());
    assert_eq!(result_clean.diagnostic(), result_conflict.diagnostic());
}

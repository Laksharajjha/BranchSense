#![allow(missing_docs)]
use branchsense_evaluation::loader::EvaluationDataset;

#[test]
fn test_dataset_loader_rejects_duplicate_cases() {
    let jsonl = r#"
    {"case_id":"test-1","schema_version":{"major":1,"minor":0},"repository":{"id":"r","hint":"u"},"base_revision":{"hash":"a"},"branch_a_revision":{"hash":"b"},"branch_b_revision":{"hash":"c"},"merge_base":{"hash":"a"},"observed_outcome":{}}
    {"case_id":"test-1","schema_version":{"major":1,"minor":0},"repository":{"id":"r","hint":"u"},"base_revision":{"hash":"a"},"branch_a_revision":{"hash":"b"},"branch_b_revision":{"hash":"c"},"merge_base":{"hash":"a"},"observed_outcome":{}}
    "#;

    let dataset = EvaluationDataset::from_jsonl(jsonl);
    assert!(!dataset.is_valid());
    assert_eq!(dataset.diagnostics().len(), 1);
    assert_eq!(dataset.cases().len(), 1);
}

#[test]
fn test_dataset_loader_rejects_malformed_revisions() {
    let jsonl = r#"
    {"case_id":"test-2","schema_version":{"major":1,"minor":0},"repository":{"id":"r","hint":"u"},"base_revision":{"hash":""},"branch_a_revision":{"hash":"b"},"branch_b_revision":{"hash":"c"},"merge_base":{"hash":"a"},"observed_outcome":{}}
    "#;

    let dataset = EvaluationDataset::from_jsonl(jsonl);
    assert!(!dataset.is_valid());
    assert_eq!(dataset.diagnostics().len(), 1);
    assert_eq!(dataset.cases().len(), 0);
}

#[test]
fn test_dataset_loader_allows_missing_outcomes() {
    let jsonl = r#"
    {"case_id":"test-3","schema_version":{"major":1,"minor":0},"repository":{"id":"r","hint":"u"},"base_revision":{"hash":"a"},"branch_a_revision":{"hash":"b"},"branch_b_revision":{"hash":"c"},"merge_base":{"hash":"a"},"observed_outcome":{"build_failure":true}}
    {"case_id":"test-4","schema_version":{"major":1,"minor":0},"repository":{"id":"r","hint":"u"},"base_revision":{"hash":"a"},"branch_a_revision":{"hash":"b"},"branch_b_revision":{"hash":"c"},"merge_base":{"hash":"a"},"observed_outcome":{}}
    "#;

    let dataset = EvaluationDataset::from_jsonl(jsonl);
    assert!(dataset.is_valid());
    assert_eq!(dataset.cases().len(), 2);
}

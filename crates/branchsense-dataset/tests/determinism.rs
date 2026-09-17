//! Determinism tests for dataset generator.
use branchsense_evaluation::model::EvaluationCase;
use branchsense_semantic::{
    DatasetSchemaVersion, EvalOutcome, EvalRepositoryIdentity, EvalRevision,
};

#[test]
fn test_serialization_determinism() {
    let repo_id =
        EvalRepositoryIdentity::new("https://github.com/example/repo", Some("example/repo"));
    let outcome = EvalOutcome::new().with_textual_merge_conflict(true);
    let case = EvaluationCase::new(
        "case-123",
        DatasetSchemaVersion::current(),
        repo_id,
        EvalRevision::new("aaaaaaa"),
        EvalRevision::new("bbbbbbb"),
        EvalRevision::new("ccccccc"),
        EvalRevision::new("ddddddd"),
        outcome,
    );

    let json1 = serde_json::to_string(&case).unwrap();
    let json2 = serde_json::to_string(&case).unwrap();
    assert_eq!(json1, json2, "Serialization must be completely deterministic");

    // Check that field ordering is stable (serde defaults to struct definition order unless sorted)
    // We expect the JSON to match a fixed literal string to prevent accidental schema reordering.
    let expected = r#"{"case_id":"case-123","schema_version":{"major":1,"minor":0},"repository":{"id":"https://github.com/example/repo","hint":"example/repo"},"base_revision":{"hash":"aaaaaaa"},"branch_a_revision":{"hash":"bbbbbbb"},"branch_b_revision":{"hash":"ccccccc"},"merge_base":{"hash":"ddddddd"},"observed_outcome":{"textual_merge_conflict":true,"build_failure":null,"test_failure":null,"semantic_integration_issue":null}}"#;
    assert_eq!(json1, expected, "Schema fields must be serialized in canonical order");
}

//! Tests for overlap normalization.

use branchsense_bcs::adapter::normalize_overlap;
use branchsense_overlap::OverlapSet;

fn create_overlap_set(json_override: &str) -> OverlapSet {
    let empty = OverlapSet::default();
    let mut val: serde_json::Value = serde_json::to_value(&empty).unwrap();
    let overrides: serde_json::Value = serde_json::from_str(json_override).unwrap();

    if let (Some(tgt), Some(src)) = (val.as_object_mut(), overrides.as_object()) {
        for (k, v) in src {
            tgt.insert(k.clone(), v.clone());
        }
    }
    serde_json::from_value(val).unwrap()
}

#[test]
fn test_overlap_normalization_empty() {
    let empty = OverlapSet::default();
    let normalized = normalize_overlap(&empty);
    assert!(normalized.is_empty(), "Empty overlap should produce no evidence");
}

#[test]
fn test_overlap_normalization_direct_vs_shared() {
    let direct = create_overlap_set(
        r#"{
        "entries": [{
            "explanation": {
                "branch_a_changed": "com.example.Foo",
                "branch_b_changed": "com.example.Foo",
                "branch_a_change_kind": null,
                "branch_b_change_kind": null,
                "targets": ["com.example.Foo"],
                "kind": "DirectChange",
                "branch_a_evidence": [],
                "branch_b_evidence": []
            }
        }],
        "statistics": {
            "branch_a_changed": 1,
            "branch_b_changed": 1,
            "overlaps": 1,
            "direct_changes": 1,
            "impact_changes": 0,
            "shared_impacts": 0, "cross_impacts": 0, "max_depth": 0, "truncated": false
        }
    }"#,
    );

    let shared = create_overlap_set(
        r#"{
        "entries": [{
            "explanation": {
                "branch_a_changed": "com.example.Foo",
                "branch_b_changed": "com.example.Bar",
                "branch_a_change_kind": null,
                "branch_b_change_kind": null,
                "targets": ["com.example.Baz"],
                "kind": "SharedImpact",
                "branch_a_evidence": [],
                "branch_b_evidence": []
            }
        }],
        "statistics": {
            "branch_a_changed": 1,
            "branch_b_changed": 1,
            "overlaps": 1,
            "direct_changes": 0,
            "impact_changes": 0,
            "shared_impacts": 1, "cross_impacts": 0, "max_depth": 0, "truncated": false
        }
    }"#,
    );

    let n_direct = normalize_overlap(&direct);
    let n_shared = normalize_overlap(&shared);

    assert_eq!(n_direct.len(), 1);
    assert_eq!(n_shared.len(), 1);
    assert!(n_direct[0].strength() > n_shared[0].strength());
    assert!(n_direct[0].description().contains("Direct"));
    assert!(n_shared[0].description().contains("Shared"));
}

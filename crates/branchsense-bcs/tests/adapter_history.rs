//! Tests for history normalization.

use branchsense_bcs::adapter::normalize_history;
use branchsense_history::HistoricalSignals;

fn create_history(json_override: &str) -> HistoricalSignals {
    // In order to instantiate without default, we construct minimal JSON and merge
    let base = r#"{
        "analysis_revision": "0000000000000000000000000000000000000000",
        "commits_analyzed": 100,
        "evidence": {
            "identity": "eval-1",
            "relation": "Subject",
            "identities": [],
            "lineage": [],
            "provenance": {
                "system_version": "1.0",
                "analysis_timestamp_utc": "2026-09-15T12:00:00Z"
            },
            "completeness": {
                "semantic": "Observed",
                "historical": "Observed",
                "responsibility": "Observed"
            },
            "state": "Observed"
        },
        "change_frequency": [],
        "recency": [],
        "symbol_co_change": [],
        "file_co_change": []
    }"#;
    let mut val: serde_json::Value = serde_json::from_str(base).unwrap();
    let overrides: serde_json::Value = serde_json::from_str(json_override).unwrap();

    if let (Some(tgt), Some(src)) = (val.as_object_mut(), overrides.as_object()) {
        for (k, v) in src {
            tgt.insert(k.clone(), v.clone());
        }
    }
    serde_json::from_value(val).unwrap()
}

#[test]
fn test_history_normalization_empty() {
    let empty = create_history("{}");
    let normalized = normalize_history(&empty);
    assert!(normalized.is_empty(), "Empty history should produce no evidence");
}

#[test]
fn test_history_normalization_co_change() {
    let history = create_history(
        r#"{
        "symbol_co_change": [
            {
                "left": {"document": "src/System.java", "qualified_name": "com.example.Foo", "kind": "Type"},
                "right": {"document": "src/System.java", "qualified_name": "com.example.Bar", "kind": "Method"},
                "co_change_count": 2,
                "commits_considered": 100,
                "revisions": []
            }
        ],
        "recency": [
            {
                "symbol": {"document": "src/System.java", "qualified_name": "com.example.Foo", "kind": "Type"},
                "last_changed_revision": "0000000000000000000000000000000000000000",
                "last_changed_timestamp_seconds": 0,
                "age_in_commits": 5
            }
        ]
    }"#,
    );

    let n = normalize_history(&history);
    assert_eq!(n.len(), 1);

    // 2 co-changes * 5 = 10, plus recency boost of 5 = 15
    assert_eq!(n[0].strength(), 15);
    assert!(n[0].description().contains("Recent historical co-changes"));
    assert_eq!(n[0].affected_entities(), vec!["com.example.Foo & com.example.Bar"]);
}

#[test]
fn test_history_normalization_truncated() {
    let history = create_history(
        r#"{
        "evidence": {
            "identity": "eval-1",
            "relation": "Subject",
            "identities": [],
            "lineage": [],
            "provenance": {
                "system_version": "1.0",
                "analysis_timestamp_utc": "2026-09-15T12:00:00Z"
            },
            "completeness": {
                "semantic": "Truncated",
                "historical": "Truncated",
                "responsibility": "Truncated"
            },
            "state": "Truncated"
        }
    }"#,
    );

    let n = normalize_history(&history);
    assert_eq!(n.len(), 1);
    assert_eq!(n[0].strength(), 10);
    assert_eq!(n[0].description(), "Truncated history");
}

use branchsense_bcs::adapter::normalize_ownership;
use branchsense_ownership::ResponsibilitySignals;

fn create_ownership(json_override: &str) -> ResponsibilitySignals {
    let base = r#"{
        "analysis_revision": "0000000000000000000000000000000000000000",
        "commits_analyzed": 100,
        "recent_window": 10,
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
        "symbol_responsibility": [],
        "file_responsibility": []
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
fn test_ownership_normalization_empty() {
    let empty = create_ownership("{}");
    let normalized = normalize_ownership(&empty);
    assert!(normalized.is_empty(), "Empty ownership should produce no evidence");
}

#[test]
fn test_ownership_normalization_strong() {
    let ownership = create_ownership(r#"{
        "symbol_responsibility": [
            {
                "entity": {
                    "Symbol": {
                        "document": "src/System.java",
                        "kind": "Type",
                        "qualified_name": "com.example.Foo"
                    }
                },
                "scope": "Symbol",
                "attribution_basis": "SemanticChange",
                "contributions": [],
                "recent_contributors": [],
                "concentration": {
                    "top_contributor_share": 0.9,
                    "active_contributors": 2
                },
                "supporting_commits": []
            }
        ]
    }"#);

    let n = normalize_ownership(&ownership);
    assert_eq!(n.len(), 1);
    
    // 0.9 * 20 = 18 + 2 (contributors) = 20
    assert_eq!(n[0].strength(), 20);
    assert!(n[0].description().contains("Strong responsibility"));
    assert_eq!(n[0].affected_entities(), vec!["com.example.Foo"]);
}

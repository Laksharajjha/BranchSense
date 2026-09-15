use branchsense_bcs::adapter::normalize_impact;
use branchsense_impact::{ImpactSet};

fn create_impact_set(json_override: &str) -> ImpactSet {
    let empty = ImpactSet::default();
    let mut val: serde_json::Value = serde_json::to_value(&empty).unwrap();
    let overrides: serde_json::Value = serde_json::from_str(json_override).unwrap();
    
    // Merge overrides into default
    if let (Some(tgt), Some(src)) = (val.as_object_mut(), overrides.as_object()) {
        for (k, v) in src {
            tgt.insert(k.clone(), v.clone());
        }
    }
    serde_json::from_value(val).unwrap()
}

#[test]
fn test_impact_normalization_empty() {
    let empty = ImpactSet::default();
    let normalized = normalize_impact(&empty);
    assert!(normalized.is_empty(), "Empty impact should produce no evidence");
}

#[test]
fn test_impact_normalization_direct_vs_transitive() {
    let direct = create_impact_set(r#"{
        "entries": [{
            "impacted_symbol": "com.example.Foo",
            "causes": [{
                "explanation": {
                    "changed_symbol": "com.example.Bar",
                    "relationship": "Calls",
                    "kind": "DirectCaller",
                    "depth": 1,
                    "path": { "steps": [] }
                },
                "relationship_fact": null
            }]
        }],
        "statistics": {
            "changed_symbols": 1,
            "impacted_symbols": 1,
            "direct_impacts": 1,
            "transitive_impacts": 0,
            "max_depth": 1,
            "truncated": false
        }
    }"#);

    let transitive = create_impact_set(r#"{
        "entries": [{
            "impacted_symbol": "com.example.Foo",
            "causes": [{
                "explanation": {
                    "changed_symbol": "com.example.Bar",
                    "relationship": "Calls",
                    "kind": "TransitiveCaller",
                    "depth": 3,
                    "path": { "steps": [] }
                },
                "relationship_fact": null
            }]
        }],
        "statistics": {
            "changed_symbols": 1,
            "impacted_symbols": 1,
            "direct_impacts": 0,
            "transitive_impacts": 1,
            "max_depth": 3,
            "truncated": false
        }
    }"#);

    let n_direct = normalize_impact(&direct);
    let n_trans = normalize_impact(&transitive);
    
    assert_eq!(n_direct.len(), 1);
    assert_eq!(n_trans.len(), 1);
    assert!(n_direct[0].strength() > n_trans[0].strength());
    assert_eq!(n_direct[0].affected_entities(), vec!["com.example.Foo"]);
}

#[test]
fn test_impact_normalization_signature_consumer_and_truncation() {
    let impact = create_impact_set(r#"{
        "entries": [{
            "impacted_symbol": "com.example.Foo",
            "causes": [{
                "explanation": {
                    "changed_symbol": "com.example.Bar",
                    "relationship": "Calls",
                    "kind": "SignatureConsumer",
                    "depth": 1,
                    "path": { "steps": [] }
                },
                "relationship_fact": null
            }]
        }],
        "statistics": {
            "changed_symbols": 1,
            "impacted_symbols": 1,
            "direct_impacts": 1,
            "transitive_impacts": 0,
            "max_depth": 1,
            "truncated": true
        }
    }"#);

    let n = normalize_impact(&impact);
    // direct (5) + sig consumer (10) + truncated (20) = 35
    assert_eq!(n[0].strength(), 35);
    assert!(n[0].description().contains("signature consumer"));
}

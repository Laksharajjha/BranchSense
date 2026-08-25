//! Benchmark for deterministic collision assessment composition.
#![allow(missing_docs)]

use std::hint::black_box;

use branchsense_collision::CollisionAnalyzer;
use branchsense_overlap::OverlapSet;
use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::json;

fn fixture(count: usize, depth: usize, kind: &str) -> OverlapSet {
    let entries = (0..count)
        .map(|index| {
            let a = format!("symbol:a{index}");
            let b = format!("symbol:b{index}");
            json!({
                "explanation": {
                    "branch_a_changed": a,
                    "branch_b_changed": b,
                    "branch_a_change_kind": "Modified",
                    "branch_b_change_kind": "Modified",
                    "targets": [b],
                    "kind": "ImpactChange",
                    "branch_a_evidence": [{
                        "changed_symbol": format!("symbol:a{index}"),
                        "target_symbol": format!("symbol:b{index}"),
                        "kind": kind,
                        "relationship": "Calls",
                        "depth": depth,
                        "path": {"steps": []},
                        "relationship_fact": null
                    }],
                    "branch_b_evidence": []
                }
            })
        })
        .collect::<Vec<_>>();
    serde_json::from_value(json!({
        "entries": entries,
        "statistics": {
            "branch_a_changed": count,
            "branch_b_changed": count,
            "overlaps": count,
            "direct_changes": 0,
            "impact_changes": count,
            "shared_impacts": 0,
            "cross_impacts": 0,
            "max_depth": depth,
            "truncated": false
        }
    }))
    .expect("valid collision benchmark fixture")
}

fn benchmark_empty_assessment(criterion: &mut Criterion) {
    let analyzer = CollisionAnalyzer::new();
    let overlaps = OverlapSet::default();
    criterion.bench_function("collision_empty", |benchmark| {
        benchmark.iter(|| analyzer.analyze(black_box(&overlaps)));
    });
}

fn benchmark_assessment_sizes(criterion: &mut Criterion) {
    let analyzer = CollisionAnalyzer::new();
    for count in [1, 10, 100] {
        let overlaps = fixture(count, 1, "DirectCaller");
        criterion.bench_function(&format!("collision_{count}_direct_impacts"), |benchmark| {
            benchmark.iter(|| analyzer.analyze(black_box(&overlaps)));
        });
    }
    let deep = fixture(10, 3, "TransitiveCaller");
    criterion.bench_function("collision_10_deep_impacts", |benchmark| {
        benchmark.iter(|| analyzer.analyze(black_box(&deep)));
    });
}

criterion_group!(benches, benchmark_empty_assessment, benchmark_assessment_sizes);
criterion_main!(benches);

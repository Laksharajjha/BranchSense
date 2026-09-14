#![allow(missing_docs, clippy::cast_possible_truncation)]
//! Benchmarks for the BCS aggregation and scoring engine.

use branchsense_bcs::ledger::BcsEvidenceAggregator;
use branchsense_bcs::normalization::{BcsEvidenceCategory, BcsNormalizedEvidence};
use branchsense_bcs::score::BcsEngine;
use branchsense_semantic::{
    AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceState,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn generate_evidence(count: usize, category: BcsEvidenceCategory) -> Vec<BcsNormalizedEvidence> {
    let mut vec = Vec::with_capacity(count);
    let env = EvidenceEnvelope::new(
        EvidenceState::Observed,
        EvidenceCompleteness::new(),
        AnalysisProvenance::new(),
    );
    for i in 0..count {
        vec.push(BcsNormalizedEvidence::new(
            category,
            env.clone(),
            vec![format!("Entity{i}")],
            format!("Desc {i}"),
            (i % 10) as u8,
        ));
    }
    vec
}

fn bench_engine(c: &mut Criterion) {
    let engine = BcsEngine::new();

    c.bench_function("bcs_empty_evidence", |b| {
        b.iter(|| {
            let aggregator = BcsEvidenceAggregator::new();
            black_box(engine.assess(black_box(&aggregator)));
        });
    });

    c.bench_function("bcs_small_evidence_set", |b| {
        let evidence = generate_evidence(10, BcsEvidenceCategory::Collision);
        b.iter(|| {
            let mut aggregator = BcsEvidenceAggregator::new();
            for e in &evidence {
                aggregator.add(e.clone());
            }
            black_box(engine.assess(black_box(&aggregator)));
        });
    });

    c.bench_function("bcs_large_evidence_set", |b| {
        let evidence = generate_evidence(1000, BcsEvidenceCategory::Collision);
        b.iter(|| {
            let mut aggregator = BcsEvidenceAggregator::new();
            for e in &evidence {
                aggregator.add(e.clone());
            }
            black_box(engine.assess(black_box(&aggregator)));
        });
    });

    c.bench_function("bcs_duplicate_heavy_evidence", |b| {
        let evidence = generate_evidence(10, BcsEvidenceCategory::Collision);
        // Duplicate them 100 times to test ledger dedup
        let mut duplicates = Vec::new();
        for _ in 0..100 {
            duplicates.extend(evidence.clone());
        }

        b.iter(|| {
            let mut aggregator = BcsEvidenceAggregator::new();
            for e in &duplicates {
                aggregator.add(e.clone());
            }
            black_box(engine.assess(black_box(&aggregator)));
        });
    });
}

criterion_group!(benches, bench_engine);
criterion_main!(benches);

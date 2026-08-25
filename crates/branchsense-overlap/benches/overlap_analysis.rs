//! Benchmark for deterministic overlap composition.
#![allow(missing_docs)]

use std::hint::black_box;

use branchsense_impact::ImpactSet;
use branchsense_index::{IndexOptions, RepositoryIndex};
use branchsense_overlap::SemanticOverlapAnalyzer;
use criterion::{Criterion, criterion_group, criterion_main};

fn benchmark_empty_analysis(criterion: &mut Criterion) {
    let analyzer = SemanticOverlapAnalyzer::new();
    let directory = tempfile::tempdir().expect("temporary source directory");
    let snapshot = RepositoryIndex::new(IndexOptions::default())
        .index(directory.path(), None)
        .expect("empty directory indexes")
        .into_parts()
        .0;
    let diff = branchsense_diff::SemanticDiffer::new().diff(&snapshot, &snapshot);
    let impacts = ImpactSet::default();
    criterion.bench_function("overlap_empty", |benchmark| {
        benchmark.iter(|| analyzer.analyze(black_box(&diff), black_box(&impacts), &diff, &impacts));
    });
}

criterion_group!(benches, benchmark_empty_analysis);
criterion_main!(benches);

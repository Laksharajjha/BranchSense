#![allow(missing_docs)]

use std::hint::black_box;

use branchsense_diff::SemanticDiffer;
use branchsense_impact::ImpactAnalyzer;
use branchsense_index::{IndexOptions, RepositoryIndex};
use criterion::{Criterion, criterion_group, criterion_main};

fn benchmark_impact(c: &mut Criterion) {
    let root = tempfile::tempdir().expect("repository");
    std::fs::write(
        root.path().join("Service.java"),
        "class Service { void run() {} } class Caller { void call() { new Service().run(); } }",
    )
    .expect("source");
    let before = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("index")
        .into_parts()
        .0;
    std::fs::write(root.path().join("Service.java"), "class Service { void run(User user) {} } class Caller { void call() { new Service().run(null); } }").expect("changed source");
    let after = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("index")
        .into_parts()
        .0;
    let diff = SemanticDiffer::new().diff(&before, &after);
    c.bench_function("impact_one_changed_symbol", |benchmark| {
        benchmark.iter(|| black_box(ImpactAnalyzer::new().analyze(&diff, &before, &after)));
    });
}

criterion_group!(benches, benchmark_impact);
criterion_main!(benches);

#![allow(missing_docs)]

use std::hint::black_box;

use branchsense_diff::SemanticDiffer;
use branchsense_impact::{ImpactAnalyzer, ImpactOptions};
use branchsense_index::{IndexOptions, RepositoryIndex};
use criterion::{Criterion, criterion_group, criterion_main};

fn fixture(
    count: usize,
) -> (
    tempfile::TempDir,
    branchsense_index::SemanticIndexSnapshot,
    branchsense_index::SemanticIndexSnapshot,
    branchsense_diff::SemanticDiff,
) {
    let root = tempfile::tempdir().expect("repository");
    let before_source = (0..count).map(|index| format!("class Service{index} {{ void run() {{}} }} class Caller{index} {{ void call() {{ Service{index}.run(); }} }}")).collect::<Vec<_>>().join("\n");
    let after_source = (0..count).map(|index| format!("class Service{index} {{ void run(User user) {{}} }} class Caller{index} {{ void call() {{ Service{index}.run(null); }} }}")).collect::<Vec<_>>().join("\n");
    std::fs::write(root.path().join("Service.java"), before_source).expect("source");
    let before = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("index")
        .into_parts()
        .0;
    std::fs::write(root.path().join("Service.java"), after_source).expect("changed source");
    let after = RepositoryIndex::new(IndexOptions::default())
        .index(root.path(), None)
        .expect("index")
        .into_parts()
        .0;
    let diff = SemanticDiffer::new().diff(&before, &after);
    (root, before, after, diff)
}

fn benchmark_impact(c: &mut Criterion) {
    for count in [1, 10, 100] {
        let (_root, before, after, diff) = fixture(count);
        c.bench_function(&format!("impact_{count}_changed_symbols"), |benchmark| {
            benchmark.iter(|| black_box(ImpactAnalyzer::new().analyze(&diff, &before, &after)));
        });
    }
    let (_root, before, after, diff) = fixture(100);
    for depth in [1, 3, 6] {
        c.bench_function(&format!("impact_100_symbols_depth_{depth}"), |benchmark| {
            let analyzer = ImpactAnalyzer::with_options(ImpactOptions::new(depth, 10_000));
            benchmark.iter(|| black_box(analyzer.analyze(&diff, &before, &after)));
        });
    }
}

criterion_group!(benches, benchmark_impact);
criterion_main!(benches);

#![allow(missing_docs)]

use std::fs;
use std::hint::black_box;

use branchsense_diff::SemanticDiffer;
use branchsense_index::{IndexOptions, RepositoryIndex};
use criterion::{Criterion, criterion_group, criterion_main};

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary repository");
    let source = root.path().join("src/main/java/payment");
    fs::create_dir_all(&source).expect("source directory");
    for index in 0..100 {
        fs::write(
            source.join(format!("PaymentService{index}.java")),
            format!("package payment; public class PaymentService{index} {{ public void process() {{}} }}"),
        )
        .expect("Java source");
    }
    root
}

fn semantic_diff(criterion: &mut Criterion) {
    let root = repository();
    let indexer = RepositoryIndex::new(IndexOptions::default());
    let before = indexer.index(root.path(), None).expect("before snapshot");
    fs::write(
        root.path().join("src/main/java/payment/PaymentService0.java"),
        "package payment; public class PaymentService0 { public void changed() {} }",
    )
    .expect("changed Java source");
    let after = indexer.index(root.path(), None).expect("after snapshot");
    let differ = SemanticDiffer::new();

    let mut group = criterion.benchmark_group("semantic_diff");
    group.bench_function("empty_snapshot_diff", |benchmark| {
        benchmark.iter(|| black_box(differ.diff(before.snapshot(), before.snapshot())));
    });
    group.bench_function("medium_repository_diff_100_files", |benchmark| {
        benchmark.iter(|| black_box(differ.diff(before.snapshot(), after.snapshot())));
    });
    group.finish();
}

criterion_group!(benches, semantic_diff);
criterion_main!(benches);

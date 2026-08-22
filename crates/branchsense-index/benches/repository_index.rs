#![allow(missing_docs)]

use std::fs;

use branchsense_index::{DiscoveryOptions, IndexOptions, RepositoryIndex, SourceDiscovery};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary root");
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

fn repository_index(criterion: &mut Criterion) {
    let root = fixture();
    let indexer = RepositoryIndex::new(IndexOptions::default());
    let mut group = criterion.benchmark_group("repository_index");
    group.bench_function("source_discovery_100_files", |benchmark| {
        benchmark.iter(|| {
            black_box(
                SourceDiscovery::new(DiscoveryOptions::default())
                    .discover(root.path())
                    .expect("discovery"),
            )
        });
    });
    group.bench_function("full_index_100_files", |benchmark| {
        benchmark.iter(|| black_box(indexer.index(root.path(), None).expect("index")));
    });
    let initial = indexer.index(root.path(), None).expect("initial index");
    fs::write(
        root.path().join("src/main/java/payment/PaymentService0.java"),
        "package payment; public class PaymentService0 { public void changed() {} }",
    )
    .expect("changed Java source");
    group.bench_function("incremental_index_one_changed_file", |benchmark| {
        benchmark.iter(|| {
            black_box(
                indexer.index(root.path(), Some(initial.snapshot())).expect("incremental index"),
            )
        });
    });
    group.finish();
}

criterion_group!(benches, repository_index);
criterion_main!(benches);

#![allow(missing_docs)]

use std::path::PathBuf;

use branchsense_git::GitRepository;
use criterion::{Criterion, criterion_group, criterion_main};

fn repository() -> GitRepository {
    GitRepository::discover(PathBuf::from(env!("CARGO_MANIFEST_DIR")))
        .expect("benchmark must run inside a Git checkout")
}

fn benchmark_repository(c: &mut Criterion) {
    let repository = repository();
    c.bench_function("git_head_resolution", |benchmark| {
        benchmark.iter(|| repository.head().expect("HEAD resolves"));
    });
    c.bench_function("git_branch_resolution", |benchmark| {
        benchmark.iter(|| repository.local_branches().expect("branches resolve"));
    });
}

criterion_group!(benches, benchmark_repository);
criterion_main!(benches);

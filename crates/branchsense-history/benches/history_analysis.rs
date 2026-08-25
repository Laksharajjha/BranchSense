//! Benchmark harness for bounded historical analysis.
#![allow(missing_docs)]

use std::{fs, process::Command};

use branchsense_git::GitRepository;
use branchsense_history::{HistoricalAnalyzer, HistoricalOptions};
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::TempDir;

fn run(root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git").args(args).current_dir(root).status().expect("git runs");
    assert!(status.success(), "git command failed: {args:?}");
}

fn fixture() -> (TempDir, GitRepository) {
    let directory = tempfile::tempdir().expect("temporary repository");
    let root = directory.path();
    run(root, &["init", "-q", "-b", "main"]);
    run(root, &["config", "user.email", "bench@example.com"]);
    run(root, &["config", "user.name", "BranchSense Benchmark"]);
    fs::create_dir(root.join("src")).expect("source directory");
    fs::write(root.join("src/Sample.java"), "class Sample { int value0; }\n").expect("source");
    run(root, &["add", "."]);
    run(root, &["commit", "-q", "-m", "base"]);
    for index in 1..=1_000 {
        fs::write(root.join("src/Sample.java"), format!("class Sample {{ int value{index}; }}\n"))
            .expect("source");
        run(root, &["commit", "-q", "-am", "history benchmark"]);
    }
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    (directory, repository)
}

fn benchmark_history(criterion: &mut Criterion) {
    let (_directory, repository) = fixture();
    let revision = repository.resolve("main").expect("resolve main");
    let analyzer = HistoricalAnalyzer::new();
    for max_commits in [10, 100, 500, 1_000] {
        criterion.bench_function(&format!("history_{max_commits}_commits"), |benchmark| {
            benchmark.iter(|| {
                analyzer
                    .analyze(&repository, &revision, HistoricalOptions::new(max_commits))
                    .expect("history analysis")
            });
        });
    }
}

criterion_group!(benches, benchmark_history);
criterion_main!(benches);

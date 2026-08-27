#![allow(missing_docs)]

use std::{fs, hint::black_box, process::Command};

use branchsense_git::GitRepository;
use branchsense_ownership::{ResponsibilityAnalyzer, ResponsibilityOptions};
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::TempDir;

fn run(root: &std::path::Path, args: &[&str]) {
    assert!(Command::new("git").args(args).current_dir(root).status().expect("git runs").success());
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
    for index in 1..=100 {
        fs::write(root.join("src/Sample.java"), format!("class Sample {{ int value{index}; }}\n"))
            .expect("source");
        run(root, &["commit", "-q", "-am", "history benchmark"]);
    }
    let repository = GitRepository::discover(directory.path()).expect("discover repository");
    (directory, repository)
}

fn responsibility_analysis(criterion: &mut Criterion) {
    let (_directory, repository) = fixture();
    let revision = repository.resolve("main").expect("resolve main");
    let analyzer = ResponsibilityAnalyzer::new();
    criterion.bench_function("ownership_100_commits", |benchmark| {
        benchmark.iter(|| {
            black_box(
                analyzer
                    .analyze(&repository, &revision, ResponsibilityOptions::new(100))
                    .expect("analysis"),
            );
        });
    });
}

criterion_group!(benches, responsibility_analysis);
criterion_main!(benches);

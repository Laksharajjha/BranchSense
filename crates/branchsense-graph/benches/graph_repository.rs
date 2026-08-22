#![allow(missing_docs)]

use std::{fmt::Write, hint::black_box};

use branchsense_core::{DocumentId, RevisionId};
use branchsense_extractor_java::JavaExtractor;
use branchsense_graph::SemanticGraph;
use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use branchsense_semantic::SemanticFactSet;
use criterion::{Criterion, criterion_group, criterion_main};

fn source(extra_field: bool) -> String {
    let mut source = String::from("package benchmark;\npublic class RepositoryFixture {\n");
    for index in 0..100 {
        writeln!(source, "    private String field{index};").expect("String writes cannot fail");
        writeln!(source, "    public void method{index}() {{ helper{index}(); }}")
            .expect("String writes cannot fail");
    }
    if extra_field {
        source.push_str("    private String changedField;\n");
    }
    source.push_str("    private void helper0() {}\n}\n");
    source
}

fn construct_empty_graph(c: &mut Criterion) {
    c.bench_function("graph_construct_empty_document", |benchmark| {
        benchmark.iter(|| {
            SemanticGraph::from_document_facts(
                DocumentId::new("benchmark.java").expect("document ID"),
                RevisionId::new("revision:benchmark").expect("revision ID"),
                SemanticFactSet::default(),
            )
            .expect("graph builds")
        });
    });
}

fn benchmark_repository_pipeline(c: &mut Criterion) {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let extractor = JavaExtractor::new();
    let initial_source = source(false);
    let changed_source = source(true);
    let initial_input = || {
        ParseInput::new(
            "RepositoryFixture.java",
            initial_source.clone(),
            DocumentVersion::initial(),
        )
    };

    c.bench_function("repository_parse", |benchmark| {
        benchmark
            .iter(|| black_box(parser.parse_source(black_box(initial_input())).expect("parses")));
    });

    let parsed = parser.parse_source(initial_input()).expect("parses").into_document();
    c.bench_function("repository_extract", |benchmark| {
        benchmark.iter(|| black_box(extractor.extract(black_box(&parsed)).expect("extracts")));
    });
    let initial_facts = extractor.extract(&parsed).expect("extracts").facts().clone();
    let graph = SemanticGraph::from_document_facts(
        DocumentId::new("RepositoryFixture.java").expect("document ID"),
        RevisionId::new("revision:benchmark").expect("revision ID"),
        initial_facts.clone(),
    )
    .expect("graph builds");
    assert!(graph.statistics().nodes() > 100);
    assert!(graph.statistics().edges() > 100);

    c.bench_function("repository_graph_construct", |benchmark| {
        benchmark.iter(|| {
            black_box(SemanticGraph::from_document_facts(
                DocumentId::new("RepositoryFixture.java").expect("document ID"),
                RevisionId::new("revision:benchmark").expect("revision ID"),
                black_box(initial_facts.clone()),
            ))
        });
    });

    let changed = extractor
        .extract(
            parser
                .parse_source(ParseInput::new(
                    "RepositoryFixture.java",
                    changed_source,
                    DocumentVersion::new(1),
                ))
                .expect("parses")
                .document(),
        )
        .expect("extracts")
        .facts()
        .clone();
    c.bench_function("repository_graph_document_update", |benchmark| {
        benchmark.iter(|| {
            black_box(
                graph
                    .replace_document_facts(
                        DocumentId::new("RepositoryFixture.java").expect("document ID"),
                        RevisionId::new("revision:changed").expect("revision ID"),
                        black_box(changed.clone()),
                    )
                    .expect("update succeeds"),
            )
        });
    });
}

criterion_group!(benches, construct_empty_graph, benchmark_repository_pipeline);
criterion_main!(benches);

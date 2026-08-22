#![allow(missing_docs)]

use branchsense_core::{DocumentId, RevisionId};
use branchsense_extractor_java::JavaExtractor;
use branchsense_graph::SemanticGraph;
use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use branchsense_query::{Query, QueryOptions};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn graph() -> SemanticGraph {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let source = "package billing; public class PaymentService { public void process() { validate(); } public void validate() {} }";
    let parsed = parser
        .parse_source(ParseInput::new("PaymentService.java", source, DocumentVersion::initial()))
        .expect("source parses");
    let facts =
        JavaExtractor::new().extract(parsed.document()).expect("facts extract").facts().clone();
    SemanticGraph::from_document_facts(
        DocumentId::new("PaymentService.java").expect("document ID"),
        RevisionId::new("revision:bench").expect("revision ID"),
        facts,
    )
    .expect("graph builds")
}

fn queries(criterion: &mut Criterion) {
    let graph = graph();
    let query = Query::new(&graph);
    let process = query.symbols_by_name("process").items()[0].id().clone();
    let mut group = criterion.benchmark_group("semantic_queries");
    group.bench_function("symbol_lookup", |benchmark| {
        benchmark.iter(|| black_box(query.symbol(&process).expect("symbol")));
    });
    group.bench_function("callers", |benchmark| {
        benchmark
            .iter(|| black_box(query.callers(&process, QueryOptions::new()).expect("callers")));
    });
    group.bench_function("callees", |benchmark| {
        benchmark
            .iter(|| black_box(query.callees(&process, QueryOptions::new()).expect("callees")));
    });
    group.bench_function("bounded_traversal", |benchmark| {
        benchmark.iter(|| {
            black_box(query.dependency_tree(&process, 3, QueryOptions::new()).expect("traversal"))
        });
    });
    group.finish();
}

criterion_group!(benches, queries);
criterion_main!(benches);

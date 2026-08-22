#![allow(missing_docs)]

use branchsense_core::{DocumentId, RevisionId};
use branchsense_graph::SemanticGraph;
use branchsense_semantic::SemanticFactSet;
use criterion::{Criterion, criterion_group, criterion_main};

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

criterion_group!(benches, construct_empty_graph);
criterion_main!(benches);

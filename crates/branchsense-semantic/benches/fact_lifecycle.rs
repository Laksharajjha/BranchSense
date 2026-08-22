#![allow(missing_docs)]

use branchsense_core::{
    DocumentId, Location, Name, Position, QualifiedName, Range, RevisionId, SymbolId,
};
use branchsense_semantic::{
    FactDelta, FactId, SemanticFact, SemanticFactRecord, SemanticFactSet, SymbolDefinition,
    SymbolKind,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn records(count: usize) -> SemanticFactSet {
    let document = DocumentId::new("benchmark.java").expect("document ID");
    let location = Location::new(
        document,
        Range::new(Position::new(0, 0, 0), Position::new(0, 1, 1)).expect("range"),
    );
    let facts = (0..count)
        .map(|index| {
            let definition = SymbolDefinition::new(
                SymbolId::new(format!("symbol:{index}")).expect("symbol ID"),
                SymbolKind::Type,
                Name::new(format!("Type{index}")).expect("name"),
                location.clone(),
            )
            .with_qualified_name(
                QualifiedName::new(format!("benchmark.Type{index}")).expect("qualified name"),
            );
            SemanticFactRecord::new(
                FactId::new(format!("fact:{index}")).expect("fact ID"),
                SemanticFact::Definition(definition),
            )
        })
        .collect();
    SemanticFactSet::new(facts)
}

fn benchmark_fact_delta(c: &mut Criterion) {
    let previous = records(1_000);
    let current = records(1_000);
    let document = DocumentId::new("benchmark.java").expect("document ID");
    let revision = RevisionId::new("revision:benchmark").expect("revision ID");
    c.bench_function("fact_delta_1000_unchanged", |benchmark| {
        benchmark.iter(|| {
            black_box(FactDelta::between(
                document.clone(),
                revision.clone(),
                Some(black_box(&previous)),
                black_box(&current),
            ))
        });
    });
}

criterion_group!(benches, benchmark_fact_delta);
criterion_main!(benches);

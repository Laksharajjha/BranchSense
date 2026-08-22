#![allow(missing_docs)]

use std::fmt::Write;

use branchsense_extractor_java::JavaExtractor;
use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn large_source() -> String {
    let mut source = String::from("package benchmark;\npublic class LargeFixture {\n");
    for index in 0..500 {
        writeln!(source, "    private String field{index};").expect("String writes cannot fail");
        writeln!(source, "    public void method{index}(String value) {{}}")
            .expect("String writes cannot fail");
    }
    source.push_str("}\n");
    source
}

fn extract_large_java(c: &mut Criterion) {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let extractor = JavaExtractor::new();
    let source = large_source();
    let parsed = parser
        .parse_source(ParseInput::new("LargeFixture.java", source, DocumentVersion::default()))
        .expect("large source parses")
        .into_document();

    c.bench_function("java_extract_large_source", |benchmark| {
        benchmark.iter(|| black_box(extractor.extract(black_box(&parsed)).expect("extracts")));
    });
}

criterion_group!(benches, extract_large_java);
criterion_main!(benches);

#![allow(missing_docs)]

use std::fmt::Write;

use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn large_source() -> String {
    let mut source = String::from("public class LargeFixture {\n");
    for index in 0..1_000 {
        writeln!(source, "    int method{index}() {{ return {index}; }}")
            .expect("writing to a String cannot fail");
    }
    source.push_str("}\n");
    source
}

fn parse_large_java(c: &mut Criterion) {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("grammar loads");
    let source = large_source();
    c.bench_function("java_parse_large_source", |benchmark| {
        benchmark.iter(|| {
            let input = ParseInput::new(
                "LargeFixture.java",
                black_box(source.clone()),
                DocumentVersion::initial(),
            );
            black_box(parser.parse_source(input).expect("large fixture parses"));
        });
    });
}

criterion_group!(benches, parse_large_java);
criterion_main!(benches);

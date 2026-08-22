#![allow(missing_docs)]

use std::sync::Arc;

use branchsense_core::Language;
use branchsense_parser::{Parser, ParserConfiguration, ParserRegistry};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

#[derive(Debug)]
struct RegistryParser;

impl Parser for RegistryParser {
    fn language(&self) -> Language {
        Language::Java
    }
    fn configuration(&self) -> &ParserConfiguration {
        static CONFIG: std::sync::OnceLock<ParserConfiguration> = std::sync::OnceLock::new();
        CONFIG.get_or_init(ParserConfiguration::default)
    }
    fn parse_source(
        &self,
        _input: branchsense_parser::ParseInput,
    ) -> Result<branchsense_parser::ParseResult, branchsense_parser::ParseError> {
        unreachable!("benchmark does not parse")
    }
}

fn registry_lookup(c: &mut Criterion) {
    let registry = ParserRegistry::new(ParserConfiguration::default());
    registry.register(Arc::new(RegistryParser)).expect("registration succeeds");
    c.bench_function("parser_registry_lookup", |benchmark| {
        benchmark.iter(|| black_box(registry.get(Language::Java).expect("parser is registered")));
    });
}

criterion_group!(benches, registry_lookup);
criterion_main!(benches);

#![allow(missing_docs)]

use branchsense_core::Language;
use branchsense_language::{
    ADAPTER_API_VERSION, AdapterConfig, AdapterError, AdapterMetadata, AdapterRegistry,
    AdapterSession, Capabilities, LanguageAdapter, Version, VersionRange,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

struct BenchAdapter {
    metadata: AdapterMetadata,
}

impl LanguageAdapter for BenchAdapter {
    fn metadata(&self) -> &AdapterMetadata {
        &self.metadata
    }
    fn start(
        &self,
        _configuration: &AdapterConfig,
    ) -> Result<Arc<dyn AdapterSession>, AdapterError> {
        unreachable!("benchmark only looks up metadata")
    }
}

fn adapter_lookup(c: &mut Criterion) {
    let registry = AdapterRegistry::default();
    registry
        .register(BenchAdapter {
            metadata: AdapterMetadata::new(
                "bench",
                Language::Java,
                Version::new(1, 0, 0),
                VersionRange::from(ADAPTER_API_VERSION),
                "benchmark",
                Capabilities::empty(),
            ),
        })
        .expect("registration succeeds");
    c.bench_function("adapter_registry_lookup", |benchmark| {
        benchmark.iter(|| black_box(registry.adapter(Language::Java).expect("adapter exists")));
    });
}

criterion_group!(benches, adapter_lookup);
criterion_main!(benches);

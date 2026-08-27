//! Bridge from lifecycle-aware language adapters to parser registration.

use std::sync::Arc;

use branchsense_parser::{
    LanguageAdapter as ParserAdapter, ParseError, Parser, ParserConfiguration,
};

use crate::{AdapterConfig, LanguageAdapter};

/// Adapts a lifecycle-aware language adapter to the parser registry contract.
///
/// The bridge keeps the parser registry independent from this crate while
/// allowing hosts to register one canonical language adapter implementation.
#[derive(Clone)]
pub struct ParserAdapterBridge {
    adapter: Arc<dyn LanguageAdapter>,
}

impl std::fmt::Debug for ParserAdapterBridge {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ParserAdapterBridge")
            .field("language", &self.adapter.language())
            .finish()
    }
}

impl ParserAdapterBridge {
    /// Creates a parser-registry bridge for an adapter.
    #[must_use]
    pub fn new(adapter: Arc<dyn LanguageAdapter>) -> Self {
        Self { adapter }
    }
}

impl ParserAdapter for ParserAdapterBridge {
    fn language(&self) -> branchsense_core::Language {
        self.adapter.language()
    }

    fn create_parser(
        &self,
        configuration: &ParserConfiguration,
    ) -> Result<Arc<dyn Parser>, ParseError> {
        self.adapter
            .start(&AdapterConfig::new().with_parser_configuration(configuration.clone()))
            .map(|session| session.parser())
            .map_err(|error| ParseError::Adapter { message: error.to_string() })
    }
}

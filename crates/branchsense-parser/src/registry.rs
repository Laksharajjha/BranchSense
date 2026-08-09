//! Thread-safe, instance-owned parser registry.

#![allow(clippy::missing_errors_doc)]

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use branchsense_core::Language;

use crate::{
    configuration::ParserConfiguration,
    error::{ParseError, RegistryError},
    parser::{LanguageAdapter, Parser},
};

/// A cloneable registry of language parser implementations.
#[derive(Clone)]
pub struct ParserRegistry {
    configuration: ParserConfiguration,
    parsers: Arc<RwLock<HashMap<Language, Arc<dyn Parser>>>>,
}

impl fmt::Debug for ParserRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ParserRegistry")
            .field("configuration", &self.configuration)
            .field("parser_count", &self.len())
            .finish_non_exhaustive()
    }
}

impl ParserRegistry {
    /// Creates an empty registry with shared immutable configuration.
    #[must_use]
    pub fn new(configuration: ParserConfiguration) -> Self {
        Self { configuration, parsers: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Returns the configuration used for adapter construction.
    #[must_use]
    pub fn configuration(&self) -> &ParserConfiguration {
        &self.configuration
    }

    /// Registers an already-created parser.
    pub fn register(&self, parser: Arc<dyn Parser>) -> Result<(), RegistryError> {
        let language = parser.language();
        let mut parsers = self.parsers.write().map_err(|_| {
            RegistryError::Adapter(ParseError::Adapter {
                message: "parser registry lock poisoned".into(),
            })
        })?;
        if parsers.contains_key(&language) {
            return Err(RegistryError::AlreadyRegistered(language));
        }
        parsers.insert(language, parser);
        Ok(())
    }

    /// Constructs and registers a parser from a language adapter.
    pub fn register_adapter(&self, adapter: &dyn LanguageAdapter) -> Result<(), RegistryError> {
        let parser = adapter.create_parser(&self.configuration).map_err(RegistryError::Adapter)?;
        self.register(parser)
    }

    /// Looks up a parser by language.
    pub fn get(&self, language: Language) -> Result<Arc<dyn Parser>, RegistryError> {
        let parsers = self.parsers.read().map_err(|_| {
            RegistryError::Adapter(ParseError::Adapter {
                message: "parser registry lock poisoned".into(),
            })
        })?;
        parsers.get(&language).cloned().ok_or(RegistryError::NotRegistered(language))
    }

    /// Returns the number of registered language parsers.
    pub fn len(&self) -> usize {
        self.parsers.read().map_or(0, |parsers| parsers.len())
    }

    /// Returns whether no parsers are registered.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

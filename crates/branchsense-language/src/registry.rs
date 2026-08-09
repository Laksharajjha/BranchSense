//! Thread-safe instance-owned language adapter registry.

#![allow(clippy::missing_errors_doc)]

use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use branchsense_core::Language;

use crate::{
    adapter::LanguageAdapter,
    error::RegistryError,
    version::{ADAPTER_API_VERSION, Version},
};

/// Registry of language adapters with framework-version validation.
#[derive(Clone)]
pub struct AdapterRegistry {
    host_api: Version,
    adapters: Arc<RwLock<HashMap<Language, Arc<dyn LanguageAdapter>>>>,
}

impl fmt::Debug for AdapterRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AdapterRegistry")
            .field("host_api", &self.host_api)
            .field("adapter_count", &self.len())
            .finish_non_exhaustive()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new(ADAPTER_API_VERSION)
    }
}

impl AdapterRegistry {
    /// Creates an empty registry for one framework API version.
    #[must_use]
    pub fn new(host_api: Version) -> Self {
        Self { host_api, adapters: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Returns the framework API version used by this registry.
    #[must_use]
    pub const fn host_api(&self) -> Version {
        self.host_api
    }

    /// Registers an adapter value.
    pub fn register<A: LanguageAdapter + 'static>(&self, adapter: A) -> Result<(), RegistryError> {
        self.register_shared(Arc::new(adapter))
    }

    /// Registers a shared adapter instance.
    pub fn register_shared(&self, adapter: Arc<dyn LanguageAdapter>) -> Result<(), RegistryError> {
        let language = adapter.language();
        if !adapter.metadata().api_compatibility().contains(self.host_api) {
            return Err(RegistryError::IncompatibleVersion { language, host: self.host_api });
        }
        let mut adapters = self.adapters.write().map_err(|_| {
            RegistryError::Adapter(crate::AdapterError::Lifecycle {
                message: "adapter registry lock poisoned".into(),
            })
        })?;
        if adapters.contains_key(&language) {
            return Err(RegistryError::AlreadyRegistered(language));
        }
        adapters.insert(language, adapter);
        Ok(())
    }

    /// Returns a registered adapter by language.
    pub fn adapter(&self, language: Language) -> Result<Arc<dyn LanguageAdapter>, RegistryError> {
        let adapters = self.adapters.read().map_err(|_| {
            RegistryError::Adapter(crate::AdapterError::Lifecycle {
                message: "adapter registry lock poisoned".into(),
            })
        })?;
        adapters.get(&language).cloned().ok_or(RegistryError::NotRegistered(language))
    }

    /// Returns the number of registered adapters.
    pub fn len(&self) -> usize {
        self.adapters.read().map_or(0, |adapters| adapters.len())
    }

    /// Returns whether the registry has no adapters.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

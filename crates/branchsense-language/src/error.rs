//! Adapter framework error contracts.

use thiserror::Error;

use crate::{capabilities::Capabilities, version::Version};
use branchsense_core::Language;

/// Errors raised while starting or negotiating an adapter.
#[derive(Debug, Error)]
pub enum AdapterError {
    /// The adapter does not provide all required host capabilities.
    #[error("adapter is missing required capabilities: {missing:?}")]
    MissingCapabilities {
        /// Capabilities required by the host but unavailable from the adapter.
        missing: Capabilities,
    },
    /// The adapter cannot run with this framework API version.
    #[error("adapter API range {supported:?} does not contain host version {host}")]
    IncompatibleApi {
        /// Framework versions supported by the adapter.
        supported: crate::VersionRange,
        /// Framework version requested by the host.
        host: Version,
    },
    /// The adapter failed during lifecycle startup or shutdown.
    #[error("adapter lifecycle failure: {message}")]
    Lifecycle {
        /// Lifecycle failure context.
        message: String,
    },
}

/// Errors raised by adapter registration and lookup.
#[derive(Debug, Error)]
pub enum RegistryError {
    /// No adapter has been registered for a language.
    #[error("no adapter registered for {0:?}")]
    NotRegistered(Language),
    /// A language already has an adapter.
    #[error("an adapter is already registered for {0:?}")]
    AlreadyRegistered(Language),
    /// An adapter is incompatible with this registry's framework version.
    #[error("adapter {language:?} is incompatible with framework version {host}")]
    IncompatibleVersion {
        /// Language of the incompatible adapter.
        language: Language,
        /// Framework version used by the registry.
        host: Version,
    },
    /// Adapter startup or feature negotiation failed.
    #[error("adapter registration failed: {0}")]
    Adapter(#[source] AdapterError),
}

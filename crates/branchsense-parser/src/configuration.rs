//! Parser configuration shared by all language adapters.

use serde::{Deserialize, Serialize};

/// Immutable limits and behavior switches supplied to a parser adapter.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParserConfiguration {
    max_source_bytes: usize,
    max_diagnostics: usize,
    incremental_enabled: bool,
    retain_source: bool,
}

impl Default for ParserConfiguration {
    fn default() -> Self {
        Self {
            max_source_bytes: 16 * 1024 * 1024,
            max_diagnostics: 256,
            incremental_enabled: true,
            retain_source: true,
        }
    }
}

impl ParserConfiguration {
    /// Creates the default production configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum accepted source size in bytes.
    #[must_use]
    pub fn with_max_source_bytes(mut self, limit: usize) -> Self {
        self.max_source_bytes = limit;
        self
    }

    /// Sets the maximum diagnostics retained in a parse result.
    #[must_use]
    pub fn with_max_diagnostics(mut self, limit: usize) -> Self {
        self.max_diagnostics = limit;
        self
    }

    /// Enables or disables incremental parsing support.
    #[must_use]
    pub fn with_incremental_enabled(mut self, enabled: bool) -> Self {
        self.incremental_enabled = enabled;
        self
    }

    /// Enables or disables retaining source text in parsed documents.
    #[must_use]
    pub fn with_retain_source(mut self, retain: bool) -> Self {
        self.retain_source = retain;
        self
    }

    /// Returns the maximum accepted source size.
    #[must_use]
    pub const fn max_source_bytes(&self) -> usize {
        self.max_source_bytes
    }

    /// Returns the diagnostic retention limit.
    #[must_use]
    pub const fn max_diagnostics(&self) -> usize {
        self.max_diagnostics
    }

    /// Returns whether incremental parsing is enabled.
    #[must_use]
    pub const fn incremental_enabled(&self) -> bool {
        self.incremental_enabled
    }

    /// Returns whether parsed documents retain source text.
    #[must_use]
    pub const fn retain_source(&self) -> bool {
        self.retain_source
    }
}

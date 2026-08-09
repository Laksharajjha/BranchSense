//! Adapter session lifecycle contracts.

#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use branchsense_parser::Parser;

use crate::{capabilities::NegotiatedFeatures, error::AdapterError};

/// Lifecycle state of a running adapter session.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AdapterState {
    /// Session is accepting parser work.
    Running,
    /// Session has released parser resources.
    Stopped,
}

/// Runtime session created by a language adapter.
pub trait AdapterSession: Send + Sync {
    /// Returns the parser implementation owned by this session.
    fn parser(&self) -> Arc<dyn Parser>;

    /// Returns the negotiated features for this session.
    fn features(&self) -> NegotiatedFeatures;

    /// Returns the current lifecycle state.
    fn state(&self) -> AdapterState;

    /// Stops the session and releases adapter-owned resources.
    fn shutdown(&mut self) -> Result<(), AdapterError>;
}

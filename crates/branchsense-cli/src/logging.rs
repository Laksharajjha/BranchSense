//! Process-wide structured logging setup.

use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::error::{CliError, Result};

/// Installs the process-wide tracing subscriber.
///
/// `RUST_LOG` controls filtering. `BranchSense` defaults to `info` and writes
/// diagnostics to stderr so command output remains machine-readable.
pub fn init() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false).with_writer(std::io::stderr))
        .try_init()
        .map_err(|error| CliError::Logging(error.to_string()))
}

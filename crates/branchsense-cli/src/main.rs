//! `BranchSense` command-line entry point.

#![forbid(unsafe_code)]

mod cli;
mod error;
mod logging;

use clap::Parser;

use crate::{cli::Cli, error::Result};

/// Starts the `BranchSense` command-line application.
fn main() -> Result<()> {
    logging::init()?;
    Cli::parse().run()
}

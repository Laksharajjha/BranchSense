//! Command-line argument parsing and command dispatch.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use branchsense_core::{BuildInfo, Language};
use branchsense_java::{JavaAdapter, JavaSyntaxTree};
use branchsense_language::{AdapterConfig, AdapterRegistry};
use clap::{Parser as ClapParser, Subcommand};
use tracing::debug;

use crate::error::{CliError, Result};

/// `BranchSense` is a local semantic intelligence engine for Git repositories.
#[derive(Debug, ClapParser)]
#[command(name = "branchsense", version, about, long_about = None)]
pub struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Print `BranchSense` version information.
    Version,
    /// Parse one Java source file.
    Parse {
        /// Java source file to parse.
        path: PathBuf,
    },
}

impl Cli {
    /// Executes the selected command.
    pub fn run(self) -> Result<()> {
        match self.command {
            Command::Version => {
                let build_info = BuildInfo::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                debug!(version = build_info.version(), "printing BranchSense version");
                println!("{build_info}");
            }
            Command::Parse { path } => parse_java(&path)?,
        }
        Ok(())
    }
}

fn parse_java(path: &Path) -> Result<()> {
    let registry = AdapterRegistry::default();
    registry
        .register(JavaAdapter::default())
        .map_err(|error| CliError::Command(error.to_string()))?;
    let adapter =
        registry.adapter(Language::Java).map_err(|error| CliError::Command(error.to_string()))?;
    let session = adapter
        .start(&AdapterConfig::default())
        .map_err(|error| CliError::Command(error.to_string()))?;
    let parser = session.parser();
    let started = Instant::now();
    let result = parser.parse(path).map_err(|error| CliError::Command(error.to_string()))?;
    let elapsed = started.elapsed();
    let tree =
        result.document().syntax_tree().as_any().downcast_ref::<JavaSyntaxTree>().ok_or_else(
            || CliError::Command("Java adapter returned an unexpected syntax tree".into()),
        )?;
    let statistics = tree.statistics();

    println!("Language: {:?}", result.document().language());
    println!("Parse success: {}", !result.has_errors());
    println!("Node count: {}", statistics.node_count());
    println!("Depth: {}", statistics.depth());
    println!("Elapsed time: {elapsed:?}");
    println!("Diagnostics: {}", result.diagnostics().len());
    debug!(path = %path.display(), elapsed = ?elapsed, "parsed Java source");
    Ok(())
}

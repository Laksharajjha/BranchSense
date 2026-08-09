//! Command-line argument parsing and command dispatch.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use branchsense_core::{BuildInfo, Language};
use branchsense_extractor_java::JavaExtractor;
use branchsense_java::{JavaAdapter, JavaSyntaxTree};
use branchsense_language::{AdapterConfig, AdapterRegistry};
use branchsense_semantic::SemanticFact;
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
    /// Inspect semantic facts extracted from one Java source file.
    Inspect {
        /// Java source file to inspect.
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
            Command::Inspect { path } => inspect_java(&path)?,
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

    if result.has_errors() {
        println!("✗ File parsed with syntax errors");
    } else {
        println!("✓ File parsed successfully");
    }
    println!();
    println!("Language: {:?}", result.document().language());
    println!();
    println!("Tree Statistics");
    println!();
    println!("- Total nodes: {}", statistics.node_count());
    println!("- Tree depth: {}", statistics.depth());
    println!("- Parse duration: {elapsed:?}");
    println!("- Syntax errors: {}", result.diagnostics().len());
    debug!(path = %path.display(), elapsed = ?elapsed, "parsed Java source");
    Ok(())
}

fn inspect_java(path: &Path) -> Result<()> {
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
    let parsed = parser.parse(path).map_err(|error| CliError::Command(error.to_string()))?;
    let started = Instant::now();
    let result = JavaExtractor::new()
        .extract(parsed.document())
        .map_err(|error| CliError::Command(error.to_string()))?;
    let elapsed = started.elapsed();
    let mut package_count = 0;
    let mut type_count = 0;
    let mut method_count = 0;
    let mut field_count = 0;
    let mut relationship_count = 0;
    for record in result.facts().facts() {
        match record.fact() {
            SemanticFact::Definition(definition) => match definition.kind() {
                branchsense_semantic::SymbolKind::Package => package_count += 1,
                branchsense_semantic::SymbolKind::Type
                | branchsense_semantic::SymbolKind::Interface
                | branchsense_semantic::SymbolKind::Enum => type_count += 1,
                branchsense_semantic::SymbolKind::Method
                | branchsense_semantic::SymbolKind::Constructor => method_count += 1,
                branchsense_semantic::SymbolKind::Field => field_count += 1,
                _ => {}
            },
            SemanticFact::Contains(_)
            | SemanticFact::Call(_)
            | SemanticFact::Reference(_)
            | SemanticFact::TypeRelation(_)
            | SemanticFact::Dependency(_) => relationship_count += 1,
            _ => {}
        }
    }

    println!("Package: {package_count}");
    println!("Types: {type_count}");
    println!("Methods: {method_count}");
    println!("Fields: {field_count}");
    println!("Relationships: {relationship_count}");
    println!("Fact Count: {}", result.facts().len());
    println!("Extraction Time: {elapsed:?}");
    println!("Syntax Diagnostics: {}", parsed.diagnostics().len());
    println!("Extraction Diagnostics: {}", result.diagnostics().len());
    debug!(path = %path.display(), elapsed = ?elapsed, facts = result.facts().len(), "inspected Java semantic facts");
    Ok(())
}

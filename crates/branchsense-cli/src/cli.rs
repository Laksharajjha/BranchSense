//! Command-line argument parsing and command dispatch.

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

use branchsense_collision::CollisionAnalyzer;
use branchsense_core::{BuildInfo, DocumentId, Language, QualifiedName, RevisionId};
use branchsense_diff::SemanticDiffer;
use branchsense_extractor_java::JavaExtractor;
use branchsense_git::{GitRepository, GitSnapshotIndexer, MergeBaseResult};
use branchsense_graph::{EdgeKind, SemanticGraph};
use branchsense_impact::ImpactAnalyzer;
use branchsense_index::{IndexOptions, RepositoryIndex};
use branchsense_java::{JavaAdapter, JavaSyntaxTree};
use branchsense_language::{AdapterConfig, AdapterRegistry};
use branchsense_overlap::SemanticOverlapAnalyzer;
use branchsense_query::{Query, QueryNode, QueryOptions, QueryResult, RelationshipResult};
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
        /// Also construct and display the semantic graph.
        #[arg(long)]
        graph: bool,
    },
    /// Index all Java sources below a repository or project path.
    Index {
        /// Repository or project path to index.
        path: PathBuf,
    },
    /// Inspect Git state without modifying the repository.
    Git {
        #[command(subcommand)]
        command: GitCommand,
    },
    /// Compare semantic snapshots at two Git revisions.
    Diff {
        /// Repository path.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Earlier revision, branch, or ref.
        #[arg(long)]
        before: String,
        /// Later revision, branch, or ref.
        #[arg(long)]
        after: String,
    },
    /// Analyze symbols affected by semantic changes between two revisions.
    Impact {
        /// Repository path.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Earlier revision, branch, or ref.
        #[arg(long)]
        before: String,
        /// Later revision, branch, or ref.
        #[arg(long)]
        after: String,
    },
    /// Analyze semantic overlap between two branches from their common base.
    Overlap {
        /// Repository path.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Expected common-base reference.
        #[arg(long)]
        base: String,
        /// First branch, revision, or ref.
        #[arg(long)]
        branch_a: String,
        /// Second branch, revision, or ref.
        #[arg(long)]
        branch_b: String,
    },
    /// Assess deterministic semantic collision evidence between two branches.
    Analyze {
        /// Repository path.
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        /// Expected common-base reference.
        #[arg(long)]
        base: String,
        /// First branch, revision, or ref.
        #[arg(long)]
        branch_a: String,
        /// Second branch, revision, or ref.
        #[arg(long)]
        branch_b: String,
    },
    /// Query callers of a symbol in one Java source graph.
    Callers {
        /// Fully qualified symbol name.
        symbol: String,
        /// Java source file used to build the graph.
        #[arg(long, conflicts_with = "project", required_unless_present = "project")]
        file: Option<PathBuf>,
        /// Repository or project path to index.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        project: Option<PathBuf>,
    },
    /// Query callees of a symbol in one Java source graph.
    Callees {
        /// Fully qualified symbol name.
        symbol: String,
        /// Java source file used to build the graph.
        #[arg(long, conflicts_with = "project", required_unless_present = "project")]
        file: Option<PathBuf>,
        /// Repository or project path to index.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        project: Option<PathBuf>,
    },
    /// Query references to a symbol in one Java source graph.
    References {
        /// Fully qualified symbol name.
        symbol: String,
        /// Java source file used to build the graph.
        #[arg(long, conflicts_with = "project", required_unless_present = "project")]
        file: Option<PathBuf>,
        /// Repository or project path to index.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        project: Option<PathBuf>,
    },
    /// Query implementations of a type in one Java source graph.
    Implementations {
        /// Fully qualified symbol name.
        symbol: String,
        /// Java source file used to build the graph.
        #[arg(long, conflicts_with = "project", required_unless_present = "project")]
        file: Option<PathBuf>,
        /// Repository or project path to index.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        project: Option<PathBuf>,
    },
    /// Query dependencies of a symbol in one Java source graph.
    Dependencies {
        /// Fully qualified symbol name.
        symbol: String,
        /// Java source file used to build the graph.
        #[arg(long, conflicts_with = "project", required_unless_present = "project")]
        file: Option<PathBuf>,
        /// Repository or project path to index.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        project: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum GitCommand {
    /// Print repository identity and HEAD.
    Info { path: PathBuf },
    /// List local and remote branches.
    Branches { path: PathBuf },
    /// List tags and other refs exposed by the repository.
    Refs { path: PathBuf },
    /// Print the deterministic merge base of two revisions.
    MergeBase {
        branch_a: String,
        branch_b: String,
        #[arg(long, default_value = ".")]
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
            Command::Inspect { path, graph } => inspect_java(&path, graph)?,
            Command::Index { path } => index_repository(&path)?,
            Command::Git { command } => run_git_command(command)?,
            Command::Diff { repo, before, after } => diff_git_revisions(&repo, &before, &after)?,
            Command::Impact { repo, before, after } => {
                impact_git_revisions(&repo, &before, &after)?;
            }
            Command::Overlap { repo, base, branch_a, branch_b } => {
                overlap_git_revisions(&repo, &base, &branch_a, &branch_b)?;
            }
            Command::Analyze { repo, base, branch_a, branch_b } => {
                analyze_git_revisions(&repo, &base, &branch_a, &branch_b)?;
            }
            Command::Callers { symbol, file, project } => {
                query_java(file.as_deref(), project.as_deref(), &symbol, QueryOperation::Callers)?;
            }
            Command::Callees { symbol, file, project } => {
                query_java(file.as_deref(), project.as_deref(), &symbol, QueryOperation::Callees)?;
            }
            Command::References { symbol, file, project } => {
                query_java(
                    file.as_deref(),
                    project.as_deref(),
                    &symbol,
                    QueryOperation::References,
                )?;
            }
            Command::Implementations { symbol, file, project } => {
                query_java(
                    file.as_deref(),
                    project.as_deref(),
                    &symbol,
                    QueryOperation::Implementations,
                )?;
            }
            Command::Dependencies { symbol, file, project } => {
                query_java(
                    file.as_deref(),
                    project.as_deref(),
                    &symbol,
                    QueryOperation::Dependencies,
                )?;
            }
        }
        Ok(())
    }
}

fn run_git_command(command: GitCommand) -> Result<()> {
    match command {
        GitCommand::Info { path } => {
            let repository = GitRepository::discover(path)
                .map_err(|error| CliError::Command(error.to_string()))?;
            let head = repository.head().map_err(|error| CliError::Command(error.to_string()))?;
            println!("Repository ID: {}", repository.identity().id());
            println!("Git directory: {}", repository.identity().git_dir().display());
            if let Some(worktree) = repository.identity().worktree() {
                println!("Working tree: {}", worktree.display());
            }
            println!("HEAD: {}", head.commit_id());
            Ok(())
        }
        GitCommand::Branches { path } => {
            let repository = GitRepository::discover(path)
                .map_err(|error| CliError::Command(error.to_string()))?;
            for reference in
                repository.local_branches().map_err(|error| CliError::Command(error.to_string()))?
            {
                println!("{} {}", reference.name(), reference.target());
            }
            for reference in repository
                .remote_branches()
                .map_err(|error| CliError::Command(error.to_string()))?
            {
                println!("{} {}", reference.name(), reference.target());
            }
            Ok(())
        }
        GitCommand::Refs { path } => {
            let repository = GitRepository::discover(path)
                .map_err(|error| CliError::Command(error.to_string()))?;
            for references in
                [repository.local_branches(), repository.remote_branches(), repository.tags()]
            {
                for reference in references.map_err(|error| CliError::Command(error.to_string()))? {
                    println!("{:?} {} {}", reference.kind(), reference.name(), reference.target());
                }
            }
            Ok(())
        }
        GitCommand::MergeBase { branch_a, branch_b, path } => {
            let repository = GitRepository::discover(path)
                .map_err(|error| CliError::Command(error.to_string()))?;
            let left = repository
                .resolve(&branch_a)
                .map_err(|error| CliError::Command(error.to_string()))?;
            let right = repository
                .resolve(&branch_b)
                .map_err(|error| CliError::Command(error.to_string()))?;
            match repository
                .merge_bases(&left, &right)
                .map_err(|error| CliError::Command(error.to_string()))?
            {
                MergeBaseResult::None => println!("No common ancestor"),
                MergeBaseResult::Single(base) => println!("{}", base.commit_id()),
                MergeBaseResult::Multiple(bases) => {
                    println!("Multiple merge bases:");
                    for base in bases {
                        println!("{}", base.commit_id());
                    }
                }
            }
            Ok(())
        }
    }
}

fn diff_git_revisions(repo_path: &Path, before: &str, after: &str) -> Result<()> {
    let repository =
        GitRepository::discover(repo_path).map_err(|error| CliError::Command(error.to_string()))?;
    let before_revision =
        repository.resolve(before).map_err(|error| CliError::Command(error.to_string()))?;
    let after_revision =
        repository.resolve(after).map_err(|error| CliError::Command(error.to_string()))?;
    let indexer = GitSnapshotIndexer::default();
    let before_snapshot = indexer
        .index_revision(&repository, &before_revision, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let after_snapshot = indexer
        .index_revision(&repository, &after_revision, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let diff = SemanticDiffer::new().diff_git(&before_snapshot, &after_snapshot);
    println!("Before: {}", before_revision.commit_id());
    println!("After: {}", after_revision.commit_id());
    println!(
        "Documents changed: {}",
        diff.statistics().documents_added()
            + diff.statistics().documents_removed()
            + diff.statistics().documents_modified()
    );
    println!(
        "Symbols changed: {}",
        diff.statistics().symbols_added()
            + diff.statistics().symbols_removed()
            + diff.statistics().symbols_modified()
    );
    println!(
        "Facts changed: {}",
        diff.statistics().facts_added()
            + diff.statistics().facts_removed()
            + diff.statistics().facts_modified()
    );
    println!(
        "Relationships changed: {}",
        diff.statistics().relationships_added()
            + diff.statistics().relationships_removed()
            + diff.statistics().relationships_modified()
    );
    Ok(())
}

fn impact_git_revisions(repo_path: &Path, before: &str, after: &str) -> Result<()> {
    let repository =
        GitRepository::discover(repo_path).map_err(|error| CliError::Command(error.to_string()))?;
    let before_revision =
        repository.resolve(before).map_err(|error| CliError::Command(error.to_string()))?;
    let after_revision =
        repository.resolve(after).map_err(|error| CliError::Command(error.to_string()))?;
    let indexer = GitSnapshotIndexer::default();
    let before_snapshot = indexer
        .index_revision(&repository, &before_revision, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let after_snapshot = indexer
        .index_revision(&repository, &after_revision, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let diff = SemanticDiffer::new().diff_git(&before_snapshot, &after_snapshot);
    let impacts =
        ImpactAnalyzer::new().analyze(&diff, before_snapshot.semantic(), after_snapshot.semantic());
    println!("Semantic Impact Analysis");
    println!();
    println!("Changed symbols: {}", impacts.statistics().changed_symbols());
    println!("Impacted symbols: {}", impacts.statistics().impacted_symbols());
    println!("Depth: {}", impacts.statistics().max_depth());
    println!("Truncated: {}", impacts.statistics().truncated());
    println!();
    for entry in impacts.entries() {
        println!(
            "{}",
            impact_symbol_name(&after_snapshot, &before_snapshot, entry.impacted_symbol())
        );
        for cause in entry.causes() {
            let explanation = cause.explanation();
            println!(
                "  └─ {:?} {} (depth {})",
                explanation.kind(),
                impact_symbol_name(&after_snapshot, &before_snapshot, explanation.changed_symbol(),),
                explanation.depth()
            );
        }
    }
    Ok(())
}

fn impact_symbol_name(
    after: &branchsense_git::GitSemanticSnapshot,
    before: &branchsense_git::GitSemanticSnapshot,
    id: &branchsense_core::SymbolId,
) -> String {
    after
        .semantic()
        .graph()
        .find_symbol(id)
        .or_else(|| before.semantic().graph().find_symbol(id))
        .and_then(branchsense_graph::GraphNode::definition)
        .and_then(|definition| definition.qualified_name().map(ToString::to_string))
        .unwrap_or_else(|| id.to_string())
}

fn overlap_git_revisions(
    repo_path: &Path,
    base: &str,
    branch_a: &str,
    branch_b: &str,
) -> Result<()> {
    let repository =
        GitRepository::discover(repo_path).map_err(|error| CliError::Command(error.to_string()))?;
    let requested_base =
        repository.resolve(base).map_err(|error| CliError::Command(error.to_string()))?;
    let revision_a =
        repository.resolve(branch_a).map_err(|error| CliError::Command(error.to_string()))?;
    let revision_b =
        repository.resolve(branch_b).map_err(|error| CliError::Command(error.to_string()))?;
    let merge_base = match repository
        .merge_bases(&revision_a, &revision_b)
        .map_err(|error| CliError::Command(error.to_string()))?
    {
        MergeBaseResult::None => {
            return Err(CliError::Command(format!(
                "branches `{branch_a}` and `{branch_b}` have no common ancestor"
            )));
        }
        MergeBaseResult::Multiple(_) => {
            return Err(CliError::Command(format!(
                "branches `{branch_a}` and `{branch_b}` have multiple merge bases"
            )));
        }
        MergeBaseResult::Single(base) => base,
    };
    if merge_base.commit_id() != requested_base.commit_id() {
        return Err(CliError::Command(format!(
            "`{base}` is not the common merge base; expected {}",
            merge_base.commit_id()
        )));
    }

    let indexer = GitSnapshotIndexer::default();
    let base_snapshot = indexer
        .index_revision(&repository, &merge_base, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let snapshot_a = indexer
        .index_revision(&repository, &revision_a, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let snapshot_b = indexer
        .index_revision(&repository, &revision_b, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let differ = SemanticDiffer::new();
    let diff_a = differ.diff_git(&base_snapshot, &snapshot_a);
    let diff_b = differ.diff_git(&base_snapshot, &snapshot_b);
    let impact_analyzer = ImpactAnalyzer::new();
    let impact_a =
        impact_analyzer.analyze(&diff_a, base_snapshot.semantic(), snapshot_a.semantic());
    let impact_b =
        impact_analyzer.analyze(&diff_b, base_snapshot.semantic(), snapshot_b.semantic());
    let overlaps = SemanticOverlapAnalyzer::new().analyze(&diff_a, &impact_a, &diff_b, &impact_b);

    println!("Semantic Branch Overlap");
    println!();
    println!("Base: {} ({base})", merge_base.commit_id());
    println!("Branch A: {branch_a} ({})", revision_a.commit_id());
    println!("Branch B: {branch_b} ({})", revision_b.commit_id());
    println!();
    println!("Changed symbols A: {}", overlaps.statistics().branch_a_changed());
    println!("Changed symbols B: {}", overlaps.statistics().branch_b_changed());
    println!("Overlaps: {}", overlaps.statistics().overlaps());
    println!("Truncated: {}", overlaps.statistics().truncated());
    println!();
    for entry in overlaps.entries() {
        let explanation = entry.explanation();
        println!("{:?}", explanation.kind());
        println!(
            "  A changed: {}",
            overlap_symbol_name(&snapshot_a, &base_snapshot, explanation.branch_a_changed())
        );
        println!(
            "  B changed: {}",
            overlap_symbol_name(&snapshot_b, &base_snapshot, explanation.branch_b_changed())
        );
        for target in explanation.targets() {
            println!("  Target: {}", overlap_symbol_name(&snapshot_a, &snapshot_b, target));
        }
        for evidence in explanation.branch_a_evidence() {
            println!("  A path: {:?}, depth {}", evidence.relationship(), evidence.depth());
        }
        for evidence in explanation.branch_b_evidence() {
            println!("  B path: {:?}, depth {}", evidence.relationship(), evidence.depth());
        }
        println!();
    }
    Ok(())
}

fn overlap_symbol_name(
    first: &branchsense_git::GitSemanticSnapshot,
    second: &branchsense_git::GitSemanticSnapshot,
    id: &branchsense_core::SymbolId,
) -> String {
    impact_symbol_name(first, second, id)
}

#[allow(clippy::too_many_lines)]
fn analyze_git_revisions(
    repo_path: &Path,
    base: &str,
    branch_a: &str,
    branch_b: &str,
) -> Result<()> {
    let repository =
        GitRepository::discover(repo_path).map_err(|error| CliError::Command(error.to_string()))?;
    let requested_base =
        repository.resolve(base).map_err(|error| CliError::Command(error.to_string()))?;
    let revision_a =
        repository.resolve(branch_a).map_err(|error| CliError::Command(error.to_string()))?;
    let revision_b =
        repository.resolve(branch_b).map_err(|error| CliError::Command(error.to_string()))?;
    let merge_base = match repository
        .merge_bases(&revision_a, &revision_b)
        .map_err(|error| CliError::Command(error.to_string()))?
    {
        MergeBaseResult::None => {
            return Err(CliError::Command(format!(
                "branches `{branch_a}` and `{branch_b}` have no common ancestor"
            )));
        }
        MergeBaseResult::Multiple(_) => {
            return Err(CliError::Command(format!(
                "branches `{branch_a}` and `{branch_b}` have multiple merge bases"
            )));
        }
        MergeBaseResult::Single(base) => base,
    };
    if merge_base.commit_id() != requested_base.commit_id() {
        return Err(CliError::Command(format!(
            "`{base}` is not the common merge base; expected {}",
            merge_base.commit_id()
        )));
    }

    let indexer = GitSnapshotIndexer::default();
    let base_snapshot = indexer
        .index_revision(&repository, &merge_base, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let snapshot_a = indexer
        .index_revision(&repository, &revision_a, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let snapshot_b = indexer
        .index_revision(&repository, &revision_b, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let differ = SemanticDiffer::new();
    let diff_a = differ.diff_git(&base_snapshot, &snapshot_a);
    let diff_b = differ.diff_git(&base_snapshot, &snapshot_b);
    let impact_analyzer = ImpactAnalyzer::new();
    let impact_a =
        impact_analyzer.analyze(&diff_a, base_snapshot.semantic(), snapshot_a.semantic());
    let impact_b =
        impact_analyzer.analyze(&diff_b, base_snapshot.semantic(), snapshot_b.semantic());
    let overlaps = branchsense_overlap::SemanticOverlapAnalyzer::new()
        .analyze(&diff_a, &impact_a, &diff_b, &impact_b);
    let assessment = CollisionAnalyzer::new().analyze(&overlaps);

    println!("BranchSense Semantic Analysis");
    println!();
    println!("Base: {}@{}", base, merge_base.commit_id());
    println!("Branch A: {}@{}", branch_a, revision_a.commit_id());
    println!("Branch B: {}@{}", branch_b, revision_b.commit_id());
    println!();
    println!("Changes:");
    println!("  Branch A: {} symbol(s)", overlaps.statistics().branch_a_changed());
    println!("  Branch B: {} symbol(s)", overlaps.statistics().branch_b_changed());
    println!("Semantic overlaps: {}", overlaps.statistics().overlaps());
    println!();
    println!("Assessment:");
    println!("  Severity: {:?}", assessment.severity());
    println!("  Evidence score: {} / 100", assessment.evidence_score());
    if assessment.is_empty() {
        println!("  No semantic collision detected.");
    } else {
        println!();
        println!("Factors:");
        for factor in assessment.factors() {
            println!("  - {:?} (strength {})", factor.kind(), factor.strength());
        }
        println!();
        println!("Explanations:");
        for explanation in assessment.explanations() {
            println!("  - {}", explanation.summary());
            println!(
                "    A: {}",
                overlap_symbol_name(&snapshot_a, &base_snapshot, explanation.branch_a_changed())
            );
            println!(
                "    B: {}",
                overlap_symbol_name(&snapshot_b, &base_snapshot, explanation.branch_b_changed())
            );
            for target in explanation.targets() {
                println!("    Target: {}", overlap_symbol_name(&snapshot_a, &snapshot_b, target));
            }
            if let Some(depth) = explanation.evidence().branch_a_depth() {
                println!("    A impact depth: {depth}");
            }
            if let Some(depth) = explanation.evidence().branch_b_depth() {
                println!("    B impact depth: {depth}");
            }
        }
    }
    println!();
    println!(
        "This is an assessment of semantic collision evidence, not a probability of merge failure."
    );
    Ok(())
}

#[derive(Clone, Copy)]
enum QueryOperation {
    Callers,
    Callees,
    References,
    Implementations,
    Dependencies,
}

fn index_repository(path: &Path) -> Result<()> {
    let result = RepositoryIndex::new(IndexOptions::default())
        .index(path, None)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let report = result.report();
    let statistics = result.snapshot().graph().statistics();
    println!("Repository");
    println!();
    println!("Root: {}", result.snapshot().repository().root().display());
    println!("Files discovered: {}", report.discovered());
    println!("Files indexed: {}", report.indexed());
    println!("Files unchanged: {}", report.unchanged());
    println!("Files skipped: {}", report.skipped());
    println!("Parse diagnostics: {}", report.parse_diagnostics());
    println!("Extraction diagnostics: {}", report.extraction_diagnostics());
    println!();
    println!("Semantic Graph");
    println!();
    println!("Documents: {}", statistics.documents());
    println!("Symbols: {}", statistics.symbols());
    println!("Nodes: {}", statistics.nodes());
    println!("Edges: {}", statistics.edges());
    println!();
    println!("Index duration: {:?}", report.duration());
    for diagnostic in report.diagnostics() {
        println!(
            "{} [{:?}] {}",
            diagnostic.path().display(),
            diagnostic.stage(),
            diagnostic.message()
        );
    }
    Ok(())
}

fn query_java(
    file: Option<&Path>,
    project: Option<&Path>,
    name: &str,
    operation: QueryOperation,
) -> Result<()> {
    let graph = if let Some(project) = project {
        RepositoryIndex::new(IndexOptions::default())
            .index(project, None)
            .map_err(|error| CliError::Command(error.to_string()))?
            .snapshot()
            .graph()
            .clone()
    } else {
        graph_for_java(file.expect("clap requires a file or project"))?
    };
    let query = Query::new(&graph);
    let qualified =
        QualifiedName::new(name).map_err(|error| CliError::Command(error.to_string()))?;
    let symbol = query
        .symbol_by_qualified_name(&qualified)
        .map_err(|error| CliError::Command(error.to_string()))?;
    let symbol_id = symbol.id().clone();
    let options = QueryOptions::new();
    let results = match operation {
        QueryOperation::Callers => query.callers(&symbol_id, options),
        QueryOperation::Callees => query.callees(&symbol_id, options),
        QueryOperation::References => query.references(&symbol_id, options),
        QueryOperation::Implementations => query.implementations(&symbol_id, options),
        QueryOperation::Dependencies => query.dependencies(&symbol_id, options),
    }
    .map_err(|error| CliError::Command(error.to_string()))?;
    let title = match operation {
        QueryOperation::Callers => "Callers of",
        QueryOperation::Callees => "Callees of",
        QueryOperation::References => "References to",
        QueryOperation::Implementations => "Implementations of",
        QueryOperation::Dependencies => "Dependencies of",
    };
    println!("{title} {name}");
    print_relationships(&results);
    Ok(())
}

fn graph_for_java(path: &Path) -> Result<SemanticGraph> {
    let registry = AdapterRegistry::default();
    registry
        .register(JavaAdapter::default())
        .map_err(|error| CliError::Command(error.to_string()))?;
    let adapter =
        registry.adapter(Language::Java).map_err(|error| CliError::Command(error.to_string()))?;
    let session = adapter
        .start(&AdapterConfig::default())
        .map_err(|error| CliError::Command(error.to_string()))?;
    let parsed =
        session.parser().parse(path).map_err(|error| CliError::Command(error.to_string()))?;
    let extracted = JavaExtractor::new()
        .extract(parsed.document())
        .map_err(|error| CliError::Command(error.to_string()))?;
    let document_id = DocumentId::new(path.display().to_string())
        .map_err(|error| CliError::Command(error.to_string()))?;
    SemanticGraph::from_document_facts(
        document_id,
        RevisionId::new(format!("document:{}", parsed.document().version().value()))
            .map_err(|error| CliError::Command(error.to_string()))?,
        extracted.facts().clone(),
    )
    .map_err(|error| CliError::Command(error.to_string()))
}

fn print_relationships(results: &QueryResult<RelationshipResult>) {
    for (index, result) in results.items().iter().enumerate() {
        println!("\n{}. {}", index + 1, display_node(result.source()));
        println!("   -> {} ({:?})", display_node(result.target()), result.kind());
        if let QueryNode::Symbol(symbol) = result.source() {
            let location = symbol.location();
            println!("   {}:{}", location.document_id(), location.range().start().line() + 1);
        }
        if let Some(resolution) = result.resolution() {
            println!("   Resolution: {resolution:?}");
        }
    }
    println!("\nTotal: {}", results.len());
}

fn display_node(node: &QueryNode) -> String {
    match node {
        QueryNode::Symbol(symbol) => {
            symbol.qualified_name().map_or_else(|| symbol.name().to_owned(), ToString::to_string)
        }
        QueryNode::External { name, .. } | QueryNode::Unresolved { name } => name.to_string(),
        QueryNode::Document(document) => document.to_string(),
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

fn inspect_java(path: &Path, show_graph: bool) -> Result<()> {
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
    if show_graph {
        let document_id = DocumentId::new(path.display().to_string())
            .map_err(|error| CliError::Command(error.to_string()))?;
        let graph = SemanticGraph::from_document_facts(
            document_id,
            RevisionId::new(format!("document:{}", parsed.document().version().value()))
                .map_err(|error| CliError::Command(error.to_string()))?,
            result.facts().clone(),
        )
        .map_err(|error| CliError::Command(error.to_string()))?;
        print_graph(&graph);
    }
    debug!(path = %path.display(), elapsed = ?elapsed, facts = result.facts().len(), "inspected Java semantic facts");
    Ok(())
}

fn print_graph(graph: &SemanticGraph) {
    let statistics = graph.statistics();
    let mut types = 0;
    let mut methods = 0;
    let mut fields = 0;
    for node in graph.nodes() {
        match node.symbol_kind() {
            Some(
                branchsense_semantic::SymbolKind::Type
                | branchsense_semantic::SymbolKind::Interface
                | branchsense_semantic::SymbolKind::Enum,
            ) => types += 1,
            Some(
                branchsense_semantic::SymbolKind::Method
                | branchsense_semantic::SymbolKind::Constructor,
            ) => methods += 1,
            Some(branchsense_semantic::SymbolKind::Field) => fields += 1,
            _ => {}
        }
    }
    let edge_count = |kind| graph.edges().filter(|edge| edge.kind() == kind).count();
    println!();
    println!("Semantic Graph");
    println!();
    println!("## Nodes");
    println!();
    println!("Documents: {}", statistics.documents());
    println!("Types: {types}");
    println!("Methods: {methods}");
    println!("Fields: {fields}");
    println!("External: {}", statistics.external());
    println!("Unresolved: {}", statistics.unresolved());
    println!();
    println!("## Edges");
    println!();
    println!("Defines: {}", edge_count(EdgeKind::Defines));
    println!("Contains: {}", edge_count(EdgeKind::Contains));
    println!("Calls: {}", edge_count(EdgeKind::Calls));
    println!("References: {}", edge_count(EdgeKind::References));
    println!("Imports: {}", edge_count(EdgeKind::Imports));
    println!("Extends: {}", edge_count(EdgeKind::Extends));
    println!("Implements: {}", edge_count(EdgeKind::Implements));
    println!("DependsOn: {}", edge_count(EdgeKind::DependsOn));
    println!("Total Nodes: {}", statistics.nodes());
    println!("Total Edges: {}", statistics.edges());
}

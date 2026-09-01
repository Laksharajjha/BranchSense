//! Repository-wide Java indexing pipeline.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use branchsense_core::{DocumentId, ProjectId, RepositoryId, RevisionId, WorkspaceId};
use branchsense_extractor_java::JavaExtractor;
use branchsense_graph::{GraphError, SemanticGraph};
use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use branchsense_semantic::{
    ContentHash, FactProvenance, ProducerIdentity, SemanticFactSet, SnapshotIdentity,
};
use serde::{Deserialize, Serialize};

use crate::{DiscoveryError, DiscoveryOptions, SourceDiscovery};

/// Configuration for one repository indexing pass.
#[derive(Clone, Debug, Default)]
pub struct IndexOptions {
    discovery: DiscoveryOptions,
    parser: ParserConfiguration,
}

impl IndexOptions {
    /// Creates default Java indexing options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Replaces source discovery options.
    #[must_use]
    pub fn with_discovery(mut self, discovery: DiscoveryOptions) -> Self {
        self.discovery = discovery;
        self
    }
    /// Replaces parser options.
    #[must_use]
    pub fn with_parser(mut self, parser: ParserConfiguration) -> Self {
        self.parser = parser;
        self
    }
    /// Returns source discovery options.
    #[must_use]
    pub const fn discovery(&self) -> &DiscoveryOptions {
        &self.discovery
    }
    /// Returns parser options.
    #[must_use]
    pub const fn parser(&self) -> &ParserConfiguration {
        &self.parser
    }
}

/// Stable identities for a path-scoped repository indexing context.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepositoryIdentity {
    root: PathBuf,
    repository_id: RepositoryId,
    workspace_id: WorkspaceId,
    project_id: ProjectId,
}

impl RepositoryIdentity {
    /// Creates an identity from an explicit repository, workspace, and
    /// project scope.
    pub fn new(
        root: impl Into<PathBuf>,
        repository_id: RepositoryId,
        workspace_id: WorkspaceId,
        project_id: ProjectId,
    ) -> Self {
        Self { root: root.into(), repository_id, workspace_id, project_id }
    }

    fn from_root(root: &Path) -> std::result::Result<Self, branchsense_core::ModelError> {
        let key = root.to_string_lossy();
        Ok(Self::new(
            root.to_path_buf(),
            RepositoryId::new(format!("repository:path:{key}"))?,
            WorkspaceId::new(format!("workspace:path:{key}"))?,
            ProjectId::new(format!("project:java:{key}"))?,
        ))
    }
    /// Returns the canonical repository root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Returns the repository identity.
    #[must_use]
    pub fn repository_id(&self) -> &RepositoryId {
        &self.repository_id
    }
    /// Returns the workspace identity.
    #[must_use]
    pub fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }
    /// Returns the Java project identity.
    #[must_use]
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }
}

/// A successfully indexed document and its content identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedDocument {
    relative_path: PathBuf,
    content_hash: ContentHash,
    facts: SemanticFactSet,
}

impl IndexedDocument {
    /// Returns the repository-relative document path.
    #[must_use]
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }
    /// Returns the source content identity.
    #[must_use]
    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }
    /// Returns the extracted facts.
    #[must_use]
    pub fn facts(&self) -> &SemanticFactSet {
        &self.facts
    }
}

/// An immutable repository semantic snapshot.
#[derive(Clone, Debug)]
pub struct SemanticIndexSnapshot {
    identity: SnapshotIdentity,
    repository: RepositoryIdentity,
    graph: SemanticGraph,
    documents: BTreeMap<PathBuf, IndexedDocument>,
}

impl SemanticIndexSnapshot {
    /// Returns a copy pinned to a caller-supplied revision identity.
    #[must_use]
    pub fn with_revision(mut self, revision_id: RevisionId) -> Self {
        self.identity = self.identity.clone().with_revision(revision_id);
        self
    }

    /// Returns snapshot identity and provenance scope.
    #[must_use]
    pub fn identity(&self) -> &SnapshotIdentity {
        &self.identity
    }
    /// Returns repository identities and canonical root.
    #[must_use]
    pub fn repository(&self) -> &RepositoryIdentity {
        &self.repository
    }
    /// Returns the immutable repository graph.
    #[must_use]
    pub fn graph(&self) -> &SemanticGraph {
        &self.graph
    }
    /// Returns indexed documents in relative-path order.
    #[must_use]
    pub fn documents(&self) -> &BTreeMap<PathBuf, IndexedDocument> {
        &self.documents
    }
}

/// Stage that produced an index diagnostic.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum IndexStage {
    /// Reading source bytes failed.
    Read,
    /// Parsing source failed before a tree was produced.
    Parse,
    /// Semantic extraction failed before facts were produced.
    Extract,
    /// Graph snapshot publication failed.
    Graph,
}

/// A file-associated diagnostic emitted during indexing.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IndexDiagnostic {
    path: PathBuf,
    stage: IndexStage,
    message: String,
}

impl IndexDiagnostic {
    fn new(path: PathBuf, stage: IndexStage, message: impl Into<String>) -> Self {
        Self { path, stage, message: message.into() }
    }
    /// Returns the affected relative path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
    /// Returns the diagnostic stage.
    #[must_use]
    pub const fn stage(&self) -> IndexStage {
        self.stage
    }
    /// Returns the diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Counts and diagnostics from one indexing pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexReport {
    discovered: usize,
    indexed: usize,
    unchanged: usize,
    skipped: usize,
    parse_diagnostics: usize,
    extraction_diagnostics: usize,
    diagnostics: Vec<IndexDiagnostic>,
    duration: Duration,
}

impl IndexReport {
    /// Returns discovered Java file count.
    #[must_use]
    pub const fn discovered(&self) -> usize {
        self.discovered
    }
    /// Returns successfully indexed file count.
    #[must_use]
    pub const fn indexed(&self) -> usize {
        self.indexed
    }
    /// Returns content-unchanged file count.
    #[must_use]
    pub const fn unchanged(&self) -> usize {
        self.unchanged
    }
    /// Returns skipped file or directory count.
    #[must_use]
    pub const fn skipped(&self) -> usize {
        self.skipped
    }
    /// Returns syntax diagnostic count.
    #[must_use]
    pub const fn parse_diagnostics(&self) -> usize {
        self.parse_diagnostics
    }
    /// Returns semantic extraction diagnostic count.
    #[must_use]
    pub const fn extraction_diagnostics(&self) -> usize {
        self.extraction_diagnostics
    }
    /// Returns structured file-associated diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[IndexDiagnostic] {
        &self.diagnostics
    }
    /// Returns elapsed indexing duration.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }
}

/// Successful output of one repository indexing pass.
#[derive(Clone, Debug)]
pub struct IndexResult {
    snapshot: SemanticIndexSnapshot,
    report: IndexReport,
}

impl IndexResult {
    /// Returns the published immutable snapshot.
    #[must_use]
    pub fn snapshot(&self) -> &SemanticIndexSnapshot {
        &self.snapshot
    }
    /// Returns pass counts and diagnostics.
    #[must_use]
    pub const fn report(&self) -> &IndexReport {
        &self.report
    }
    /// Consumes the result into its snapshot and report.
    #[must_use]
    pub fn into_parts(self) -> (SemanticIndexSnapshot, IndexReport) {
        (self.snapshot, self.report)
    }
}

/// Repository indexing service.
#[derive(Clone, Debug)]
pub struct RepositoryIndex {
    options: IndexOptions,
}

impl RepositoryIndex {
    /// Creates a repository indexer with explicit options.
    #[must_use]
    pub const fn new(options: IndexOptions) -> Self {
        Self { options }
    }
    /// Returns indexing options.
    #[must_use]
    pub const fn options(&self) -> &IndexOptions {
        &self.options
    }

    /// Indexes all discovered Java files into one coherent graph snapshot.
    ///
    /// When `previous` belongs to the same canonical root, unchanged documents
    /// are reused and changed or deleted documents are applied through the
    /// graph's document replacement contracts. Graph publication happens only
    /// after the complete pass succeeds.
    ///
    /// # Errors
    /// Returns an error if discovery, parser construction, identity creation,
    /// or graph publication fails. Per-file read, parse, and extraction
    /// failures are retained as diagnostics and do not abort the pass.
    #[allow(clippy::too_many_lines)]
    pub fn index(
        &self,
        path: impl AsRef<Path>,
        previous: Option<&SemanticIndexSnapshot>,
    ) -> Result<IndexResult, IndexError> {
        let started = Instant::now();
        let discovery = SourceDiscovery::new(self.options.discovery.clone()).discover(path)?;
        let repository = RepositoryIdentity::from_root(discovery.root())?;
        let parser =
            JavaParser::new(self.options.parser.clone()).map_err(IndexError::ParseSetup)?;
        let extractor = JavaExtractor::new();
        let compatible_previous =
            previous.filter(|snapshot| snapshot.repository().root() == repository.root());
        let mut documents =
            compatible_previous.map_or_else(BTreeMap::new, |snapshot| snapshot.documents.clone());
        let mut graph = compatible_previous
            .map_or_else(SemanticGraph::empty, |snapshot| snapshot.graph.clone());
        let mut report = IndexReportBuilder::new(discovery.files().len(), discovery.skipped());
        let mut current_paths = BTreeSet::new();
        for file in discovery.files() {
            let relative = file.relative_path().to_path_buf();
            current_paths.insert(relative.clone());
            let source = match fs::read_to_string(file.absolute_path()) {
                Ok(source) => source,
                Err(error) => {
                    report.skipped += 1;
                    report.diagnostics.push(IndexDiagnostic::new(
                        relative,
                        IndexStage::Read,
                        error.to_string(),
                    ));
                    continue;
                }
            };
            let hash = content_hash(&source)?;
            if let Some(existing) = documents.get(&relative) {
                if existing.content_hash() == &hash {
                    report.unchanged += 1;
                    report.indexed += 1;
                    continue;
                }
            }
            let document_name = relative.to_string_lossy().to_string();
            let parsed = match parser.parse_source(ParseInput::new(
                &document_name,
                source,
                DocumentVersion::initial(),
            )) {
                Ok(parsed) => parsed,
                Err(error) => {
                    report.skipped += 1;
                    report.diagnostics.push(IndexDiagnostic::new(
                        relative,
                        IndexStage::Parse,
                        error.to_string(),
                    ));
                    continue;
                }
            };
            report.parse_diagnostics += parsed.diagnostics().len();
            let extracted = match extractor.extract(parsed.document()) {
                Ok(extracted) => extracted,
                Err(error) => {
                    report.skipped += 1;
                    report.diagnostics.push(IndexDiagnostic::new(
                        relative,
                        IndexStage::Extract,
                        error.to_string(),
                    ));
                    continue;
                }
            };
            report.extraction_diagnostics += extracted.diagnostics().len();
            let document_id = DocumentId::new(document_name)?;
            let revision_id = revision_id(&repository, &hash)?;
            let provenance = FactProvenance::new(
                repository.repository_id().clone(),
                repository.workspace_id().clone(),
                document_id.clone(),
                revision_id.clone(),
                hash.clone(),
                ProducerIdentity::new("branchsense-extractor-java", env!("CARGO_PKG_VERSION"))?,
            )
            .with_project(repository.project_id().clone());
            let facts = extracted.facts().clone().with_provenance(provenance);
            graph = graph
                .replace_document_facts(document_id.clone(), revision_id, facts.clone())
                .map_err(IndexError::Graph)?;
            documents.insert(
                relative.clone(),
                IndexedDocument { relative_path: relative, content_hash: hash, facts },
            );
            report.indexed += 1;
        }
        let deleted = documents
            .keys()
            .filter(|path| !current_paths.contains(*path))
            .cloned()
            .collect::<Vec<_>>();
        for relative in deleted {
            let document_id = DocumentId::new(relative.to_string_lossy().to_string())?;
            let revision = revision_id(&repository, &ContentHash::new("fnv1a64:deleted")?)?;
            graph = graph.remove_document(&document_id, revision).map_err(IndexError::Graph)?;
            documents.remove(&relative);
        }
        let snapshot_revision = repository_revision(&repository, &documents)?;
        let identity = SnapshotIdentity::new(
            repository.repository_id().clone(),
            repository.workspace_id().clone(),
            snapshot_revision,
        )
        .with_project(repository.project_id().clone());
        report.duration = started.elapsed();
        Ok(IndexResult {
            snapshot: SemanticIndexSnapshot { identity, repository, graph, documents },
            report: report.finish(),
        })
    }

    /// Indexes UTF-8 Java sources supplied by a Git tree or another immutable
    /// source provider.
    ///
    /// The source map must contain repository-relative paths. This method
    /// shares the parser, extractor, graph, provenance, and reporting pipeline
    /// with filesystem indexing while avoiding any working-tree mutation.
    ///
    /// # Errors
    /// Returns an error when parser setup, semantic identity construction, or
    /// graph publication fails.
    pub fn index_sources(
        &self,
        repository: RepositoryIdentity,
        sources: BTreeMap<PathBuf, String>,
        previous: Option<&SemanticIndexSnapshot>,
    ) -> Result<IndexResult> {
        let started = Instant::now();
        let parser =
            JavaParser::new(self.options.parser.clone()).map_err(IndexError::ParseSetup)?;
        let extractor = JavaExtractor::new();
        let compatible_previous = previous
            .filter(|snapshot| snapshot.repository().repository_id() == repository.repository_id());
        let mut documents =
            compatible_previous.map_or_else(BTreeMap::new, |snapshot| snapshot.documents.clone());
        let mut graph = compatible_previous
            .map_or_else(SemanticGraph::empty, |snapshot| snapshot.graph.clone());
        let mut report = IndexReportBuilder::new(sources.len(), 0);
        let mut current_paths = BTreeSet::new();
        for (relative, source) in sources {
            current_paths.insert(relative.clone());
            let hash = content_hash(&source)?;
            if let Some(existing) = documents.get(&relative) {
                if existing.content_hash() == &hash {
                    report.unchanged += 1;
                    report.indexed += 1;
                    continue;
                }
            }
            let document_name = relative.to_string_lossy().to_string();
            let parsed = match parser.parse_source(ParseInput::new(
                &document_name,
                source,
                DocumentVersion::initial(),
            )) {
                Ok(parsed) => parsed,
                Err(error) => {
                    report.skipped += 1;
                    report.diagnostics.push(IndexDiagnostic::new(
                        relative,
                        IndexStage::Parse,
                        error.to_string(),
                    ));
                    continue;
                }
            };
            report.parse_diagnostics += parsed.diagnostics().len();
            let extracted = match extractor.extract(parsed.document()) {
                Ok(extracted) => extracted,
                Err(error) => {
                    report.skipped += 1;
                    report.diagnostics.push(IndexDiagnostic::new(
                        relative,
                        IndexStage::Extract,
                        error.to_string(),
                    ));
                    continue;
                }
            };
            report.extraction_diagnostics += extracted.diagnostics().len();
            let document_id = DocumentId::new(document_name)?;
            let revision_id = revision_id(&repository, &hash)?;
            let provenance = FactProvenance::new(
                repository.repository_id().clone(),
                repository.workspace_id().clone(),
                document_id.clone(),
                revision_id.clone(),
                hash.clone(),
                ProducerIdentity::new("branchsense-extractor-java", env!("CARGO_PKG_VERSION"))?,
            )
            .with_project(repository.project_id().clone());
            let facts = extracted.facts().clone().with_provenance(provenance);
            graph = graph
                .replace_document_facts(document_id.clone(), revision_id, facts.clone())
                .map_err(IndexError::Graph)?;
            documents.insert(
                relative.clone(),
                IndexedDocument { relative_path: relative, content_hash: hash, facts },
            );
            report.indexed += 1;
        }
        let deleted = documents
            .keys()
            .filter(|path| !current_paths.contains(*path))
            .cloned()
            .collect::<Vec<_>>();
        for relative in deleted {
            let document_id = DocumentId::new(relative.to_string_lossy().to_string())?;
            let revision = revision_id(&repository, &ContentHash::new("fnv1a64:deleted")?)?;
            graph = graph.remove_document(&document_id, revision).map_err(IndexError::Graph)?;
            documents.remove(&relative);
        }
        let snapshot_revision = repository_revision(&repository, &documents)?;
        let identity = SnapshotIdentity::new(
            repository.repository_id().clone(),
            repository.workspace_id().clone(),
            snapshot_revision,
        )
        .with_project(repository.project_id().clone());
        report.duration = started.elapsed();
        Ok(IndexResult {
            snapshot: SemanticIndexSnapshot { identity, repository, graph, documents },
            report: report.finish(),
        })
    }
}

struct IndexReportBuilder {
    discovered: usize,
    indexed: usize,
    unchanged: usize,
    skipped: usize,
    parse_diagnostics: usize,
    extraction_diagnostics: usize,
    diagnostics: Vec<IndexDiagnostic>,
    duration: Duration,
}
impl IndexReportBuilder {
    fn new(discovered: usize, skipped: usize) -> Self {
        Self {
            discovered,
            indexed: 0,
            unchanged: 0,
            skipped,
            parse_diagnostics: 0,
            extraction_diagnostics: 0,
            diagnostics: Vec::new(),
            duration: Duration::ZERO,
        }
    }
    fn finish(self) -> IndexReport {
        IndexReport {
            discovered: self.discovered,
            indexed: self.indexed,
            unchanged: self.unchanged,
            skipped: self.skipped,
            parse_diagnostics: self.parse_diagnostics,
            extraction_diagnostics: self.extraction_diagnostics,
            diagnostics: self.diagnostics,
            duration: self.duration,
        }
    }
}

fn content_hash(source: &str) -> branchsense_semantic::Result<ContentHash> {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in source.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    ContentHash::new(format!("fnv1a64:{hash:016x}"))
}

fn revision_id(
    repository: &RepositoryIdentity,
    hash: &ContentHash,
) -> std::result::Result<RevisionId, branchsense_core::ModelError> {
    RevisionId::new(format!("revision:index:{}:{}", repository.repository_id(), hash.as_str()))
}

fn repository_revision(
    repository: &RepositoryIdentity,
    documents: &BTreeMap<PathBuf, IndexedDocument>,
) -> std::result::Result<RevisionId, branchsense_core::ModelError> {
    let mut material = String::new();
    for (path, document) in documents {
        material.push_str(&path.to_string_lossy());
        material.push('\0');
        material.push_str(document.content_hash().as_str());
        material.push('\0');
    }
    let hash = content_hash(&material)
        .map_err(|_| branchsense_core::ModelError::EmptyValue { kind: "repository revision" })?;
    revision_id(repository, &hash)
}

/// Errors raised before a coherent index snapshot can be published.
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    /// Source discovery failed.
    #[error("source discovery failed: {0}")]
    Discovery(#[from] DiscoveryError),
    /// Core identity construction failed.
    #[error("core identity construction failed: {0}")]
    Core(#[from] branchsense_core::ModelError),
    /// Semantic provenance construction failed.
    #[error("semantic identity construction failed: {0}")]
    Semantic(#[from] branchsense_semantic::SemanticError),
    /// Java parser construction failed.
    #[error("Java parser setup failed: {0}")]
    ParseSetup(#[from] branchsense_parser::ParseError),
    /// Graph publication failed.
    #[error("semantic graph publication failed: {0}")]
    Graph(#[from] GraphError),
}

/// Result alias for repository indexing.
pub type Result<T, E = IndexError> = std::result::Result<T, E>;

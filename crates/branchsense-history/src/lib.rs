//! Bounded, deterministic historical evidence for semantic repositories.
//!
//! Historical analysis is intentionally independent from collision assessment.
//! It walks a revision's read-only Git ancestry, compares each commit with its
//! first parent, and reports evidence such as symbol change frequency,
//! recency, semantic symbol co-change, and file co-change. It does not score
//! collision risk, infer ownership, reconstruct conflicts, or implement BCS.
//!
//! Symbol identity is revision-safe by design. Git-backed Java symbol IDs
//! include a document path and may also include a signature that changes over
//! time, so this crate does not compare opaque [`SymbolId`] values across
//! revisions. It derives a best-effort [`SymbolKey`] from document path,
//! semantic kind, and signature-independent qualified name. Ambiguous or
//! renamed declarations are not silently treated as the same entity.
#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use branchsense_core::SymbolId;
use branchsense_diff::{ChangeKind, SemanticDiff};
use branchsense_git::{
    GitCommitId, GitError, GitRepository, GitRevision, GitSemanticSnapshot, GitSnapshotIndexer,
};
use branchsense_index::SemanticIndexSnapshot;
use branchsense_semantic::{SemanticFact, SemanticFactRecord, SymbolDefinition, SymbolKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned by bounded historical analysis.
#[derive(Debug, Error)]
pub enum HistoricalError {
    /// Read-only Git traversal or snapshot loading failed.
    #[error(transparent)]
    Git(#[from] GitError),
    /// The requested history window is invalid.
    #[error("history option `{0}` must be greater than zero")]
    InvalidOption(&'static str),
}

/// The standard historical-analysis result.
pub type Result<T> = std::result::Result<T, HistoricalError>;

/// A stable semantic key used only for best-effort cross-revision comparison.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SymbolKey {
    document: PathBuf,
    kind: SymbolKind,
    qualified_name: String,
}

impl SymbolKey {
    /// Returns the repository-relative document path.
    #[must_use]
    pub fn document(&self) -> &std::path::Path {
        &self.document
    }
    /// Returns the language-neutral declaration kind.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }
    /// Returns the signature-independent qualified name used for matching.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
}

/// One symbol's historical change frequency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ChangeFrequencySignal {
    symbol: SymbolKey,
    total_changes: usize,
    commits_considered: usize,
    first_observed_change: Option<GitCommitId>,
    most_recent_change: Option<GitCommitId>,
}

impl ChangeFrequencySignal {
    /// Returns the matched semantic symbol key.
    #[must_use]
    pub fn symbol(&self) -> &SymbolKey {
        &self.symbol
    }
    /// Returns the number of commits changing the symbol.
    #[must_use]
    pub const fn total_changes(&self) -> usize {
        self.total_changes
    }
    /// Returns the bounded denominator used by this signal.
    #[must_use]
    pub const fn commits_considered(&self) -> usize {
        self.commits_considered
    }
    /// Returns the oldest observed changing revision.
    #[must_use]
    pub fn first_observed_change(&self) -> Option<&GitCommitId> {
        self.first_observed_change.as_ref()
    }
    /// Returns the newest observed changing revision.
    #[must_use]
    pub fn most_recent_change(&self) -> Option<&GitCommitId> {
        self.most_recent_change.as_ref()
    }
}

/// Recency evidence for one semantic symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecencySignal {
    symbol: SymbolKey,
    last_changed_revision: GitCommitId,
    last_changed_timestamp_seconds: i64,
    age_in_commits: usize,
}

impl RecencySignal {
    /// Returns the matched semantic symbol key.
    #[must_use]
    pub fn symbol(&self) -> &SymbolKey {
        &self.symbol
    }
    /// Returns the newest revision that changed the symbol.
    #[must_use]
    pub fn last_changed_revision(&self) -> &GitCommitId {
        &self.last_changed_revision
    }
    /// Returns the recorded committer timestamp in Unix seconds.
    #[must_use]
    pub const fn last_changed_timestamp_seconds(&self) -> i64 {
        self.last_changed_timestamp_seconds
    }
    /// Returns the number of analyzed commit positions since the change.
    #[must_use]
    pub const fn age_in_commits(&self) -> usize {
        self.age_in_commits
    }
}

/// Semantic symbol pair co-change evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CoChangeSignal {
    left: SymbolKey,
    right: SymbolKey,
    co_change_count: usize,
    commits_considered: usize,
    revisions: Vec<GitCommitId>,
}

impl CoChangeSignal {
    /// Returns the first pair member in stable key order.
    #[must_use]
    pub fn left(&self) -> &SymbolKey {
        &self.left
    }
    /// Returns the second pair member in stable key order.
    #[must_use]
    pub fn right(&self) -> &SymbolKey {
        &self.right
    }
    /// Returns commits in which both symbols changed.
    #[must_use]
    pub const fn co_change_count(&self) -> usize {
        self.co_change_count
    }
    /// Returns the bounded commit denominator.
    #[must_use]
    pub const fn commits_considered(&self) -> usize {
        self.commits_considered
    }
    /// Returns supporting revisions in deterministic newest-first order.
    #[must_use]
    pub fn revisions(&self) -> &[GitCommitId] {
        &self.revisions
    }
}

/// File-level co-change evidence kept separate from symbol evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileCoChangeSignal {
    left: PathBuf,
    right: PathBuf,
    co_change_count: usize,
    commits_considered: usize,
    revisions: Vec<GitCommitId>,
}

impl FileCoChangeSignal {
    /// Returns the first repository-relative file path.
    #[must_use]
    pub fn left(&self) -> &std::path::Path {
        &self.left
    }
    /// Returns the second repository-relative file path.
    #[must_use]
    pub fn right(&self) -> &std::path::Path {
        &self.right
    }
    /// Returns commits in which both files changed.
    #[must_use]
    pub const fn co_change_count(&self) -> usize {
        self.co_change_count
    }
    /// Returns the bounded commit denominator.
    #[must_use]
    pub const fn commits_considered(&self) -> usize {
        self.commits_considered
    }
    /// Returns supporting revisions in deterministic newest-first order.
    #[must_use]
    pub fn revisions(&self) -> &[GitCommitId] {
        &self.revisions
    }
}

/// Immutable historical evidence pinned to one analysis revision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HistoricalSignals {
    analysis_revision: GitCommitId,
    commits_analyzed: usize,
    change_frequency: Vec<ChangeFrequencySignal>,
    recency: Vec<RecencySignal>,
    symbol_co_change: Vec<CoChangeSignal>,
    file_co_change: Vec<FileCoChangeSignal>,
}

impl HistoricalSignals {
    /// Returns the revision from which the history walk began.
    #[must_use]
    pub fn analysis_revision(&self) -> &GitCommitId {
        &self.analysis_revision
    }
    /// Returns the number of commits included by the bounded window.
    #[must_use]
    pub const fn commits_analyzed(&self) -> usize {
        self.commits_analyzed
    }
    /// Returns symbol frequency evidence in stable key order.
    #[must_use]
    pub fn change_frequency(&self) -> &[ChangeFrequencySignal] {
        &self.change_frequency
    }
    /// Returns symbol recency evidence in stable key order.
    #[must_use]
    pub fn recency(&self) -> &[RecencySignal] {
        &self.recency
    }
    /// Returns semantic symbol co-change evidence in stable pair order.
    #[must_use]
    pub fn symbol_co_change(&self) -> &[CoChangeSignal] {
        &self.symbol_co_change
    }
    /// Returns separate file-level co-change evidence.
    #[must_use]
    pub fn file_co_change(&self) -> &[FileCoChangeSignal] {
        &self.file_co_change
    }
}

/// Bounds for one historical analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoricalOptions {
    max_commits: usize,
}

impl Default for HistoricalOptions {
    fn default() -> Self {
        Self { max_commits: 500 }
    }
}

impl HistoricalOptions {
    /// Creates a bounded commit window.
    #[must_use]
    pub const fn new(max_commits: usize) -> Self {
        Self { max_commits }
    }
    /// Returns the maximum number of commits to inspect.
    #[must_use]
    pub const fn max_commits(&self) -> usize {
        self.max_commits
    }
}

/// Read-only historical analyzer built on Git revisions and semantic diffs.
#[derive(Clone, Debug, Default)]
pub struct HistoricalAnalyzer {
    indexer: GitSnapshotIndexer,
}

impl HistoricalAnalyzer {
    /// Creates an analyzer with default Java snapshot indexing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes bounded history from `revision` without mutating the repository.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid options, Git traversal failures, or
    /// revision snapshot indexing failures.
    pub fn analyze(
        &self,
        repository: &GitRepository,
        revision: &GitRevision,
        options: HistoricalOptions,
    ) -> Result<HistoricalSignals> {
        if options.max_commits == 0 {
            return Err(HistoricalError::InvalidOption("max_commits"));
        }
        let history = repository.history(revision, options.max_commits)?;
        let mut snapshots = BTreeMap::<GitCommitId, GitSemanticSnapshot>::new();
        let mut frequency = BTreeMap::<SymbolKey, FrequencyState>::new();
        let mut symbol_pairs = BTreeMap::<(SymbolKey, SymbolKey), PairState>::new();
        let mut file_pairs = BTreeMap::<(PathBuf, PathBuf), PairState>::new();

        for (age, current) in history.iter().enumerate() {
            let current_snapshot = snapshot(repository, &self.indexer, current, &mut snapshots)?;
            let (changed_symbols, changed_files) = if let Some(parent_id) =
                current.parents().first()
            {
                let parent = repository.resolve(parent_id.as_str())?;
                let parent_snapshot = snapshot(repository, &self.indexer, &parent, &mut snapshots)?;
                let diff = branchsense_diff::SemanticDiffer::new()
                    .diff_git(&parent_snapshot, &current_snapshot);
                (
                    changed_symbols(&diff, parent_snapshot.semantic(), current_snapshot.semantic()),
                    changed_files(&diff),
                )
            } else {
                (
                    snapshot_symbols(current_snapshot.semantic()),
                    current_snapshot.semantic().documents().keys().cloned().collect(),
                )
            };
            let revision_id = current.commit_id().clone();
            for symbol in &changed_symbols {
                let state = frequency.entry(symbol.clone()).or_default();
                state.count += 1;
                state.newest.get_or_insert(revision_id.clone());
                state.oldest = Some(revision_id.clone());
                state.newest_age.get_or_insert(age);
            }
            add_pairs(&mut symbol_pairs, &changed_symbols, &revision_id);
            add_file_pairs(&mut file_pairs, &changed_files, &revision_id);
        }

        let commits_analyzed = history.len();
        let recency = frequency_recency(&frequency, &history);
        let change_frequency = frequency
            .into_iter()
            .map(|(symbol, state)| ChangeFrequencySignal {
                symbol,
                total_changes: state.count,
                commits_considered: commits_analyzed,
                first_observed_change: state.oldest,
                most_recent_change: state.newest,
            })
            .collect();
        let symbol_co_change = pair_signals(symbol_pairs, commits_analyzed);
        let file_co_change = file_pair_signals(file_pairs, commits_analyzed);
        Ok(HistoricalSignals {
            analysis_revision: revision.commit_id().clone(),
            commits_analyzed,
            change_frequency,
            recency,
            symbol_co_change,
            file_co_change,
        })
    }
}

#[derive(Clone, Debug, Default)]
struct FrequencyState {
    count: usize,
    oldest: Option<GitCommitId>,
    newest: Option<GitCommitId>,
    newest_age: Option<usize>,
}

#[derive(Clone, Debug, Default)]
struct PairState {
    count: usize,
    revisions: Vec<GitCommitId>,
}

fn snapshot(
    repository: &GitRepository,
    indexer: &GitSnapshotIndexer,
    revision: &GitRevision,
    cache: &mut BTreeMap<GitCommitId, GitSemanticSnapshot>,
) -> Result<GitSemanticSnapshot> {
    if let Some(snapshot) = cache.get(revision.commit_id()) {
        return Ok(snapshot.clone());
    }
    let snapshot = indexer.index_revision(repository, revision, None)?;
    cache.insert(revision.commit_id().clone(), snapshot.clone());
    Ok(snapshot)
}

fn changed_files(diff: &SemanticDiff) -> BTreeSet<PathBuf> {
    diff.documents()
        .iter()
        .filter(|change| change.kind() != ChangeKind::Unchanged)
        .map(|change| change.path().to_path_buf())
        .collect()
}

fn changed_symbols(
    diff: &SemanticDiff,
    before: &SemanticIndexSnapshot,
    after: &SemanticIndexSnapshot,
) -> BTreeSet<SymbolKey> {
    let mut symbols = BTreeSet::new();
    for change in diff.symbols().iter().filter(|change| change.kind() != ChangeKind::Unchanged) {
        if let Some(definition) = change.after().or(change.before()) {
            symbols.insert(symbol_key(definition));
        }
    }
    for relationship in diff.relationships() {
        let fact = relationship
            .fact()
            .after()
            .or(relationship.fact().before())
            .map(SemanticFactRecord::fact);
        if let Some(source) = fact.and_then(source_symbol) {
            let definition =
                after.graph().find_symbol(&source).and_then(|node| node.definition()).or_else(
                    || before.graph().find_symbol(&source).and_then(|node| node.definition()),
                );
            if let Some(definition) = definition {
                symbols.insert(symbol_key(definition));
            }
        }
    }
    symbols
}

fn snapshot_symbols(snapshot: &SemanticIndexSnapshot) -> BTreeSet<SymbolKey> {
    snapshot
        .documents()
        .values()
        .flat_map(|document| document.facts().facts())
        .filter_map(|record| {
            if let SemanticFact::Definition(definition) = record.fact() {
                Some(symbol_key(definition))
            } else {
                None
            }
        })
        .collect()
}

fn symbol_key(definition: &SymbolDefinition) -> SymbolKey {
    let qualified_name = definition
        .qualified_name()
        .map_or_else(|| definition.name().to_string(), ToString::to_string);
    let qualified_name =
        qualified_name.split_once('(').map_or(qualified_name.as_str(), |(name, _)| name).to_owned();
    SymbolKey {
        document: PathBuf::from(definition.location().document_id().as_str()),
        kind: definition.kind(),
        qualified_name,
    }
}

fn source_symbol(fact: &SemanticFact) -> Option<SymbolId> {
    match fact {
        SemanticFact::Contains(fact) => Some(fact.container().clone()),
        SemanticFact::Call(fact) => Some(fact.caller().clone()),
        SemanticFact::Reference(fact) => Some(fact.source().clone()),
        SemanticFact::TypeRelation(fact) => Some(fact.source().clone()),
        SemanticFact::Dependency(fact) => Some(fact.source().clone()),
        SemanticFact::Definition(_)
        | SemanticFact::Parameter(_)
        | SemanticFact::ReturnType(_)
        | SemanticFact::Import(_)
        | SemanticFact::Documentation(_)
        | SemanticFact::Annotation(_) => None,
    }
}

fn add_pairs(
    pairs: &mut BTreeMap<(SymbolKey, SymbolKey), PairState>,
    symbols: &BTreeSet<SymbolKey>,
    revision: &GitCommitId,
) {
    let symbols = symbols.iter().collect::<Vec<_>>();
    for (index, left) in symbols.iter().enumerate() {
        for right in symbols.iter().skip(index + 1) {
            let state = pairs.entry(((*left).clone(), (*right).clone())).or_default();
            state.count += 1;
            state.revisions.push(revision.clone());
        }
    }
}

fn add_file_pairs(
    pairs: &mut BTreeMap<(PathBuf, PathBuf), PairState>,
    files: &BTreeSet<PathBuf>,
    revision: &GitCommitId,
) {
    let files = files.iter().collect::<Vec<_>>();
    for (index, left) in files.iter().enumerate() {
        for right in files.iter().skip(index + 1) {
            let state = pairs.entry(((*left).clone(), (*right).clone())).or_default();
            state.count += 1;
            state.revisions.push(revision.clone());
        }
    }
}

fn frequency_recency(
    frequency: &BTreeMap<SymbolKey, FrequencyState>,
    history: &[GitRevision],
) -> Vec<RecencySignal> {
    frequency
        .iter()
        .filter_map(|(symbol, state)| {
            let revision = state.newest.as_ref()?;
            let age = state.newest_age?;
            let timestamp = history.get(age)?.committer().timestamp_seconds();
            Some(RecencySignal {
                symbol: symbol.clone(),
                last_changed_revision: revision.clone(),
                last_changed_timestamp_seconds: timestamp,
                age_in_commits: age,
            })
        })
        .collect()
}

fn pair_signals(
    pairs: BTreeMap<(SymbolKey, SymbolKey), PairState>,
    commits: usize,
) -> Vec<CoChangeSignal> {
    pairs
        .into_iter()
        .map(|((left, right), state)| CoChangeSignal {
            left,
            right,
            co_change_count: state.count,
            commits_considered: commits,
            revisions: state.revisions,
        })
        .collect()
}

fn file_pair_signals(
    pairs: BTreeMap<(PathBuf, PathBuf), PairState>,
    commits: usize,
) -> Vec<FileCoChangeSignal> {
    pairs
        .into_iter()
        .map(|((left, right), state)| FileCoChangeSignal {
            left,
            right,
            co_change_count: state.count,
            commits_considered: commits,
            revisions: state.revisions,
        })
        .collect()
}

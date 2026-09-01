//! Deterministic historical contributor responsibility evidence.
//!
//! This crate answers a deliberately limited question: which Git commit
//! authors contributed changes to semantic symbols or files in a bounded
//! history window? It does not infer ownership, expertise, intent, or social
//! responsibility. Its results are evidence that a later collision-analysis
//! layer may consume independently.
//!
//! Symbol attribution is emitted only when the semantic diff identifies the
//! changed declaration or the source symbol of a changed relationship. File
//! attribution is retained separately for document changes that cannot be
//! mapped reliably to a symbol. Revision-specific [`branchsense_core::SymbolId`]
//! values are never compared across revisions; [`SemanticEntityKey`] provides
//! the same conservative path/kind/qualified-name matching policy used by the
//! historical subsystem.
#![forbid(unsafe_code)]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use branchsense_diff::{ChangeKind, SemanticDiff};
use branchsense_git::{
    GitCommitId, GitError, GitRepository, GitRevision, GitSemanticSnapshot, GitSnapshotIndexer,
};
use branchsense_index::SemanticIndexSnapshot;
use branchsense_semantic::{
    AnalysisProvenance, EvidenceCompleteness, EvidenceEnvelope, EvidenceIdentity, EvidenceKind,
    EvidenceState, SemanticEntityIdentity, SemanticFact, SemanticFactRecord, SymbolDefinition,
    SymbolKind,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned by responsibility analysis.
#[derive(Debug, Error)]
pub enum ResponsibilityError {
    /// Read-only Git traversal or snapshot loading failed.
    #[error(transparent)]
    Git(#[from] GitError),
    /// A canonical provenance identity could not be constructed.
    #[error("responsibility identity construction failed: {0}")]
    Identity(#[from] branchsense_core::ModelError),
    /// An analysis option is outside its valid range.
    #[error("responsibility option `{0}` must be greater than zero")]
    InvalidOption(&'static str),
}

/// The standard result type for this crate.
pub type Result<T> = std::result::Result<T, ResponsibilityError>;

/// A conservative cross-revision semantic entity key.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SemanticEntityKey {
    document: PathBuf,
    kind: SymbolKind,
    qualified_name: String,
}

impl SemanticEntityKey {
    /// Returns the repository-relative document path.
    #[must_use]
    pub fn document(&self) -> &Path {
        &self.document
    }
    /// Returns the language-independent declaration kind.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }
    /// Returns the signature-independent qualified name.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
}

/// The evidence scope, kept explicit so file evidence is not mistaken for
/// symbol evidence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ResponsibilityScope {
    /// Semantic declaration evidence.
    Symbol,
    /// Document-level evidence.
    File,
}

/// The basis on which a contribution was attributed.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum AttributionBasis {
    /// A semantic declaration or relationship source changed.
    SemanticChange,
    /// The document changed but no reliable symbol mapping existed.
    DocumentChange,
}

/// A conservative Git author identity.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Contributor {
    name: String,
    email: String,
}

impl Contributor {
    /// Creates an identity by trimming the name and case-folding only the
    /// email address. Names are never used to merge identities.
    #[must_use]
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into().trim().to_owned(),
            email: email.into().trim().to_ascii_lowercase(),
        }
    }
    /// Returns the recorded author name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the normalized author email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Returns a copy with the email address replaced by a stable redaction.
    #[must_use]
    pub fn redacted(&self) -> Self {
        Self { name: self.name.clone(), email: "[redacted]".into() }
    }
}

/// One contributor's observed change count and exact share.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Contribution {
    contributor: Contributor,
    commit_count: usize,
    share: f64,
}

impl Contribution {
    /// Returns the contributor identity.
    #[must_use]
    pub fn contributor(&self) -> &Contributor {
        &self.contributor
    }
    /// Returns the number of distinct analyzed commits attributed to the contributor.
    #[must_use]
    pub const fn commit_count(&self) -> usize {
        self.commit_count
    }
    /// Returns `commit_count / total attributed commits`.
    #[must_use]
    pub const fn share(&self) -> f64 {
        self.share
    }
}

/// A simple, explainable concentration summary.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResponsibilityConcentration {
    top_contributor_share: f64,
    active_contributors: usize,
}

impl ResponsibilityConcentration {
    /// Returns the share held by the leading contributor, or zero for no evidence.
    #[must_use]
    pub const fn top_contributor_share(&self) -> f64 {
        self.top_contributor_share
    }
    /// Returns the number of contributors with at least one attributed commit.
    #[must_use]
    pub const fn active_contributors(&self) -> usize {
        self.active_contributors
    }
}

/// Evidence collected for one semantic symbol or file.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResponsibilityEvidence {
    entity: ResponsibilityEntity,
    scope: ResponsibilityScope,
    attribution_basis: AttributionBasis,
    contributions: Vec<Contribution>,
    recent_contributors: Vec<Contributor>,
    concentration: ResponsibilityConcentration,
    supporting_commits: Vec<GitCommitId>,
}

impl ResponsibilityEvidence {
    /// Returns the attributed entity.
    #[must_use]
    pub fn entity(&self) -> &ResponsibilityEntity {
        &self.entity
    }
    /// Returns whether this is symbol- or file-level evidence.
    #[must_use]
    pub const fn scope(&self) -> ResponsibilityScope {
        self.scope
    }
    /// Returns the evidence attribution basis.
    #[must_use]
    pub const fn attribution_basis(&self) -> AttributionBasis {
        self.attribution_basis
    }
    /// Returns contributions in descending count and stable identity order.
    #[must_use]
    pub fn contributions(&self) -> &[Contribution] {
        &self.contributions
    }
    /// Returns contributors with an attribution in the recent window.
    #[must_use]
    pub fn recent_contributors(&self) -> &[Contributor] {
        &self.recent_contributors
    }
    /// Returns concentration evidence.
    #[must_use]
    pub const fn concentration(&self) -> &ResponsibilityConcentration {
        &self.concentration
    }
    /// Returns supporting commit IDs in newest-first order.
    #[must_use]
    pub fn supporting_commits(&self) -> &[GitCommitId] {
        &self.supporting_commits
    }
}

/// An explicitly typed symbol or file evidence target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ResponsibilityEntity {
    /// A semantic entity matched conservatively across revisions.
    Symbol(SemanticEntityKey),
    /// A repository-relative document path.
    File(PathBuf),
}

/// Immutable responsibility evidence pinned to one revision and window.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ResponsibilitySignals {
    analysis_revision: GitCommitId,
    commits_analyzed: usize,
    recent_window: usize,
    evidence: EvidenceEnvelope,
    symbol_responsibility: Vec<ResponsibilityEvidence>,
    file_responsibility: Vec<ResponsibilityEvidence>,
}

impl ResponsibilitySignals {
    /// Returns the analyzed revision.
    #[must_use]
    pub fn analysis_revision(&self) -> &GitCommitId {
        &self.analysis_revision
    }
    /// Returns the number of commits included in the bounded window.
    #[must_use]
    pub const fn commits_analyzed(&self) -> usize {
        self.commits_analyzed
    }
    /// Returns the recent-contributor window in commit positions.
    #[must_use]
    pub const fn recent_window(&self) -> usize {
        self.recent_window
    }
    /// Returns whether responsibility evidence was observed or inconclusive.
    #[must_use]
    pub const fn state(&self) -> EvidenceState {
        self.evidence.state()
    }
    /// Returns provenance sufficient to reproduce this analysis.
    #[must_use]
    pub fn provenance(&self) -> &AnalysisProvenance {
        self.evidence.provenance()
    }

    /// Returns the complete evidence contract for this analysis.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceEnvelope {
        &self.evidence
    }
    /// Returns symbol-level evidence only.
    #[must_use]
    pub fn symbol_responsibility(&self) -> &[ResponsibilityEvidence] {
        &self.symbol_responsibility
    }
    /// Returns file-level evidence only.
    #[must_use]
    pub fn file_responsibility(&self) -> &[ResponsibilityEvidence] {
        &self.file_responsibility
    }

    /// Returns a copy with contributor email addresses redacted.
    #[must_use]
    pub fn redacted(&self) -> Self {
        fn redact(items: &[ResponsibilityEvidence]) -> Vec<ResponsibilityEvidence> {
            items
                .iter()
                .map(|item| ResponsibilityEvidence {
                    entity: item.entity.clone(),
                    scope: item.scope,
                    attribution_basis: item.attribution_basis,
                    contributions: item
                        .contributions
                        .iter()
                        .map(|contribution| Contribution {
                            contributor: contribution.contributor.redacted(),
                            commit_count: contribution.commit_count,
                            share: contribution.share,
                        })
                        .collect(),
                    recent_contributors: item
                        .recent_contributors
                        .iter()
                        .map(Contributor::redacted)
                        .collect(),
                    concentration: item.concentration.clone(),
                    supporting_commits: item.supporting_commits.clone(),
                })
                .collect()
        }

        Self {
            analysis_revision: self.analysis_revision.clone(),
            commits_analyzed: self.commits_analyzed,
            recent_window: self.recent_window,
            evidence: self.evidence.clone(),
            symbol_responsibility: redact(&self.symbol_responsibility),
            file_responsibility: redact(&self.file_responsibility),
        }
    }
}

/// Bounds for one analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResponsibilityOptions {
    max_commits: usize,
    recent_commits: usize,
}

impl Default for ResponsibilityOptions {
    fn default() -> Self {
        Self { max_commits: 500, recent_commits: 10 }
    }
}

impl ResponsibilityOptions {
    /// Creates a bounded analysis with a ten-commit recent window.
    #[must_use]
    pub const fn new(max_commits: usize) -> Self {
        Self { max_commits, recent_commits: 10 }
    }
    /// Sets the recent-contributor window.
    #[must_use]
    pub const fn with_recent_commits(mut self, recent_commits: usize) -> Self {
        self.recent_commits = recent_commits;
        self
    }
    /// Returns the history bound.
    #[must_use]
    pub const fn max_commits(&self) -> usize {
        self.max_commits
    }
    /// Returns the recent window bound.
    #[must_use]
    pub const fn recent_commits(&self) -> usize {
        self.recent_commits
    }
}

/// Read-only analyzer of historical contributor evidence.
#[derive(Clone, Debug, Default)]
pub struct ResponsibilityAnalyzer {
    indexer: GitSnapshotIndexer,
}

impl ResponsibilityAnalyzer {
    /// Creates an analyzer using the repository's standard Java snapshot indexer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyzes author contribution evidence without changing Git state.
    ///
    /// A commit contributes at most once to an entity, even if many facts in
    /// that entity changed. Merge commits are compared with their first parent,
    /// matching the historical subsystem's conservative policy.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid options, Git traversal failures, or
    /// snapshot indexing failures.
    pub fn analyze(
        &self,
        repository: &GitRepository,
        revision: &GitRevision,
        options: ResponsibilityOptions,
    ) -> Result<ResponsibilitySignals> {
        if options.max_commits == 0 {
            return Err(ResponsibilityError::InvalidOption("max_commits"));
        }
        if options.recent_commits == 0 {
            return Err(ResponsibilityError::InvalidOption("recent_commits"));
        }
        let mut history = repository.history(revision, options.max_commits.saturating_add(1))?;
        let truncated = history.len() > options.max_commits;
        history.truncate(options.max_commits);
        let mut snapshots = BTreeMap::new();
        let mut symbols = BTreeMap::<SemanticEntityKey, EntityState>::new();
        let mut files = BTreeMap::<PathBuf, EntityState>::new();
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
            let contributor = Contributor::new(current.author().name(), current.author().email());
            for entity in changed_symbols {
                symbols.entry(entity).or_default().attribute(
                    &contributor,
                    age,
                    current.commit_id().clone(),
                );
            }
            for path in changed_files {
                files.entry(path).or_default().attribute(
                    &contributor,
                    age,
                    current.commit_id().clone(),
                );
            }
        }
        let state = if truncated {
            EvidenceState::Truncated
        } else if symbols.is_empty() && files.is_empty() {
            EvidenceState::NoEvidence
        } else {
            EvidenceState::Observed
        };
        let provenance = AnalysisProvenance::new()
            .with_repository(repository.identity().id().clone())
            .with_revision(branchsense_core::RevisionId::new(revision.commit_id().as_str())?)
            .with_history_window(options.max_commits);
        let mut evidence = EvidenceEnvelope::new(
            state,
            EvidenceCompleteness::new().with_responsibility(state),
            provenance,
        );
        for entity in symbols.keys() {
            let identity = SemanticEntityIdentity::new(
                entity.document().to_path_buf(),
                entity.kind(),
                entity.qualified_name(),
            );
            evidence = evidence
                .with_identity(EvidenceIdentity::semantic(EvidenceKind::Primary, &identity));
        }
        for path in files.keys() {
            evidence = evidence.with_identity(EvidenceIdentity::new(
                EvidenceKind::Primary,
                format!("file:{}", path.display()),
                Vec::new(),
            ));
        }
        Ok(ResponsibilitySignals {
            analysis_revision: revision.commit_id().clone(),
            commits_analyzed: history.len(),
            recent_window: options.recent_commits,
            evidence,
            symbol_responsibility: build_evidence(
                symbols,
                ResponsibilityScope::Symbol,
                options.recent_commits,
            ),
            file_responsibility: build_evidence(
                files,
                ResponsibilityScope::File,
                options.recent_commits,
            ),
        })
    }
}

#[derive(Clone, Debug, Default)]
struct EntityState {
    contributors: BTreeMap<Contributor, ContributorState>,
}
#[derive(Clone, Debug)]
struct ContributorState {
    count: usize,
    newest_age: usize,
    commits: Vec<(usize, GitCommitId)>,
}

impl EntityState {
    fn attribute(&mut self, contributor: &Contributor, age: usize, commit: GitCommitId) {
        let state = self
            .contributors
            .entry(contributor.clone())
            .or_insert_with(|| ContributorState { count: 0, newest_age: age, commits: Vec::new() });
        state.count += 1;
        state.newest_age = state.newest_age.min(age);
        state.commits.push((age, commit));
    }
}

fn build_evidence<K: Ord + Into<ResponsibilityEntity>>(
    states: BTreeMap<K, EntityState>,
    scope: ResponsibilityScope,
    recent_window: usize,
) -> Vec<ResponsibilityEvidence> {
    states
        .into_iter()
        .map(|(entity, state)| {
            let total = state.contributors.values().map(|value| value.count).sum::<usize>();
            let mut contributions = state
                .contributors
                .iter()
                .map(|(contributor, value)| Contribution {
                    contributor: contributor.clone(),
                    commit_count: value.count,
                    share: ratio(value.count, total),
                })
                .collect::<Vec<_>>();
            contributions.sort_by(|left, right| {
                right
                    .commit_count
                    .cmp(&left.commit_count)
                    .then_with(|| left.contributor.cmp(&right.contributor))
            });
            let mut recent = state
                .contributors
                .iter()
                .filter(|(_, value)| value.newest_age < recent_window)
                .map(|(contributor, value)| (value.newest_age, contributor.clone()))
                .collect::<Vec<_>>();
            recent.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
            let mut supporting = state
                .contributors
                .values()
                .flat_map(|value| value.commits.iter().cloned())
                .collect::<Vec<_>>();
            supporting
                .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
            supporting.dedup_by(|left, right| left.1 == right.1);
            ResponsibilityEvidence {
                entity: entity.into(),
                scope,
                attribution_basis: if scope == ResponsibilityScope::Symbol {
                    AttributionBasis::SemanticChange
                } else {
                    AttributionBasis::DocumentChange
                },
                concentration: ResponsibilityConcentration {
                    top_contributor_share: contributions.first().map_or(0.0, Contribution::share),
                    active_contributors: contributions.len(),
                },
                contributions,
                recent_contributors: recent
                    .into_iter()
                    .map(|(_, contributor)| contributor)
                    .collect(),
                supporting_commits: supporting.into_iter().map(|(_, commit)| commit).collect(),
            }
        })
        .collect()
}

#[allow(clippy::cast_precision_loss)]
fn ratio(numerator: usize, denominator: usize) -> f64 {
    numerator as f64 / denominator as f64
}

impl From<SemanticEntityKey> for ResponsibilityEntity {
    fn from(value: SemanticEntityKey) -> Self {
        Self::Symbol(value)
    }
}
impl From<PathBuf> for ResponsibilityEntity {
    fn from(value: PathBuf) -> Self {
        Self::File(value)
    }
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
) -> BTreeSet<SemanticEntityKey> {
    let mut symbols = BTreeSet::new();
    for change in diff.symbols().iter().filter(|change| change.kind() != ChangeKind::Unchanged) {
        if let Some(definition) = change.after().or(change.before()) {
            symbols.insert(entity_key(definition));
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
                after.graph().find_symbol(source).and_then(|node| node.definition()).or_else(
                    || before.graph().find_symbol(source).and_then(|node| node.definition()),
                );
            if let Some(definition) = definition {
                symbols.insert(entity_key(definition));
            }
        }
    }
    symbols
}

fn snapshot_symbols(snapshot: &SemanticIndexSnapshot) -> BTreeSet<SemanticEntityKey> {
    snapshot
        .documents()
        .values()
        .flat_map(|document| document.facts().facts())
        .filter_map(|record| match record.fact() {
            SemanticFact::Definition(definition) => Some(entity_key(definition)),
            _ => None,
        })
        .collect()
}

fn entity_key(definition: &SymbolDefinition) -> SemanticEntityKey {
    let identity = SemanticEntityIdentity::from_definition(definition)
        .expect("semantic definitions have valid identity fields");
    SemanticEntityKey {
        document: identity.document().to_path_buf(),
        kind: identity.kind(),
        qualified_name: identity.qualified_name().to_owned(),
    }
}

fn source_symbol(fact: &SemanticFact) -> Option<&branchsense_core::SymbolId> {
    match fact {
        SemanticFact::Contains(value) => Some(value.container()),
        SemanticFact::Call(value) => Some(value.caller()),
        SemanticFact::Reference(value) => Some(value.source()),
        SemanticFact::TypeRelation(value) => Some(value.source()),
        SemanticFact::Dependency(value) => Some(value.source()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributor_identity_is_conservative() {
        let first = Contributor::new("Alice Smith", " Alice@Example.com ");
        let second = Contributor::new("Alice Smith", "alice@company.com");
        assert_ne!(first, second);
        assert_eq!(first.email(), "alice@example.com");
        assert_eq!(first.redacted().email(), "[redacted]");
    }

    #[test]
    fn redaction_preserves_contribution_values() {
        let contributor = Contributor::new("Alice", "alice@example.com");
        let mut state = EntityState::default();
        state.attribute(&contributor, 0, GitCommitId::new("a").expect("valid ID"));
        let key = SemanticEntityKey {
            document: PathBuf::from("A.java"),
            kind: SymbolKind::Type,
            qualified_name: "A".into(),
        };
        let evidence =
            build_evidence(BTreeMap::from([(key, state)]), ResponsibilityScope::Symbol, 2)
                .pop()
                .expect("evidence");
        assert_eq!(evidence.contributions()[0].commit_count(), 1);
        assert!((evidence.contributions()[0].share() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn evidence_orders_contributors_and_computes_share() {
        let alice = Contributor::new("Alice", "alice@example.com");
        let bob = Contributor::new("Bob", "bob@example.com");
        let mut state = EntityState::default();
        state.attribute(&alice, 0, GitCommitId::new("a").expect("valid ID"));
        state.attribute(&alice, 1, GitCommitId::new("b").expect("valid ID"));
        state.attribute(&bob, 2, GitCommitId::new("c").expect("valid ID"));
        let key = SemanticEntityKey {
            document: PathBuf::from("A.java"),
            kind: SymbolKind::Type,
            qualified_name: "A".into(),
        };
        let evidence =
            build_evidence(BTreeMap::from([(key, state)]), ResponsibilityScope::Symbol, 2)
                .pop()
                .expect("evidence");
        assert_eq!(evidence.contributions()[0].contributor(), &alice);
        assert!((evidence.contributions()[0].share() - (2.0 / 3.0)).abs() < f64::EPSILON);
        assert_eq!(evidence.recent_contributors(), &[alice]);
    }

    #[test]
    fn options_reject_zero_recent_window() {
        assert_eq!(ResponsibilityOptions::new(4).with_recent_commits(0).recent_commits(), 0);
    }
}

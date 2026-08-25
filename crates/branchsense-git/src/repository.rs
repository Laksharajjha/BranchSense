//! Read-only repository, reference, and merge-base access.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use branchsense_core::RepositoryId;
use serde::{Deserialize, Serialize};

use crate::{GitCommitId, GitError, GitRevision, Result};

/// The kind of Git reference.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum GitRefKind {
    /// A local branch under `refs/heads`.
    LocalBranch,
    /// A remote-tracking branch under `refs/remotes`.
    RemoteBranch,
    /// A tag under `refs/tags`.
    Tag,
    /// Any other reference namespace.
    Other,
}

/// A resolved, read-only Git reference.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitRef {
    name: String,
    kind: GitRefKind,
    target: GitCommitId,
}

impl GitRef {
    fn new(name: String, kind: GitRefKind, target: GitCommitId) -> Self {
        Self { name, kind, target }
    }
    /// Returns the complete reference name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the reference category.
    #[must_use]
    pub const fn kind(&self) -> GitRefKind {
        self.kind
    }
    /// Returns the peeled commit identifier.
    #[must_use]
    pub fn target(&self) -> &GitCommitId {
        &self.target
    }
}

/// Stable identity for a discovered Git repository.
///
/// Git does not store a universal repository UUID. `BranchSense` therefore uses
/// the canonical common Git directory path as a stable local identity. It is
/// stable across repeated discovery from the same repository, but can change
/// when the repository is moved or cloned. This is intentionally not presented
/// as a cryptographic global identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepositoryIdentity {
    id: RepositoryId,
    worktree: Option<PathBuf>,
    git_dir: PathBuf,
}

impl RepositoryIdentity {
    fn from_repository(repository: &gix::Repository) -> Result<Self> {
        let git_dir = repository.git_dir().to_path_buf();
        let canonical_git_dir = std::fs::canonicalize(&git_dir).unwrap_or(git_dir);
        let id = RepositoryId::new(format!("repository:git:{}", canonical_git_dir.display()))
            .map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
        Ok(Self {
            id,
            worktree: repository.workdir().map(Path::to_path_buf),
            git_dir: canonical_git_dir,
        })
    }

    /// Returns the `BranchSense` repository identity.
    #[must_use]
    pub fn id(&self) -> &RepositoryId {
        &self.id
    }
    /// Returns the working tree, if this repository has one.
    #[must_use]
    pub fn worktree(&self) -> Option<&Path> {
        self.worktree.as_deref()
    }
    /// Returns the canonical Git directory.
    #[must_use]
    pub fn git_dir(&self) -> &Path {
        &self.git_dir
    }
}

/// A read-only handle to a discovered Git repository.
#[derive(Clone, Debug)]
pub struct GitRepository {
    repository: Rc<gix::Repository>,
    identity: RepositoryIdentity,
}

impl GitRepository {
    /// Discovers a repository from a path without modifying it.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is not inside a discoverable repository
    /// or when repository identity construction fails.
    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let repository =
            gix::discover(path).map_err(|error| GitError::Discovery(error.to_string()))?;
        let identity = RepositoryIdentity::from_repository(&repository)?;
        Ok(Self { repository: Rc::new(repository), identity })
    }

    /// Returns repository identity and paths.
    #[must_use]
    pub fn identity(&self) -> &RepositoryIdentity {
        &self.identity
    }
    /// Resolves `HEAD` to a commit.
    ///
    /// # Errors
    ///
    /// Returns an error when `HEAD` cannot be resolved to a commit.
    pub fn head(&self) -> Result<GitRevision> {
        let id = self.repository.head_id().map_err(GitError::operation)?;
        self.revision_from_id(id.detach())
    }
    /// Resolves a branch, ref, or revision expression to a commit.
    ///
    /// # Errors
    ///
    /// Returns an error when the expression cannot be resolved to a commit.
    pub fn resolve(&self, name: &str) -> Result<GitRevision> {
        let id = self.repository.rev_parse_single(name).map_err(GitError::operation)?;
        self.revision_from_id(id.detach())
    }
    /// Walks bounded commit history from a revision in newest-first order.
    ///
    /// The traversal reads commit objects only. It does not check out a
    /// branch, update refs, or modify the working tree.
    ///
    /// # Errors
    ///
    /// Returns an error when the revision cannot be represented as a Git object
    /// identifier or when ancestry traversal or commit metadata fails.
    pub fn history(&self, revision: &GitRevision, max_commits: usize) -> Result<Vec<GitRevision>> {
        let id = revision
            .commit_id()
            .as_str()
            .parse::<gix::ObjectId>()
            .map_err(|error| GitError::InvalidObjectId(error.to_string()))?;
        let walk = self
            .repository
            .rev_walk([id])
            .sorting(gix::revision::walk::Sorting::ByCommitTime(
                gix::traverse::commit::simple::CommitTimeOrder::NewestFirst,
            ))
            .all()
            .map_err(GitError::operation)?;
        walk.take(max_commits)
            .map(|info| {
                info.map_err(GitError::operation).and_then(|info| self.revision_from_id(info.id))
            })
            .collect()
    }
    /// Resolves a named ref to a peeled commit.
    ///
    /// # Errors
    ///
    /// Returns an error when the reference is missing or cannot be peeled to a
    /// commit.
    pub fn reference(&self, name: &str) -> Result<GitRef> {
        let mut reference = self.repository.find_reference(name).map_err(GitError::operation)?;
        let target = reference.peel_to_id().map_err(GitError::operation)?;
        let target = GitCommitId::from_gix(target.detach());
        let full_name = reference.name().as_bstr().to_string();
        Ok(GitRef::new(full_name.clone(), ref_kind(&full_name), target))
    }
    /// Lists local branches in deterministic reference-name order.
    ///
    /// # Errors
    ///
    /// Returns an error when Git reference storage cannot be read.
    pub fn local_branches(&self) -> Result<Vec<GitRef>> {
        self.references("refs/heads/")
    }
    /// Lists remote-tracking branches in deterministic reference-name order.
    ///
    /// # Errors
    ///
    /// Returns an error when Git reference storage cannot be read.
    pub fn remote_branches(&self) -> Result<Vec<GitRef>> {
        self.references("refs/remotes/")
    }
    /// Lists tags in deterministic reference-name order.
    ///
    /// # Errors
    ///
    /// Returns an error when Git reference storage cannot be read.
    pub fn tags(&self) -> Result<Vec<GitRef>> {
        self.references("refs/tags/")
    }
    /// Returns all best merge bases between two revisions.
    ///
    /// # Errors
    ///
    /// Returns an error when either revision is invalid or the commit graph
    /// cannot be traversed.
    pub fn merge_bases(&self, left: &GitRevision, right: &GitRevision) -> Result<MergeBaseResult> {
        let bases = self
            .repository
            .merge_bases_many(
                left.commit_id()
                    .as_str()
                    .parse::<gix::ObjectId>()
                    .map_err(|error| GitError::InvalidObjectId(error.to_string()))?,
                &[right
                    .commit_id()
                    .as_str()
                    .parse::<gix::ObjectId>()
                    .map_err(|error| GitError::InvalidObjectId(error.to_string()))?],
            )
            .map_err(GitError::operation)?;
        let mut revisions =
            bases.into_iter().map(|id| self.revision_from_id(id)).collect::<Result<Vec<_>>>()?;
        revisions.sort_by(|left, right| left.commit_id().cmp(right.commit_id()));
        Ok(match revisions.len() {
            0 => MergeBaseResult::None,
            1 => MergeBaseResult::Single(Box::new(revisions.remove(0))),
            _ => MergeBaseResult::Multiple(revisions),
        })
    }

    /// Reads Java source blobs from a commit tree without changing the
    /// working tree or Git index.
    ///
    /// # Errors
    ///
    /// Returns an error when the tree or a Java blob cannot be read, or when a
    /// Java blob is not valid UTF-8.
    pub fn java_sources(&self, revision: &GitRevision) -> Result<BTreeMap<PathBuf, String>> {
        let tree_id = revision
            .tree_id()
            .as_str()
            .parse::<gix::ObjectId>()
            .map_err(|error| GitError::InvalidObjectId(error.to_string()))?;
        let mut sources = BTreeMap::new();
        self.collect_java_sources(tree_id, Path::new(""), &mut sources)?;
        Ok(sources)
    }

    fn revision_from_id(&self, id: impl Into<gix::ObjectId>) -> Result<GitRevision> {
        self.repository
            .find_commit(id)
            .map_err(GitError::operation)
            .and_then(|commit| GitRevision::from_commit(&commit))
    }

    fn references(&self, prefix: &str) -> Result<Vec<GitRef>> {
        let mut references = self
            .repository
            .references()
            .map_err(GitError::operation)?
            .prefixed(prefix)
            .map_err(GitError::operation)?
            .map(|reference| {
                let mut reference = reference.map_err(GitError::operation)?;
                let name = reference.name().as_bstr().to_string();
                let target = reference.peel_to_id().map_err(GitError::operation)?;
                Ok(GitRef::new(
                    name.clone(),
                    ref_kind(&name),
                    GitCommitId::from_gix(target.detach()),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        references.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(references)
    }

    fn collect_java_sources(
        &self,
        tree_id: gix::ObjectId,
        prefix: &Path,
        sources: &mut BTreeMap<PathBuf, String>,
    ) -> Result<()> {
        let tree = self.repository.find_tree(tree_id).map_err(GitError::operation)?;
        for entry in tree.iter() {
            let entry = entry.map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
            let filename = entry.filename().to_string();
            let path = prefix.join(filename);
            if entry.inner.mode.is_tree() {
                self.collect_java_sources(entry.inner.oid.to_owned(), &path, sources)?;
            } else if entry.inner.mode.is_blob()
                && path.extension().is_some_and(|ext| ext == "java")
            {
                let blob = self
                    .repository
                    .find_blob(entry.inner.oid.to_owned())
                    .map_err(GitError::operation)?;
                let source = String::from_utf8(blob.data.clone())
                    .map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
                sources.insert(path, source);
            }
        }
        Ok(())
    }
}

/// The result of deterministic merge-base discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MergeBaseResult {
    /// No common ancestor exists.
    None,
    /// Exactly one best common ancestor exists.
    Single(Box<GitRevision>),
    /// Multiple best common ancestors exist and none was selected implicitly.
    Multiple(Vec<GitRevision>),
}

fn ref_kind(name: &str) -> GitRefKind {
    if name.starts_with("refs/heads/") {
        GitRefKind::LocalBranch
    } else if name.starts_with("refs/remotes/") {
        GitRefKind::RemoteBranch
    } else if name.starts_with("refs/tags/") {
        GitRefKind::Tag
    } else {
        GitRefKind::Other
    }
}

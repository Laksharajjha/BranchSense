//! Git revision to semantic snapshot integration.

use branchsense_core::{ProjectId, WorkspaceId};
use branchsense_index::{
    IndexOptions, IndexReport, RepositoryIdentity, RepositoryIndex, SemanticIndexSnapshot,
};

use crate::{GitError, GitRepository, GitRevision, Result};

/// A semantic index bound to the exact Git revision that supplied its source.
#[derive(Clone, Debug)]
pub struct GitSemanticSnapshot {
    repository: GitRepository,
    revision: GitRevision,
    semantic: SemanticIndexSnapshot,
    report: IndexReport,
}

impl GitSemanticSnapshot {
    pub(crate) fn new(
        repository: GitRepository,
        revision: GitRevision,
        semantic: SemanticIndexSnapshot,
        report: IndexReport,
    ) -> Self {
        Self { repository, revision, semantic, report }
    }

    /// Returns the Git repository handle used to load this snapshot.
    #[must_use]
    pub fn repository(&self) -> &GitRepository {
        &self.repository
    }
    /// Returns the exact commit revision that produced this snapshot.
    #[must_use]
    pub fn revision(&self) -> &GitRevision {
        &self.revision
    }
    /// Returns the reusable semantic index snapshot.
    #[must_use]
    pub fn semantic(&self) -> &SemanticIndexSnapshot {
        &self.semantic
    }
    /// Returns indexing diagnostics and counts.
    #[must_use]
    pub const fn report(&self) -> &IndexReport {
        &self.report
    }
}

/// Indexes immutable Git trees into BranchSense semantic snapshots.
#[derive(Clone, Debug, Default)]
pub struct GitSnapshotIndexer {
    options: IndexOptions,
}

impl GitSnapshotIndexer {
    /// Creates an indexer with default Java indexing options.
    #[must_use]
    pub fn new(options: IndexOptions) -> Self {
        Self { options }
    }

    /// Indexes a commit tree without checkout or working-tree mutation.
    pub fn index_revision(
        &self,
        repository: &GitRepository,
        revision: &GitRevision,
        previous: Option<&SemanticIndexSnapshot>,
    ) -> Result<GitSemanticSnapshot> {
        let sources = repository.java_sources(revision)?;
        let identity = semantic_identity(repository)?;
        let result = RepositoryIndex::new(self.options.clone())
            .index_sources(identity, sources, previous)
            .map_err(GitError::Index)?;
        let (semantic, report) = result.into_parts();
        Ok(GitSemanticSnapshot::new(repository.clone(), revision.clone(), semantic, report))
    }
}

fn semantic_identity(repository: &GitRepository) -> Result<RepositoryIdentity> {
    let id = repository.identity().id().clone();
    let workspace = WorkspaceId::new(format!("workspace:git:{id}"))
        .map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
    let project = ProjectId::new(format!("project:java:git:{id}"))
        .map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
    let root = repository.identity().worktree().unwrap_or(repository.identity().git_dir());
    Ok(RepositoryIdentity::new(root.to_path_buf(), id, workspace, project))
}

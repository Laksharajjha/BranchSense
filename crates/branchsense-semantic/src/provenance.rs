//! Provenance and identity metadata attached to semantic facts.

use branchsense_core::{DocumentId, ProjectId, RepositoryId, RevisionId, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::{Result, SemanticError};

/// A content-addressed source hash supplied by the host.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ContentHash(String);

impl ContentHash {
    /// Creates a content hash. The value should include its algorithm, for
    /// example `sha256:<hex>`, so consumers never need to infer it.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyValue`] for an empty value.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SemanticError::EmptyValue { kind: "content hash" });
        }
        Ok(Self(value))
    }

    /// Returns the serialized hash value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Identity of the adapter or extractor that produced facts.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProducerIdentity {
    name: String,
    version: String,
}

impl ProducerIdentity {
    /// Creates producer identity from a stable name and version.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyValue`] when either value is empty.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let version = version.into();
        if name.trim().is_empty() {
            return Err(SemanticError::EmptyValue { kind: "producer name" });
        }
        if version.trim().is_empty() {
            return Err(SemanticError::EmptyValue { kind: "producer version" });
        }
        Ok(Self { name, version })
    }

    /// Returns the producer name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the producer version.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Fingerprint of the extraction configuration used for a fact set.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ConfigurationFingerprint(String);

impl ConfigurationFingerprint {
    /// Creates a configuration fingerprint supplied by the host.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyValue`] for an empty value.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SemanticError::EmptyValue { kind: "configuration fingerprint" });
        }
        Ok(Self(value))
    }

    /// Returns the serialized fingerprint.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Provenance for all facts extracted from one source document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FactProvenance {
    repository_id: RepositoryId,
    workspace_id: WorkspaceId,
    project_id: Option<ProjectId>,
    document_id: DocumentId,
    revision_id: RevisionId,
    content_hash: ContentHash,
    producer: ProducerIdentity,
    configuration: Option<ConfigurationFingerprint>,
}

impl FactProvenance {
    /// Creates the minimum provenance required to identify a fact batch.
    #[must_use]
    pub fn new(
        repository_id: RepositoryId,
        workspace_id: WorkspaceId,
        document_id: DocumentId,
        revision_id: RevisionId,
        content_hash: ContentHash,
        producer: ProducerIdentity,
    ) -> Self {
        Self {
            repository_id,
            workspace_id,
            project_id: None,
            document_id,
            revision_id,
            content_hash,
            producer,
            configuration: None,
        }
    }

    /// Associates the facts with a project.
    #[must_use]
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Associates the facts with an extraction configuration fingerprint.
    #[must_use]
    pub fn with_configuration(mut self, configuration: ConfigurationFingerprint) -> Self {
        self.configuration = Some(configuration);
        self
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

    /// Returns the optional project identity.
    #[must_use]
    pub fn project_id(&self) -> Option<&ProjectId> {
        self.project_id.as_ref()
    }

    /// Returns the source document identity.
    #[must_use]
    pub fn document_id(&self) -> &DocumentId {
        &self.document_id
    }

    /// Returns the semantic revision identity.
    #[must_use]
    pub fn revision_id(&self) -> &RevisionId {
        &self.revision_id
    }

    /// Returns the source content hash.
    #[must_use]
    pub fn content_hash(&self) -> &ContentHash {
        &self.content_hash
    }

    /// Returns the producing adapter or extractor identity.
    #[must_use]
    pub fn producer(&self) -> &ProducerIdentity {
        &self.producer
    }

    /// Returns the optional extraction configuration fingerprint.
    #[must_use]
    pub fn configuration(&self) -> Option<&ConfigurationFingerprint> {
        self.configuration.as_ref()
    }
}

/// Identity of an immutable collection of facts for one workspace revision.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SnapshotIdentity {
    repository_id: RepositoryId,
    workspace_id: WorkspaceId,
    project_id: Option<ProjectId>,
    revision_id: RevisionId,
}

/// Common provenance for an analysis result.
///
/// All revision fields are optional because not every analysis is Git-backed.
/// The type is deliberately expressed in semantic IDs rather than Git types,
/// allowing local, editor, and future server callers to share the contract.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AnalysisProvenance {
    repository_id: Option<RepositoryId>,
    revision_id: Option<RevisionId>,
    base_revision_id: Option<RevisionId>,
    branch_a_revision_id: Option<RevisionId>,
    branch_b_revision_id: Option<RevisionId>,
    merge_base_revision_id: Option<RevisionId>,
    configuration: Option<ConfigurationFingerprint>,
    history_window: Option<usize>,
    producer: Option<ProducerIdentity>,
}

impl AnalysisProvenance {
    /// Creates empty provenance for a non-revision-pinned analysis.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            repository_id: None,
            revision_id: None,
            base_revision_id: None,
            branch_a_revision_id: None,
            branch_b_revision_id: None,
            merge_base_revision_id: None,
            configuration: None,
            history_window: None,
            producer: None,
        }
    }

    /// Sets the repository identity.
    #[must_use]
    pub fn with_repository(mut self, repository_id: RepositoryId) -> Self {
        self.repository_id = Some(repository_id);
        self
    }

    /// Sets the primary analysis revision.
    #[must_use]
    pub fn with_revision(mut self, revision_id: RevisionId) -> Self {
        self.revision_id = Some(revision_id);
        self
    }

    /// Sets the compared base revision.
    #[must_use]
    pub fn with_base_revision(mut self, revision_id: RevisionId) -> Self {
        self.base_revision_id = Some(revision_id);
        self
    }

    /// Sets branch revisions and their merge base.
    #[must_use]
    #[allow(clippy::similar_names)]
    pub fn with_branches(
        mut self,
        branch_a_revision_id: RevisionId,
        branch_b_revision_id: RevisionId,
        merge_base_revision_id: RevisionId,
    ) -> Self {
        self.branch_a_revision_id = Some(branch_a_revision_id);
        self.branch_b_revision_id = Some(branch_b_revision_id);
        self.merge_base_revision_id = Some(merge_base_revision_id);
        self
    }

    /// Sets the analysis configuration fingerprint.
    #[must_use]
    pub fn with_configuration(mut self, configuration: ConfigurationFingerprint) -> Self {
        self.configuration = Some(configuration);
        self
    }

    /// Sets the bounded history window.
    #[must_use]
    pub const fn with_history_window(mut self, history_window: usize) -> Self {
        self.history_window = Some(history_window);
        self
    }

    /// Sets the producing component identity.
    #[must_use]
    pub fn with_producer(mut self, producer: ProducerIdentity) -> Self {
        self.producer = Some(producer);
        self
    }

    /// Returns the repository identity, when available.
    #[must_use]
    pub fn repository_id(&self) -> Option<&RepositoryId> {
        self.repository_id.as_ref()
    }

    /// Returns the primary revision, when available.
    #[must_use]
    pub fn revision_id(&self) -> Option<&RevisionId> {
        self.revision_id.as_ref()
    }

    /// Returns the base revision, when available.
    #[must_use]
    pub fn base_revision_id(&self) -> Option<&RevisionId> {
        self.base_revision_id.as_ref()
    }

    /// Returns branch A's revision, when this is a branch comparison.
    #[must_use]
    pub fn branch_a_revision_id(&self) -> Option<&RevisionId> {
        self.branch_a_revision_id.as_ref()
    }

    /// Returns branch B's revision, when this is a branch comparison.
    #[must_use]
    pub fn branch_b_revision_id(&self) -> Option<&RevisionId> {
        self.branch_b_revision_id.as_ref()
    }

    /// Returns the common merge-base revision, when this is a branch comparison.
    #[must_use]
    pub fn merge_base_revision_id(&self) -> Option<&RevisionId> {
        self.merge_base_revision_id.as_ref()
    }

    /// Returns the configured history window, when available.
    #[must_use]
    pub const fn history_window(&self) -> Option<usize> {
        self.history_window
    }
}

impl SnapshotIdentity {
    /// Creates a revision-pinned snapshot identity.
    #[must_use]
    pub fn new(
        repository_id: RepositoryId,
        workspace_id: WorkspaceId,
        revision_id: RevisionId,
    ) -> Self {
        Self { repository_id, workspace_id, project_id: None, revision_id }
    }

    /// Associates the snapshot with a project.
    #[must_use]
    pub fn with_project(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Replaces the revision while retaining repository, workspace, and
    /// project identity.
    #[must_use]
    pub fn with_revision(mut self, revision_id: RevisionId) -> Self {
        self.revision_id = revision_id;
        self
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

    /// Returns the optional project identity.
    #[must_use]
    pub fn project_id(&self) -> Option<&ProjectId> {
        self.project_id.as_ref()
    }

    /// Returns the revision identity.
    #[must_use]
    pub fn revision_id(&self) -> &RevisionId {
        &self.revision_id
    }
}

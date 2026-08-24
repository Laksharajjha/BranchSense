//! Immutable Git revision and commit metadata types.

use std::fmt;

use branchsense_core::RevisionId;
use serde::{Deserialize, Serialize};

use crate::{GitError, Result};

/// A Git object identifier rendered in hexadecimal form.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GitCommitId(String);

impl GitCommitId {
    pub(crate) fn from_gix(id: gix::ObjectId) -> Self {
        Self(id.to_string())
    }

    /// Creates an object identifier from a non-empty hexadecimal string.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() || !value.chars().all(|character| character.is_ascii_hexdigit()) {
            return Err(GitError::InvalidObjectId(value));
        }
        Ok(Self(value))
    }

    /// Returns the hexadecimal object identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for GitCommitId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for GitCommitId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A Git tree object identifier.
pub type GitTreeId = GitCommitId;

/// Author or committer identity recorded on a Git commit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitSignature {
    name: String,
    email: String,
    timestamp_seconds: i64,
    offset_minutes: i32,
}

impl GitSignature {
    pub(crate) fn new(
        name: String,
        email: String,
        timestamp_seconds: i64,
        offset_minutes: i32,
    ) -> Self {
        Self { name, email, timestamp_seconds, offset_minutes }
    }

    /// Returns the recorded name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the recorded email.
    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }
    /// Returns the Unix timestamp in seconds.
    #[must_use]
    pub const fn timestamp_seconds(&self) -> i64 {
        self.timestamp_seconds
    }
    /// Returns the timezone offset in minutes.
    #[must_use]
    pub const fn offset_minutes(&self) -> i32 {
        self.offset_minutes
    }
}

/// Immutable metadata and ancestry for one Git commit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GitRevision {
    id: RevisionId,
    commit_id: GitCommitId,
    tree_id: GitTreeId,
    parents: Vec<GitCommitId>,
    author: GitSignature,
    committer: GitSignature,
    message: String,
}

impl GitRevision {
    pub(crate) fn from_commit(commit: gix::Commit<'_>) -> Result<Self> {
        let id = GitCommitId::from_gix(commit.id);
        let tree_id = GitCommitId::from_gix(
            commit
                .tree_id()
                .map_err(|error| GitError::InvalidMetadata(error.to_string()))?
                .detach(),
        );
        let parents = commit.parent_ids().map(|id| GitCommitId::from_gix(id.detach())).collect();
        let author = signature(
            commit.author().map_err(|error| GitError::InvalidMetadata(error.to_string()))?,
        )?;
        let committer = signature(
            commit.committer().map_err(|error| GitError::InvalidMetadata(error.to_string()))?,
        )?;
        let message = commit
            .message_raw()
            .map_err(|error| GitError::InvalidMetadata(error.to_string()))?
            .to_string();
        let revision_id = RevisionId::new(format!("git:commit:{}", id.as_str()))
            .map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
        Ok(Self { id: revision_id, commit_id: id, tree_id, parents, author, committer, message })
    }

    /// Returns the BranchSense revision identity.
    #[must_use]
    pub fn id(&self) -> &RevisionId {
        &self.id
    }
    /// Returns the Git commit object identifier.
    #[must_use]
    pub fn commit_id(&self) -> &GitCommitId {
        &self.commit_id
    }
    /// Returns the root tree object identifier.
    #[must_use]
    pub fn tree_id(&self) -> &GitTreeId {
        &self.tree_id
    }
    /// Returns the direct parents in commit order.
    #[must_use]
    pub fn parents(&self) -> &[GitCommitId] {
        &self.parents
    }
    /// Returns the author signature.
    #[must_use]
    pub fn author(&self) -> &GitSignature {
        &self.author
    }
    /// Returns the committer signature.
    #[must_use]
    pub fn committer(&self) -> &GitSignature {
        &self.committer
    }
    /// Returns the raw commit message decoded as UTF-8 lossily.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

fn signature(signature: gix::actor::SignatureRef<'_>) -> Result<GitSignature> {
    let time = signature.time().map_err(|error| GitError::InvalidMetadata(error.to_string()))?;
    Ok(GitSignature::new(
        signature.name.to_string(),
        signature.email.to_string(),
        time.seconds,
        time.offset / 60,
    ))
}

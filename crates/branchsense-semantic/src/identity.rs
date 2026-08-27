//! Conservative identities used to correlate semantic entities across revisions.

use std::path::{Path, PathBuf};

use branchsense_core::{ModelError, QualifiedName, SymbolId};
use serde::{Deserialize, Serialize};

use crate::{SymbolDefinition, SymbolKind};

/// A language-independent identity for a declaration across revisions.
///
/// The identity intentionally excludes opaque [`SymbolId`] values. It is a
/// conservative correlation key: a path, kind, and qualified name must all
/// agree before two declarations are considered the same entity. Renames,
/// moves, package changes, and ambiguous overloads therefore do not silently
/// acquire continuity.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SemanticEntityIdentity {
    document: PathBuf,
    kind: SymbolKind,
    qualified_name: String,
}

impl SemanticEntityIdentity {
    /// Creates an identity from its stable correlation fields.
    #[must_use]
    pub fn new(
        document: impl Into<PathBuf>,
        kind: SymbolKind,
        qualified_name: impl Into<String>,
    ) -> Self {
        Self { document: document.into(), kind, qualified_name: qualified_name.into() }
    }

    /// Derives a conservative identity from a semantic declaration.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] if a declaration without a qualified name has
    /// an invalid display name.
    pub fn from_definition(definition: &SymbolDefinition) -> Result<Self, ModelError> {
        let qualified_name = match definition.qualified_name() {
            Some(name) => name.clone(),
            None => QualifiedName::new(definition.name().as_str())?,
        };
        let qualified_name = qualified_name
            .as_str()
            .split_once('(')
            .map_or(qualified_name.as_str(), |(name, _)| name)
            .to_owned();
        Ok(Self::new(
            PathBuf::from(definition.location().document_id().as_str()),
            definition.kind(),
            qualified_name,
        ))
    }

    /// Returns the source document identity.
    #[must_use]
    pub fn document(&self) -> &Path {
        &self.document
    }

    /// Returns the declaration kind.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Returns the qualified semantic name.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
}

/// Result of attempting to correlate an entity across revisions.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum IdentityMatch {
    /// Exactly one candidate matched.
    Matched(SemanticEntityIdentity),
    /// More than one candidate is compatible with the available evidence.
    Ambiguous(Vec<SemanticEntityIdentity>),
    /// No candidate can be established conservatively.
    Unknown,
}

/// Converts a symbol identity into the canonical cross-revision identity.
///
/// # Errors
///
/// Returns [`ModelError`] if the declaration cannot provide a valid qualified
/// name.
pub fn canonical_identity(
    definition: &SymbolDefinition,
) -> Result<SemanticEntityIdentity, ModelError> {
    SemanticEntityIdentity::from_definition(definition)
}

/// Returns the opaque revision-local ID when one is available separately.
///
/// This helper exists to make accidental use of `SymbolId` for cross-revision
/// matching visible at call sites.
#[must_use]
pub fn revision_local_id(definition: &SymbolDefinition) -> &SymbolId {
    definition.id()
}

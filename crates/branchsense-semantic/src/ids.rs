//! Identifiers owned by the semantic vocabulary.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{Result, SemanticError};

/// Stable identity for a semantic fact within a revision.
///
/// A fact ID is assigned by the producer or a future canonicalization layer.
/// It is intentionally opaque so consumers do not depend on a hash or storage
/// format. Symbol identity remains [`branchsense_core::SymbolId`].
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FactId(String);

impl FactId {
    /// Creates a fact ID after rejecting empty values.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyValue`] when `value` is empty or only
    /// whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(SemanticError::EmptyValue { kind: "fact identifier" })
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the serialized identifier value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for FactId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for FactId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

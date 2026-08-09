//! Immutable semantic value objects.

#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]

use std::{
    fmt,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{errors::ModelError, ids::DocumentId};

/// An absolute path that identifies a repository workspace root.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct WorkspaceRoot(PathBuf);

impl WorkspaceRoot {
    /// Validates and stores an absolute workspace root.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::RelativeWorkspaceRoot`] for a relative path.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, ModelError> {
        let path = path.into();
        if path.is_absolute() {
            Ok(Self(path))
        } else {
            Err(ModelError::RelativeWorkspaceRoot { path: path.display().to_string() })
        }
    }

    /// Returns the validated path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Position {
    line: u32,
    column: u32,
    offset: u32,
}

impl Position {
    #[must_use]
    pub const fn new(line: u32, column: u32, offset: u32) -> Self {
        Self { line, column, offset }
    }
    #[must_use]
    pub const fn line(self) -> u32 {
        self.line
    }
    #[must_use]
    pub const fn column(self) -> u32 {
        self.column
    }
    #[must_use]
    pub const fn offset(self) -> u32 {
        self.offset
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Range {
    start: Position,
    end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Result<Self, ModelError> {
        if end < start { Err(ModelError::InvalidRange) } else { Ok(Self { start, end }) }
    }
    #[must_use]
    pub const fn start(self) -> Position {
        self.start
    }
    #[must_use]
    pub const fn end(self) -> Position {
        self.end
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Location {
    document_id: DocumentId,
    range: Range,
}

impl Location {
    #[must_use]
    pub fn new(document_id: DocumentId, range: Range) -> Self {
        Self { document_id, range }
    }
    #[must_use]
    pub fn document_id(&self) -> &DocumentId {
        &self.document_id
    }
    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Name(String);

impl Name {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(ModelError::EmptyValue { kind: "name" })
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct QualifiedName(String);

impl QualifiedName {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(ModelError::EmptyValue { kind: "qualified name" })
        } else {
            Ok(Self(value))
        }
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Language {
    Java,
    Kotlin,
    Go,
    Rust,
    TypeScript,
    Python,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Visibility {
    Public,
    Package,
    Protected,
    Private,
    Unspecified,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Modifier {
    Abstract,
    Final,
    Static,
    Synchronized,
    Native,
    Default,
    Sealed,
}

#[cfg(test)]
mod tests {
    use super::{Name, Position, QualifiedName, Range, WorkspaceRoot};
    use crate::ModelError;

    #[test]
    fn range_is_half_open_and_ordered() {
        let start = Position::new(2, 3, 10);
        let end = Position::new(2, 8, 15);
        let range = Range::new(start, end).expect("ordered positions are valid");

        assert_eq!(range.start(), start);
        assert_eq!(range.end(), end);
    }

    #[test]
    fn range_rejects_reversed_positions() {
        let result = Range::new(Position::new(4, 0, 20), Position::new(3, 0, 10));

        assert_eq!(result, Err(ModelError::InvalidRange));
    }

    #[test]
    fn names_reject_empty_values() {
        assert!(Name::new(" ").is_err());
        assert!(QualifiedName::new("").is_err());
    }

    #[test]
    fn workspace_root_rejects_relative_paths() {
        assert!(WorkspaceRoot::new("src").is_err());
        assert!(WorkspaceRoot::new("/workspace").is_ok());
    }
}

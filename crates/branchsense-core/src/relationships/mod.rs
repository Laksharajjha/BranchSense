//! Typed relationships between semantic entities.

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{
    ids::{
        BuildTargetId, DependencyId, DocumentId, ImportId, ModuleId, PackageId, ProjectId,
        SourceRootId, SymbolId, WorkspaceId,
    },
    value_objects::Location,
};

/// Strongly typed reference to an entity in a semantic snapshot.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum SemanticEntityId {
    Workspace(WorkspaceId),
    Project(ProjectId),
    SourceRoot(SourceRootId),
    Document(DocumentId),
    Package(PackageId),
    Module(ModuleId),
    Import(ImportId),
    Symbol(SymbolId),
    BuildTarget(BuildTargetId),
    Dependency(DependencyId),
}

/// A typed edge in the semantic model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SemanticRelationship {
    Contains { parent: SemanticEntityId, child: SemanticEntityId },
    Imports { document: DocumentId, import: ImportId },
    References { from: SymbolId, to: SymbolId, location: Location },
    DependsOn { from: SemanticEntityId, to: SemanticEntityId },
    Implements { type_symbol: SymbolId, interface_symbol: SymbolId },
    Extends { type_symbol: SymbolId, parent_symbol: SymbolId },
}

#[cfg(test)]
mod tests {
    use super::{SemanticEntityId, SemanticRelationship};
    use crate::{DocumentId, ImportId};

    #[test]
    fn import_relationship_preserves_typed_endpoints() {
        let document = DocumentId::new("document-1").expect("valid document id");
        let import = ImportId::new("import-1").expect("valid import id");
        let relationship =
            SemanticRelationship::Imports { document: document.clone(), import: import.clone() };

        assert_eq!(
            relationship,
            SemanticRelationship::Imports { document: document.clone(), import }
        );
        assert_eq!(
            SemanticEntityId::Document(document.clone()),
            SemanticEntityId::Document(document)
        );
    }
}

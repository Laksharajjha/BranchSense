//! Strongly typed identifiers used by the semantic model.

#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::errors::ModelError;

macro_rules! define_id {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
                let value = value.into();
                if value.trim().is_empty() {
                    Err(ModelError::EmptyValue { kind: $label })
                } else {
                    Ok(Self(value))
                }
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

define_id!(WorkspaceId, "workspace identifier");
define_id!(RepositoryId, "repository identifier");
define_id!(DocumentId, "document identifier");
define_id!(RevisionId, "revision identifier");
define_id!(ProjectId, "project identifier");
define_id!(PackageId, "package identifier");
define_id!(ModuleId, "module identifier");
define_id!(ImportId, "import identifier");
define_id!(SymbolId, "symbol identifier");
define_id!(BuildTargetId, "build target identifier");
define_id!(DependencyId, "dependency identifier");
define_id!(SourceRootId, "source root identifier");

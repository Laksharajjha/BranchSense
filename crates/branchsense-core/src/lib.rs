//! The immutable semantic domain model for `BranchSense`.
//!
//! This crate contains no parser, Git, graph, transport, or persistence
//! implementation. It defines stable values exchanged by future subsystems.

#![forbid(unsafe_code)]

pub mod domain;
pub mod entities;
pub mod errors;
pub mod ids;
pub mod relationships;
pub mod traits;
pub mod value_objects;

mod metadata;

pub use domain::{SemanticModel, SemanticSnapshot};
pub use entities::*;
pub use errors::{CoreError, ModelError, Result};
pub use ids::*;
pub use metadata::BuildInfo;
pub use relationships::{SemanticEntityId, SemanticRelationship};
pub use value_objects::*;

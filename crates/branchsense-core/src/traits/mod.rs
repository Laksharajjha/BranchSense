//! Traits defining stable capabilities of semantic domain values.

use crate::{ids::SymbolId, value_objects::Location};

/// Provides a stable semantic symbol identifier.
pub trait Identified {
    /// Returns the stable symbol identifier.
    fn symbol_id(&self) -> &SymbolId;
}

/// Provides a source location.
pub trait Located {
    /// Returns the source location.
    fn location(&self) -> &Location;
}

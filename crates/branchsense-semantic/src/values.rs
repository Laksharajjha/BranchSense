//! Immutable semantic value objects.

use branchsense_core::{Name, QualifiedName, SymbolId};
use serde::{Deserialize, Serialize};

use crate::{FactProvenance, Result, SemanticError};

/// The semantic role of a declared symbol.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum SymbolKind {
    /// A namespace-like scope.
    Namespace,
    /// A package or crate-level grouping.
    Package,
    /// A language module or source module.
    Module,
    /// A concrete class, struct, record, or equivalent type.
    Type,
    /// An interface, protocol, trait, or equivalent contract.
    Interface,
    /// An enumerated type.
    Enum,
    /// A free-standing callable.
    Function,
    /// A member callable.
    Method,
    /// A type constructor.
    Constructor,
    /// A declared data member.
    Field,
    /// A callable parameter.
    Parameter,
    /// An annotation, attribute, or decorator declaration.
    Annotation,
    /// A named member of an enum-like type.
    EnumVariant,
    /// A constant or language-specific named value.
    Constant,
}

/// A language-neutral reference to a type.
///
/// Resolution is optional because parsing and semantic resolution can be
/// separate phases. The qualified name is always retained for diagnostics and
/// deterministic fallback behavior.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TypeReference {
    name: QualifiedName,
    resolution: ResolutionState,
}

impl TypeReference {
    /// Creates an unresolved type reference.
    #[must_use]
    pub fn unresolved(name: QualifiedName) -> Self {
        Self { name, resolution: ResolutionState::Unresolved }
    }

    /// Creates a type reference with a resolved symbol identity.
    #[must_use]
    pub fn resolved(name: QualifiedName, symbol: SymbolId) -> Self {
        Self { name, resolution: ResolutionState::Resolved(symbol) }
    }

    /// Returns the referenced qualified name.
    #[must_use]
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// Returns the resolved symbol, when resolution succeeded.
    #[must_use]
    pub fn resolved_symbol(&self) -> Option<&SymbolId> {
        match &self.resolution {
            ResolutionState::Resolved(symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Returns the current type resolution state.
    #[must_use]
    pub const fn resolution(&self) -> &ResolutionState {
        &self.resolution
    }
}

/// A language-neutral reference to any symbol.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SymbolReference {
    name: QualifiedName,
    resolution: ResolutionState,
}

impl SymbolReference {
    /// Creates an unresolved symbol reference.
    #[must_use]
    pub fn unresolved(name: QualifiedName) -> Self {
        Self { name, resolution: ResolutionState::Unresolved }
    }

    /// Creates a symbol reference with a resolved identity.
    #[must_use]
    pub fn resolved(name: QualifiedName, symbol: SymbolId) -> Self {
        Self { name, resolution: ResolutionState::Resolved(symbol) }
    }

    /// Returns the referenced qualified name.
    #[must_use]
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// Returns the resolved symbol, when resolution succeeded.
    #[must_use]
    pub fn resolved_symbol(&self) -> Option<&SymbolId> {
        match &self.resolution {
            ResolutionState::Resolved(symbol) => Some(symbol),
            _ => None,
        }
    }

    /// Creates an ambiguous reference with all candidate symbols retained.
    #[must_use]
    pub fn ambiguous(name: QualifiedName, candidates: Vec<SymbolId>) -> Self {
        Self { name, resolution: ResolutionState::Ambiguous(candidates) }
    }

    /// Creates a reference to a symbol outside the indexed workspace.
    #[must_use]
    pub fn external(name: QualifiedName, external: ExternalSymbolId) -> Self {
        Self { name, resolution: ResolutionState::External(external) }
    }

    /// Creates a reference that could not be resolved because of a diagnostic.
    #[must_use]
    pub fn invalid(name: QualifiedName, reason: impl Into<String>) -> Self {
        Self { name, resolution: ResolutionState::Invalid { reason: reason.into() } }
    }

    /// Returns the current resolution state.
    #[must_use]
    pub const fn resolution(&self) -> &ResolutionState {
        &self.resolution
    }
}

/// Identity for a symbol provided by an external dependency or runtime.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ExternalSymbolId(String);

impl ExternalSymbolId {
    /// Creates an external symbol identity.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyValue`] for an empty value.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(SemanticError::EmptyValue { kind: "external symbol identifier" });
        }
        Ok(Self(value))
    }

    /// Returns the serialized external identity.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Resolution state of a semantic symbol reference.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum ResolutionState {
    /// The reference resolved to exactly one indexed symbol.
    Resolved(SymbolId),
    /// No target is currently known.
    Unresolved,
    /// Multiple indexed symbols remain possible targets.
    Ambiguous(Vec<SymbolId>),
    /// The target is known to exist outside the indexed workspace.
    External(ExternalSymbolId),
    /// Resolution failed because the source or analysis was invalid.
    Invalid {
        /// Diagnostic explaining why resolution was invalid.
        reason: String,
    },
}

/// Documentation attached to a semantic subject.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Documentation(String);

impl Documentation {
    /// Creates documentation text after rejecting empty values.
    ///
    /// # Errors
    ///
    /// Returns [`SemanticError::EmptyValue`] when `text` is empty or only
    /// whitespace.
    pub fn new(text: impl Into<String>) -> Result<Self> {
        let text = text.into();
        if text.trim().is_empty() {
            Err(SemanticError::EmptyValue { kind: "documentation" })
        } else {
            Ok(Self(text))
        }
    }

    /// Returns the documentation text without interpreting its markup.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A language-neutral annotation or attribute.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Annotation {
    name: QualifiedName,
    arguments: Vec<AnnotationArgument>,
}

impl Annotation {
    /// Creates an annotation with ordered arguments.
    #[must_use]
    pub fn new(name: QualifiedName, arguments: Vec<AnnotationArgument>) -> Self {
        Self { name, arguments }
    }

    /// Returns the annotation name.
    #[must_use]
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// Returns arguments in the order supplied by the adapter.
    #[must_use]
    pub fn arguments(&self) -> &[AnnotationArgument] {
        &self.arguments
    }
}

/// One positional or named annotation argument.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AnnotationArgument {
    name: Option<Name>,
    value: AnnotationValue,
}

impl AnnotationArgument {
    /// Creates a positional annotation argument.
    #[must_use]
    pub fn positional(value: AnnotationValue) -> Self {
        Self { name: None, value }
    }

    /// Creates a named annotation argument.
    #[must_use]
    pub fn named(name: Name, value: AnnotationValue) -> Self {
        Self { name: Some(name), value }
    }

    /// Returns the optional argument name.
    #[must_use]
    pub fn name(&self) -> Option<&Name> {
        self.name.as_ref()
    }

    /// Returns the argument value.
    #[must_use]
    pub const fn value(&self) -> &AnnotationValue {
        &self.value
    }
}

/// Values supported by language-neutral annotations.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum AnnotationValue {
    /// A string literal value.
    String(String),
    /// A numeric literal represented without loss of source precision.
    Number(String),
    /// A boolean value.
    Boolean(bool),
    /// A class, enum, or symbolic value reference.
    Symbol(SymbolReference),
    /// An ordered collection of annotation values.
    List(Vec<Self>),
}

/// The kind of semantic dependency represented by a fact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DependencyKind {
    /// A source-level import or use declaration.
    Import,
    /// A call from one callable to another.
    Call,
    /// A type annotation or type reference.
    TypeReference,
    /// A return type dependency.
    ReturnType,
    /// A parameter type dependency.
    ParameterType,
    /// A field type dependency.
    FieldType,
    /// An inheritance relation.
    Inheritance,
    /// An interface or trait implementation relation.
    Implementation,
    /// An annotation or attribute dependency.
    Annotation,
}

/// The semantic role of a symbol reference.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum ReferenceKind {
    /// A type is referenced.
    Type,
    /// A callable is invoked or referenced.
    Call,
    /// A field or named value is referenced.
    Value,
    /// An annotation or attribute is referenced.
    Annotation,
    /// A language-specific reference that has no more precise classification.
    Other,
}

/// A semantic batch with stable iteration order and no graph behavior.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SemanticFactSet {
    facts: Vec<crate::SemanticFactRecord>,
    provenance: Option<FactProvenance>,
}

impl SemanticFactSet {
    /// Creates a fact set from facts emitted by an adapter.
    #[must_use]
    pub fn new(facts: Vec<crate::SemanticFactRecord>) -> Self {
        Self { facts, provenance: None }
    }

    /// Attaches source provenance to this fact batch.
    #[must_use]
    pub fn with_provenance(mut self, provenance: FactProvenance) -> Self {
        self.provenance = Some(provenance);
        self
    }

    /// Returns facts in producer order.
    #[must_use]
    pub fn facts(&self) -> &[crate::SemanticFactRecord] {
        &self.facts
    }

    /// Returns the number of facts in this batch.
    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Returns whether this batch contains no facts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Returns provenance when the producer supplied it.
    #[must_use]
    pub fn provenance(&self) -> Option<&FactProvenance> {
        self.provenance.as_ref()
    }
}

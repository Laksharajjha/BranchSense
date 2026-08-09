//! Immutable semantic facts emitted by language adapters.

use branchsense_core::{DocumentId, Location, Modifier, Name, QualifiedName, SymbolId, Visibility};
use serde::{Deserialize, Serialize};

use crate::{
    Annotation, DependencyKind, Documentation, FactId, ReferenceKind, SymbolKind, SymbolReference,
    TypeReference,
};

/// A declaration and its language-neutral semantic metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SymbolDefinition {
    id: SymbolId,
    kind: SymbolKind,
    name: Name,
    qualified_name: Option<QualifiedName>,
    location: Location,
    container: Option<SymbolId>,
    visibility: Visibility,
    modifiers: Vec<Modifier>,
    documentation: Option<Documentation>,
    annotations: Vec<Annotation>,
}

impl SymbolDefinition {
    /// Creates a definition with conservative defaults for optional metadata.
    #[must_use]
    pub fn new(id: SymbolId, kind: SymbolKind, name: Name, location: Location) -> Self {
        Self {
            id,
            kind,
            name,
            qualified_name: None,
            location,
            container: None,
            visibility: Visibility::Unspecified,
            modifiers: Vec::new(),
            documentation: None,
            annotations: Vec::new(),
        }
    }

    /// Adds a qualified name while preserving immutability of the final value.
    #[must_use]
    pub fn with_qualified_name(mut self, name: QualifiedName) -> Self {
        self.qualified_name = Some(name);
        self
    }

    /// Associates the definition with its containing symbol.
    #[must_use]
    pub fn with_container(mut self, container: SymbolId) -> Self {
        self.container = Some(container);
        self
    }

    /// Sets visibility.
    #[must_use]
    pub const fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Adds a modifier.
    #[must_use]
    pub fn with_modifier(mut self, modifier: Modifier) -> Self {
        self.modifiers.push(modifier);
        self
    }

    /// Attaches documentation.
    #[must_use]
    pub fn with_documentation(mut self, documentation: Documentation) -> Self {
        self.documentation = Some(documentation);
        self
    }

    /// Attaches an annotation.
    #[must_use]
    pub fn with_annotation(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    /// Returns the stable symbol identity.
    #[must_use]
    pub fn id(&self) -> &SymbolId {
        &self.id
    }

    /// Returns the semantic symbol kind.
    #[must_use]
    pub const fn kind(&self) -> SymbolKind {
        self.kind
    }

    /// Returns the unqualified display name.
    #[must_use]
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the optional qualified name.
    #[must_use]
    pub fn qualified_name(&self) -> Option<&QualifiedName> {
        self.qualified_name.as_ref()
    }

    /// Returns the source location of the declaration.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// Returns the containing symbol, when known.
    #[must_use]
    pub fn container(&self) -> Option<&SymbolId> {
        self.container.as_ref()
    }

    /// Returns declaration visibility.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// Returns declaration modifiers.
    #[must_use]
    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }

    /// Returns attached documentation, when present.
    #[must_use]
    pub fn documentation(&self) -> Option<&Documentation> {
        self.documentation.as_ref()
    }

    /// Returns attached annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// A fact that a callable parameter exists and has a type.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ParameterFact {
    callable: SymbolId,
    parameter: SymbolDefinition,
    position: u32,
    parameter_type: TypeReference,
}

impl ParameterFact {
    /// Creates a parameter fact. The parameter definition should have kind
    /// [`SymbolKind::Parameter`].
    #[must_use]
    pub fn new(
        callable: SymbolId,
        parameter: SymbolDefinition,
        position: u32,
        parameter_type: TypeReference,
    ) -> Self {
        Self { callable, parameter, position, parameter_type }
    }

    /// Returns the owning callable.
    #[must_use]
    pub fn callable(&self) -> &SymbolId {
        &self.callable
    }

    /// Returns the parameter definition.
    #[must_use]
    pub fn parameter(&self) -> &SymbolDefinition {
        &self.parameter
    }

    /// Returns the zero-based semantic parameter position.
    #[must_use]
    pub const fn position(&self) -> u32 {
        self.position
    }

    /// Returns the parameter type.
    #[must_use]
    pub fn parameter_type(&self) -> &TypeReference {
        &self.parameter_type
    }
}

/// A fact that one callable returns a type.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReturnTypeFact {
    callable: SymbolId,
    return_type: TypeReference,
}

impl ReturnTypeFact {
    /// Creates a return-type fact. A language adapter may use a `void` or
    /// equivalent qualified name for procedures without a value.
    #[must_use]
    pub fn new(callable: SymbolId, return_type: TypeReference) -> Self {
        Self { callable, return_type }
    }

    /// Returns the callable identity.
    #[must_use]
    pub fn callable(&self) -> &SymbolId {
        &self.callable
    }

    /// Returns the return type.
    #[must_use]
    pub fn return_type(&self) -> &TypeReference {
        &self.return_type
    }
}

/// A fact that one symbol contains another symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContainsFact {
    container: SymbolId,
    member: SymbolId,
}

impl ContainsFact {
    /// Creates a containment fact.
    #[must_use]
    pub fn new(container: SymbolId, member: SymbolId) -> Self {
        Self { container, member }
    }

    /// Returns the containing symbol.
    #[must_use]
    pub fn container(&self) -> &SymbolId {
        &self.container
    }

    /// Returns the contained symbol.
    #[must_use]
    pub fn member(&self) -> &SymbolId {
        &self.member
    }
}

/// A callable invocation or semantic call edge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CallFact {
    caller: SymbolId,
    callee: SymbolReference,
    location: Location,
}

impl CallFact {
    /// Creates a call fact at the call-site location.
    #[must_use]
    pub fn new(calling_symbol: SymbolId, target: SymbolReference, location: Location) -> Self {
        Self { caller: calling_symbol, callee: target, location }
    }

    /// Returns the calling symbol.
    #[must_use]
    pub fn caller(&self) -> &SymbolId {
        &self.caller
    }

    /// Returns the called symbol reference.
    #[must_use]
    pub fn callee(&self) -> &SymbolReference {
        &self.callee
    }

    /// Returns the call-site location.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// A reference from one symbol to another semantic symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReferenceFact {
    source: SymbolId,
    target: SymbolReference,
    kind: ReferenceKind,
    location: Location,
}

impl ReferenceFact {
    /// Creates a typed symbol-reference fact.
    #[must_use]
    pub fn new(
        source: SymbolId,
        target: SymbolReference,
        kind: ReferenceKind,
        location: Location,
    ) -> Self {
        Self { source, target, kind, location }
    }

    /// Returns the referencing symbol.
    #[must_use]
    pub fn source(&self) -> &SymbolId {
        &self.source
    }

    /// Returns the referenced symbol.
    #[must_use]
    pub fn target(&self) -> &SymbolReference {
        &self.target
    }

    /// Returns the reference role.
    #[must_use]
    pub const fn kind(&self) -> ReferenceKind {
        self.kind
    }

    /// Returns the reference location.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// The relation between two type-like symbols.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TypeRelation {
    /// A type derives from or extends another type.
    Extends,
    /// A type fulfills an interface, trait, or protocol.
    Implements,
}

/// A type inheritance or implementation fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TypeRelationFact {
    source: SymbolId,
    target: SymbolReference,
    relation: TypeRelation,
    location: Location,
}

impl TypeRelationFact {
    /// Creates an inheritance or implementation fact.
    #[must_use]
    pub fn new(
        source: SymbolId,
        target: SymbolReference,
        relation: TypeRelation,
        location: Location,
    ) -> Self {
        Self { source, target, relation, location }
    }

    /// Returns the derived or implementing symbol.
    #[must_use]
    pub fn source(&self) -> &SymbolId {
        &self.source
    }

    /// Returns the parent or contract reference.
    #[must_use]
    pub fn target(&self) -> &SymbolReference {
        &self.target
    }

    /// Returns the relation kind.
    #[must_use]
    pub const fn relation(&self) -> TypeRelation {
        self.relation
    }

    /// Returns the relation location.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// An import or use declaration in a source document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportFact {
    document: DocumentId,
    target: QualifiedName,
    is_static: bool,
    location: Location,
}

impl ImportFact {
    /// Creates an import fact.
    #[must_use]
    pub fn new(
        document: DocumentId,
        target: QualifiedName,
        is_static: bool,
        location: Location,
    ) -> Self {
        Self { document, target, is_static, location }
    }

    /// Returns the importing document.
    #[must_use]
    pub fn document(&self) -> &DocumentId {
        &self.document
    }

    /// Returns the imported qualified name.
    #[must_use]
    pub fn target(&self) -> &QualifiedName {
        &self.target
    }

    /// Returns whether the import is static or equivalent to a member import.
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Returns the import location.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// Documentation attached to a symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentationFact {
    subject: SymbolId,
    documentation: Documentation,
}

impl DocumentationFact {
    /// Creates a documentation fact.
    #[must_use]
    pub fn new(subject: SymbolId, documentation: Documentation) -> Self {
        Self { subject, documentation }
    }

    /// Returns the documented symbol.
    #[must_use]
    pub fn subject(&self) -> &SymbolId {
        &self.subject
    }

    /// Returns the documentation value.
    #[must_use]
    pub fn documentation(&self) -> &Documentation {
        &self.documentation
    }
}

/// An annotation attached to a symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AnnotationFact {
    subject: SymbolId,
    annotation: Annotation,
    location: Location,
}

impl AnnotationFact {
    /// Creates an annotation fact.
    #[must_use]
    pub fn new(subject: SymbolId, annotation: Annotation, location: Location) -> Self {
        Self { subject, annotation, location }
    }

    /// Returns the annotated symbol.
    #[must_use]
    pub fn subject(&self) -> &SymbolId {
        &self.subject
    }

    /// Returns the annotation value.
    #[must_use]
    pub fn annotation(&self) -> &Annotation {
        &self.annotation
    }

    /// Returns the annotation location.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// A dependency fact that does not require a graph representation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DependencyFact {
    source: SymbolId,
    target: SymbolReference,
    kind: DependencyKind,
    location: Option<Location>,
}

impl DependencyFact {
    /// Creates a dependency fact with optional source provenance.
    #[must_use]
    pub fn new(
        source: SymbolId,
        target: SymbolReference,
        kind: DependencyKind,
        location: Option<Location>,
    ) -> Self {
        Self { source, target, kind, location }
    }

    /// Returns the dependent symbol.
    #[must_use]
    pub fn source(&self) -> &SymbolId {
        &self.source
    }

    /// Returns the dependency target.
    #[must_use]
    pub fn target(&self) -> &SymbolReference {
        &self.target
    }

    /// Returns the dependency kind.
    #[must_use]
    pub const fn kind(&self) -> DependencyKind {
        self.kind
    }

    /// Returns optional source provenance.
    #[must_use]
    pub fn location(&self) -> Option<&Location> {
        self.location.as_ref()
    }
}

/// A single semantic fact with an adapter-independent identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SemanticFact {
    /// A symbol definition.
    Definition(SymbolDefinition),
    /// A parameter declaration and type.
    Parameter(ParameterFact),
    /// A callable return type.
    ReturnType(ReturnTypeFact),
    /// A containment relationship.
    Contains(ContainsFact),
    /// A call-site relationship.
    Call(CallFact),
    /// A symbol reference.
    Reference(ReferenceFact),
    /// A type inheritance or implementation relationship.
    TypeRelation(TypeRelationFact),
    /// An import or use declaration.
    Import(ImportFact),
    /// Documentation attached to a symbol.
    Documentation(DocumentationFact),
    /// An annotation attached to a symbol.
    Annotation(AnnotationFact),
    /// A dependency relationship.
    Dependency(DependencyFact),
}

/// A semantic fact paired with a stable identity for diffing and transport.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SemanticFactRecord {
    id: FactId,
    fact: SemanticFact,
}

impl SemanticFactRecord {
    /// Creates an identified semantic fact.
    #[must_use]
    pub fn new(id: FactId, fact: SemanticFact) -> Self {
        Self { id, fact }
    }

    /// Returns the fact identity.
    #[must_use]
    pub fn id(&self) -> &FactId {
        &self.id
    }

    /// Returns the semantic fact payload.
    #[must_use]
    pub fn fact(&self) -> &SemanticFact {
        &self.fact
    }
}

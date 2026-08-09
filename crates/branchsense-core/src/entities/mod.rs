//! Immutable semantic entities.

#![allow(missing_docs)]
#![allow(clippy::struct_field_names)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    ids::{
        BuildTargetId, DependencyId, DocumentId, ImportId, ModuleId, PackageId, ProjectId,
        RevisionId, SourceRootId, SymbolId, WorkspaceId,
    },
    traits::{Identified, Located},
    value_objects::{Language, Location, Modifier, Name, QualifiedName, Visibility, WorkspaceRoot},
};

/// Repository workspace aggregate root.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Workspace {
    id: WorkspaceId,
    name: Name,
    root: WorkspaceRoot,
    current_revision: Option<RevisionId>,
    projects: Vec<ProjectId>,
}

impl Workspace {
    pub fn new(id: WorkspaceId, name: Name, root: WorkspaceRoot) -> Self {
        Self { id, name, root, current_revision: None, projects: Vec::new() }
    }
    #[must_use]
    pub fn with_current_revision(mut self, revision: RevisionId) -> Self {
        self.current_revision = Some(revision);
        self
    }
    pub fn id(&self) -> &WorkspaceId {
        &self.id
    }
    pub fn name(&self) -> &Name {
        &self.name
    }
    pub fn root(&self) -> &WorkspaceRoot {
        &self.root
    }
    pub fn current_revision(&self) -> Option<&RevisionId> {
        self.current_revision.as_ref()
    }
    pub fn projects(&self) -> &[ProjectId] {
        &self.projects
    }
}

/// Immutable revision identity for a workspace semantic snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkspaceRevision {
    id: RevisionId,
    workspace_id: WorkspaceId,
    parent: Option<RevisionId>,
    sequence: u64,
}

impl WorkspaceRevision {
    pub const fn new(
        id: RevisionId,
        workspace_id: WorkspaceId,
        parent: Option<RevisionId>,
        sequence: u64,
    ) -> Self {
        Self { id, workspace_id, parent, sequence }
    }
    pub fn id(&self) -> &RevisionId {
        &self.id
    }
    pub fn workspace_id(&self) -> &WorkspaceId {
        &self.workspace_id
    }
    pub fn parent(&self) -> Option<&RevisionId> {
        self.parent.as_ref()
    }
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// Source document descriptor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Document {
    id: DocumentId,
    path: PathBuf,
    language: Language,
    source_root: SourceRootId,
    revision: RevisionId,
}

impl Document {
    pub fn new(
        id: DocumentId,
        path: PathBuf,
        language: Language,
        source_root: SourceRootId,
        revision: RevisionId,
    ) -> Self {
        Self { id, path, language, source_root, revision }
    }
    pub fn id(&self) -> &DocumentId {
        &self.id
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub const fn language(&self) -> Language {
        self.language
    }
    pub fn source_root(&self) -> &SourceRootId {
        &self.source_root
    }
    pub fn revision(&self) -> &RevisionId {
        &self.revision
    }
}

/// Project discovered from repository or build metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Project {
    id: ProjectId,
    name: Name,
    language: Language,
    source_roots: Vec<SourceRootId>,
    build_targets: Vec<BuildTargetId>,
}

impl Project {
    pub fn new(id: ProjectId, name: Name, language: Language) -> Self {
        Self { id, name, language, source_roots: Vec::new(), build_targets: Vec::new() }
    }
    pub fn id(&self) -> &ProjectId {
        &self.id
    }
    pub fn name(&self) -> &Name {
        &self.name
    }
    pub const fn language(&self) -> Language {
        self.language
    }
    pub fn source_roots(&self) -> &[SourceRootId] {
        &self.source_roots
    }
    pub fn build_targets(&self) -> &[BuildTargetId] {
        &self.build_targets
    }
}

/// Source root within a project.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SourceRoot {
    id: SourceRootId,
    project_id: ProjectId,
    path: PathBuf,
    generated: bool,
}

impl SourceRoot {
    pub fn new(id: SourceRootId, project_id: ProjectId, path: PathBuf, generated: bool) -> Self {
        Self { id, project_id, path, generated }
    }
    pub fn id(&self) -> &SourceRootId {
        &self.id
    }
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub const fn generated(&self) -> bool {
        self.generated
    }
}

/// Named package.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Package {
    id: PackageId,
    project_id: ProjectId,
    name: QualifiedName,
}

impl Package {
    pub fn new(id: PackageId, project_id: ProjectId, name: QualifiedName) -> Self {
        Self { id, project_id, name }
    }
    pub fn id(&self) -> &PackageId {
        &self.id
    }
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }
}

/// Logical module within a package.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Module {
    id: ModuleId,
    package_id: PackageId,
    name: Name,
    language: Language,
}

impl Module {
    pub fn new(id: ModuleId, package_id: PackageId, name: Name, language: Language) -> Self {
        Self { id, package_id, name, language }
    }
    pub fn id(&self) -> &ModuleId {
        &self.id
    }
    pub fn package_id(&self) -> &PackageId {
        &self.package_id
    }
    pub fn name(&self) -> &Name {
        &self.name
    }
    pub const fn language(&self) -> Language {
        self.language
    }
}

/// Import declaration in a document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Import {
    id: ImportId,
    document_id: DocumentId,
    target: QualifiedName,
    is_static: bool,
    location: Location,
}

impl Import {
    pub fn new(
        id: ImportId,
        document_id: DocumentId,
        target: QualifiedName,
        is_static: bool,
        location: Location,
    ) -> Self {
        Self { id, document_id, target, is_static, location }
    }
    pub fn id(&self) -> &ImportId {
        &self.id
    }
    pub fn document_id(&self) -> &DocumentId {
        &self.document_id
    }
    pub fn target(&self) -> &QualifiedName {
        &self.target
    }
    pub const fn is_static(&self) -> bool {
        self.is_static
    }
    pub fn location(&self) -> &Location {
        &self.location
    }
}

/// Shared immutable metadata for all declarations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Declaration {
    id: SymbolId,
    name: Name,
    location: Location,
    visibility: Visibility,
    modifiers: Vec<Modifier>,
    annotations: Vec<Annotation>,
}

impl Declaration {
    pub fn new(
        id: SymbolId,
        name: Name,
        location: Location,
        visibility: Visibility,
        modifiers: Vec<Modifier>,
        annotations: Vec<Annotation>,
    ) -> Self {
        Self { id, name, location, visibility, modifiers, annotations }
    }
    pub fn id(&self) -> &SymbolId {
        &self.id
    }
    pub fn name(&self) -> &Name {
        &self.name
    }
    pub fn location(&self) -> &Location {
        &self.location
    }
    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }
    pub fn modifiers(&self) -> &[Modifier] {
        &self.modifiers
    }
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

impl Identified for Declaration {
    fn symbol_id(&self) -> &SymbolId {
        &self.id
    }
}
impl Located for Declaration {
    fn location(&self) -> &Location {
        &self.location
    }
}

/// Concrete class-like type kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TypeKind {
    Class,
    Record,
}

/// Class-like type declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TypeDeclaration {
    declaration: Declaration,
    kind: TypeKind,
    module_id: ModuleId,
}

impl TypeDeclaration {
    pub fn new(declaration: Declaration, kind: TypeKind, module_id: ModuleId) -> Self {
        Self { declaration, kind, module_id }
    }
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    pub const fn kind(&self) -> TypeKind {
        self.kind
    }
    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
}

/// Interface declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InterfaceDeclaration {
    declaration: Declaration,
    module_id: ModuleId,
    extends: Vec<SymbolId>,
}

impl InterfaceDeclaration {
    pub fn new(declaration: Declaration, module_id: ModuleId, extends: Vec<SymbolId>) -> Self {
        Self { declaration, module_id, extends }
    }
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub fn extends(&self) -> &[SymbolId] {
        &self.extends
    }
}

/// Enum declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnumDeclaration {
    declaration: Declaration,
    module_id: ModuleId,
    variants: Vec<Name>,
}

impl EnumDeclaration {
    pub fn new(declaration: Declaration, module_id: ModuleId, variants: Vec<Name>) -> Self {
        Self { declaration, module_id, variants }
    }
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }
    pub fn variants(&self) -> &[Name] {
        &self.variants
    }
}

/// Method declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MethodDeclaration {
    declaration: Declaration,
    return_type: Option<QualifiedName>,
    parameters: Vec<Parameter>,
}

impl MethodDeclaration {
    pub fn new(
        declaration: Declaration,
        return_type: Option<QualifiedName>,
        parameters: Vec<Parameter>,
    ) -> Self {
        Self { declaration, return_type, parameters }
    }
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    pub fn return_type(&self) -> Option<&QualifiedName> {
        self.return_type.as_ref()
    }
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
}

/// Constructor declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConstructorDeclaration {
    declaration: Declaration,
    parameters: Vec<Parameter>,
}

impl ConstructorDeclaration {
    pub fn new(declaration: Declaration, parameters: Vec<Parameter>) -> Self {
        Self { declaration, parameters }
    }
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
}

/// Field declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FieldDeclaration {
    declaration: Declaration,
    field_type: QualifiedName,
}

impl FieldDeclaration {
    pub fn new(declaration: Declaration, field_type: QualifiedName) -> Self {
        Self { declaration, field_type }
    }
    pub fn declaration(&self) -> &Declaration {
        &self.declaration
    }
    pub fn field_type(&self) -> &QualifiedName {
        &self.field_type
    }
}

/// Parameter declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Parameter {
    id: SymbolId,
    name: Name,
    parameter_type: QualifiedName,
    location: Location,
    annotations: Vec<Annotation>,
}

impl Parameter {
    pub fn new(
        id: SymbolId,
        name: Name,
        parameter_type: QualifiedName,
        location: Location,
        annotations: Vec<Annotation>,
    ) -> Self {
        Self { id, name, parameter_type, location, annotations }
    }
    pub fn id(&self) -> &SymbolId {
        &self.id
    }
    pub fn name(&self) -> &Name {
        &self.name
    }
    pub fn parameter_type(&self) -> &QualifiedName {
        &self.parameter_type
    }
    pub fn location(&self) -> &Location {
        &self.location
    }
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

impl Identified for Parameter {
    fn symbol_id(&self) -> &SymbolId {
        &self.id
    }
}
impl Located for Parameter {
    fn location(&self) -> &Location {
        &self.location
    }
}

/// Annotation attached to a declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Annotation {
    name: QualifiedName,
    location: Location,
}

impl Annotation {
    pub fn new(name: QualifiedName, location: Location) -> Self {
        Self { name, location }
    }
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }
    pub fn location(&self) -> &Location {
        &self.location
    }
}

impl Located for Annotation {
    fn location(&self) -> &Location {
        &self.location
    }
}

/// Build target within a project.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BuildTarget {
    id: BuildTargetId,
    project_id: ProjectId,
    name: Name,
}

impl BuildTarget {
    pub fn new(id: BuildTargetId, project_id: ProjectId, name: Name) -> Self {
        Self { id, project_id, name }
    }
    pub fn id(&self) -> &BuildTargetId {
        &self.id
    }
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }
    pub fn name(&self) -> &Name {
        &self.name
    }
}

/// Dependency from a build target to a semantic or external target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Dependency {
    id: DependencyId,
    source: BuildTargetId,
    target: QualifiedName,
    scope: DependencyScope,
}

impl Dependency {
    pub fn new(
        id: DependencyId,
        source: BuildTargetId,
        target: QualifiedName,
        scope: DependencyScope,
    ) -> Self {
        Self { id, source, target, scope }
    }
    pub fn id(&self) -> &DependencyId {
        &self.id
    }
    pub fn source(&self) -> &BuildTargetId {
        &self.source
    }
    pub fn target(&self) -> &QualifiedName {
        &self.target
    }
    pub const fn scope(&self) -> DependencyScope {
        self.scope
    }
}

/// Dependency visibility scope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DependencyScope {
    Compile,
    Runtime,
    Test,
    Provided,
    Unknown,
}

/// Union of all semantic entities stored in a snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SemanticEntity {
    Workspace(Workspace),
    WorkspaceRevision(WorkspaceRevision),
    Project(Project),
    SourceRoot(SourceRoot),
    Document(Document),
    Package(Package),
    Module(Module),
    Import(Import),
    Type(TypeDeclaration),
    Interface(InterfaceDeclaration),
    Enum(EnumDeclaration),
    Method(MethodDeclaration),
    Constructor(ConstructorDeclaration),
    Field(FieldDeclaration),
    Parameter(Parameter),
    Annotation(Annotation),
    BuildTarget(BuildTarget),
    Dependency(Dependency),
}

impl SemanticEntity {
    /// Returns the symbol identifier for declaration entities.
    pub fn symbol_id(&self) -> Option<&SymbolId> {
        match self {
            Self::Type(value) => Some(value.declaration().id()),
            Self::Interface(value) => Some(value.declaration().id()),
            Self::Enum(value) => Some(value.declaration().id()),
            Self::Method(value) => Some(value.declaration().id()),
            Self::Constructor(value) => Some(value.declaration().id()),
            Self::Field(value) => Some(value.declaration().id()),
            Self::Parameter(value) => Some(value.id()),
            _ => None,
        }
    }
}

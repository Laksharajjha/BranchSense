//! Java syntax-to-fact mapping implementation.

use std::collections::BTreeSet;

use branchsense_core::{
    DocumentId, Language, Location, Modifier, Name, Position, QualifiedName, Range, SymbolId,
    Visibility,
};
use branchsense_java::{JavaNode, JavaSyntaxTree};
use branchsense_parser::ParsedDocument;
use branchsense_semantic::{
    Annotation, AnnotationArgument, AnnotationFact, AnnotationValue, CallFact, ContainsFact,
    DependencyFact, DependencyKind, Documentation, DocumentationFact, FactId, ImportFact,
    ParameterFact, ReturnTypeFact, SemanticFact, SemanticFactRecord, SemanticFactSet,
    SymbolDefinition, SymbolKind, SymbolReference, TypeReference, TypeRelation, TypeRelationFact,
};

use crate::{ExtractionDiagnostic, ExtractionError, ExtractionResult, ExtractionSeverity, Result};

/// Deterministic Java semantic extractor.
#[derive(Clone, Debug, Default)]
pub struct JavaExtractor;

impl JavaExtractor {
    /// Creates a Java semantic extractor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Extracts semantic facts from a parsed Java document.
    ///
    /// Recoverable syntax problems are returned in the result diagnostics and
    /// do not prevent partial facts from being returned.
    ///
    /// # Errors
    ///
    /// Returns an error when the document is not Java, does not contain the
    /// Java adapter tree, does not retain source text, or contains an invalid
    /// core identity.
    pub fn extract(&self, document: &ParsedDocument) -> Result<ExtractionResult> {
        if document.language() != Language::Java {
            return Err(ExtractionError::LanguageMismatch { actual: document.language() });
        }
        if document.source().is_empty() {
            return Err(ExtractionError::SourceUnavailable);
        }
        let tree = document
            .syntax_tree()
            .as_any()
            .downcast_ref::<JavaSyntaxTree>()
            .ok_or(ExtractionError::SyntaxTreeMismatch)?;
        let document_id = DocumentId::new(document.path().display().to_string())?;
        let mut context = ExtractionContext::new(document.source(), document_id);
        context.collect_recovery_diagnostics(tree.root_node());
        context.extract_package(tree.root_node())?;
        context.visit(tree.root_node(), None, None)?;
        let diagnostics = std::mem::take(&mut context.diagnostics);
        Ok(ExtractionResult::new(context.finish(), diagnostics))
    }
}

struct ExtractionContext<'source> {
    source: &'source str,
    document_id: DocumentId,
    package: Option<String>,
    facts: FactBuilder,
    diagnostics: Vec<ExtractionDiagnostic>,
}

impl<'source> ExtractionContext<'source> {
    fn new(source: &'source str, document_id: DocumentId) -> Self {
        Self {
            source,
            document_id,
            package: None,
            facts: FactBuilder::default(),
            diagnostics: Vec::new(),
        }
    }

    fn finish(self) -> SemanticFactSet {
        SemanticFactSet::new(self.facts.records)
    }

    fn extract_package(&mut self, root: JavaNode<'_>) -> Result<()> {
        for index in 0..root.named_child_count() {
            let Some(child) = root.named_child(index) else { continue };
            if child.kind() != "package_declaration" {
                continue;
            }
            let package = clean_qualified_name(
                self.text(child).trim().trim_start_matches("package").trim().trim_end_matches(';'),
            );
            if package.is_empty() {
                self.warn("package declaration has an empty name", Some(child));
                continue;
            }
            let qualified = QualifiedName::new(package.clone())?;
            let name = Name::new(last_segment(&package))?;
            let definition = self.definition(
                child,
                SymbolKind::Package,
                name,
                qualified,
                None,
                Vec::new(),
                Vec::new(),
            )?;
            self.emit(
                "definition",
                &format!("package:{package}"),
                SemanticFact::Definition(definition),
            )?;
            self.package = Some(package);
            break;
        }
        Ok(())
    }

    fn visit(
        &mut self,
        node: JavaNode<'_>,
        container: Option<&SymbolId>,
        qualified_container: Option<&str>,
    ) -> Result<()> {
        match node.kind() {
            "package_declaration" => {}
            "import_declaration" => self.extract_import(node)?,
            "class_declaration" => {
                self.extract_type(node, SymbolKind::Type, container, qualified_container)?;
            }
            "interface_declaration" => {
                self.extract_type(node, SymbolKind::Interface, container, qualified_container)?;
            }
            "enum_declaration" => {
                self.extract_type(node, SymbolKind::Enum, container, qualified_container)?;
            }
            "method_declaration" => {
                self.extract_callable(node, SymbolKind::Method, container, qualified_container)?;
            }
            "constructor_declaration" => {
                self.extract_callable(
                    node,
                    SymbolKind::Constructor,
                    container,
                    qualified_container,
                )?;
            }
            "field_declaration" => self.extract_fields(node, container, qualified_container)?,
            "enum_constant" => {
                self.extract_enum_constant(node, container, qualified_container)?;
            }
            _ => self.visit_children(node, container, qualified_container)?,
        }
        Ok(())
    }

    fn visit_children(
        &mut self,
        node: JavaNode<'_>,
        container: Option<&SymbolId>,
        qualified_container: Option<&str>,
    ) -> Result<()> {
        for index in 0..node.named_child_count() {
            if let Some(child) = node.named_child(index) {
                self.visit(child, container, qualified_container)?;
            }
        }
        Ok(())
    }

    fn extract_import(&mut self, node: JavaNode<'_>) -> Result<()> {
        let raw = self.text(node);
        let target = raw
            .trim()
            .trim_start_matches("import")
            .trim()
            .trim_start_matches("static")
            .trim()
            .trim_end_matches(';')
            .trim()
            .to_owned();
        if target.is_empty() {
            self.warn("import declaration has no target", Some(node));
            return Ok(());
        }
        let location = self.location(node)?;
        let target_name = QualifiedName::new(clean_qualified_name(&target))?;
        let is_static = raw.contains("static");
        let fact = ImportFact::new(self.document_id.clone(), target_name, is_static, location);
        self.emit("import", &target, SemanticFact::Import(fact))
    }

    fn extract_type(
        &mut self,
        node: JavaNode<'_>,
        kind: SymbolKind,
        container: Option<&SymbolId>,
        qualified_container: Option<&str>,
    ) -> Result<()> {
        let Some(name) = self.node_name(node) else {
            self.warn("type declaration has no name", Some(node));
            return self.visit_children(node, container, qualified_container);
        };
        let qualified = join_name(self.package.as_deref(), qualified_container, &name);
        let (_, modifiers, annotations) = self.metadata(node);
        let definition = self.definition(
            node,
            kind,
            Name::new(name.clone())?,
            QualifiedName::new(qualified.clone())?,
            container,
            modifiers,
            annotations,
        )?;
        let symbol_id = definition.id().clone();
        self.emit_definition(&definition)?;
        self.extract_type_relations(node, &symbol_id)?;
        self.visit_children(node, Some(&symbol_id), Some(&qualified))?;
        Ok(())
    }

    fn extract_callable(
        &mut self,
        node: JavaNode<'_>,
        kind: SymbolKind,
        container: Option<&SymbolId>,
        qualified_container: Option<&str>,
    ) -> Result<()> {
        let Some(name) = self.node_name(node) else {
            self.warn("callable declaration has no name", Some(node));
            return self.visit_children(node, container, qualified_container);
        };
        let parameters = node.child_by_field_name("parameters");
        let signature = parameters.map(|value| self.parameter_signature(value)).unwrap_or_default();
        let qualified = join_name(self.package.as_deref(), qualified_container, &name);
        let qualified_with_signature = format!("{qualified}({signature})");
        let (_, modifiers, annotations) = self.metadata(node);
        let definition = self.definition(
            node,
            kind,
            Name::new(name)?,
            QualifiedName::new(qualified_with_signature.clone())?,
            container,
            modifiers,
            annotations,
        )?;
        let symbol_id = definition.id().clone();
        self.emit_definition(&definition)?;
        if let Some(type_node) = node.child_by_field_name("type") {
            let return_type = self.type_reference(type_node)?;
            self.emit(
                "return",
                &qualified_with_signature,
                SemanticFact::ReturnType(ReturnTypeFact::new(symbol_id.clone(), return_type)),
            )?;
        }
        if let Some(parameters) = parameters {
            self.extract_parameters(parameters, &symbol_id, &qualified_with_signature)?;
        }
        self.extract_calls(node, &symbol_id)?;
        self.visit_children(node, Some(&symbol_id), Some(&qualified_with_signature))?;
        Ok(())
    }

    fn extract_parameters(
        &mut self,
        parameters: JavaNode<'_>,
        callable: &SymbolId,
        callable_name: &str,
    ) -> Result<()> {
        for index in 0..parameters.named_child_count() {
            let Some(parameter) = parameters.named_child(index) else { continue };
            if !matches!(parameter.kind(), "formal_parameter" | "spread_parameter") {
                continue;
            }
            let Some(name) = self.node_name(parameter) else {
                self.warn("parameter has no name", Some(parameter));
                continue;
            };
            let type_node =
                parameter.child_by_field_name("type").or_else(|| parameter.named_child(0));
            let Some(type_node) = type_node else {
                self.warn("parameter has no type", Some(parameter));
                continue;
            };
            let parameter_type = self.type_reference(type_node)?;
            let key = format!("{callable_name}:parameter:{index}:{name}");
            let parameter_id = self.symbol_id(SymbolKind::Parameter, &key);
            let (visibility, modifiers, annotations) = self.metadata(parameter);
            let definition = self.definition_with_id(
                parameter,
                parameter_id,
                SymbolKind::Parameter,
                Name::new(name)?,
                None,
                Some(callable),
                visibility,
                modifiers,
                annotations,
            )?;
            self.emit_definition(&definition)?;
            self.emit(
                "parameter",
                &key,
                SemanticFact::Parameter(ParameterFact::new(
                    callable.clone(),
                    definition,
                    u32::try_from(index).unwrap_or(u32::MAX),
                    parameter_type,
                )),
            )?;
        }
        Ok(())
    }

    fn extract_fields(
        &mut self,
        node: JavaNode<'_>,
        container: Option<&SymbolId>,
        qualified_container: Option<&str>,
    ) -> Result<()> {
        let Some(type_node) = node.child_by_field_name("type") else {
            self.warn("field declaration has no type", Some(node));
            return Ok(());
        };
        let field_type = self.type_reference(type_node)?;
        let (_, modifiers, annotations) = self.metadata(node);
        for index in 0..node.named_child_count() {
            let Some(declarator) = node.named_child(index) else { continue };
            if declarator.kind() != "variable_declarator" {
                continue;
            }
            let Some(name) = self.node_name(declarator) else {
                self.warn("field declarator has no name", Some(declarator));
                continue;
            };
            let qualified = join_name(self.package.as_deref(), qualified_container, &name);
            let definition = self.definition(
                declarator,
                SymbolKind::Field,
                Name::new(name.clone())?,
                QualifiedName::new(qualified.clone())?,
                container,
                modifiers.clone(),
                annotations.clone(),
            )?;
            let field_id = definition.id().clone();
            self.emit_definition(&definition)?;
            self.emit(
                "field-type",
                &qualified,
                SemanticFact::Dependency(DependencyFact::new(
                    field_id,
                    SymbolReference::unresolved(field_type.name().clone()),
                    DependencyKind::FieldType,
                    self.location(declarator).ok(),
                )),
            )?;
        }
        Ok(())
    }

    fn extract_enum_constant(
        &mut self,
        node: JavaNode<'_>,
        container: Option<&SymbolId>,
        qualified_container: Option<&str>,
    ) -> Result<()> {
        let Some(name) = self.node_name(node) else {
            return Ok(());
        };
        let qualified = join_name(self.package.as_deref(), qualified_container, &name);
        let definition = self.definition(
            node,
            SymbolKind::EnumVariant,
            Name::new(name)?,
            QualifiedName::new(qualified)?,
            container,
            Vec::new(),
            Vec::new(),
        )?;
        self.emit_definition(&definition)
    }

    fn extract_type_relations(&mut self, node: JavaNode<'_>, source: &SymbolId) -> Result<()> {
        if let Some(superclass) = node
            .child_by_field_name("superclass")
            .or_else(|| direct_named_child(node, "superclass"))
        {
            self.emit_type_relation(superclass, source, TypeRelation::Extends)?;
        }
        if let Some(interfaces) = node
            .child_by_field_name("super_interfaces")
            .or_else(|| node.child_by_field_name("interfaces"))
            .or_else(|| direct_named_child(node, "super_interfaces"))
            .or_else(|| direct_named_child(node, "interfaces"))
        {
            for index in 0..interfaces.named_child_count() {
                if let Some(interface) = interfaces.named_child(index) {
                    self.emit_type_relation(interface, source, TypeRelation::Implements)?;
                }
            }
        }
        Ok(())
    }

    fn emit_type_relation(
        &mut self,
        node: JavaNode<'_>,
        source: &SymbolId,
        relation: TypeRelation,
    ) -> Result<()> {
        let target = node.named_child(0).map_or_else(|| self.text(node), |child| self.text(child));
        let target = clean_qualified_name(target);
        if target.is_empty() {
            return Ok(());
        }
        let location = self.location(node)?;
        let reference = SymbolReference::unresolved(QualifiedName::new(target.clone())?);
        let key = format!("{}:{target}", source.as_str());
        self.emit(
            "type-relation",
            &key,
            SemanticFact::TypeRelation(TypeRelationFact::new(
                source.clone(),
                reference,
                relation,
                location,
            )),
        )
    }

    fn extract_calls(&mut self, node: JavaNode<'_>, caller: &SymbolId) -> Result<()> {
        for index in 0..node.named_child_count() {
            let Some(child) = node.named_child(index) else { continue };
            if child.kind() == "method_invocation" {
                let Some(name_node) = child.child_by_field_name("name") else { continue };
                let name = clean_qualified_name(self.text(name_node));
                if name.is_empty() {
                    continue;
                }
                let target = if let Some(object) = child.child_by_field_name("object") {
                    let object = clean_qualified_name(self.text(object));
                    QualifiedName::new(format!("{object}.{name}"))?
                } else {
                    QualifiedName::new(name.clone())?
                };
                let key = format!("{}:{}:{}", caller.as_str(), child.start_byte(), name);
                self.emit(
                    "call",
                    &key,
                    SemanticFact::Call(CallFact::new(
                        caller.clone(),
                        SymbolReference::unresolved(target),
                        self.location(child)?,
                    )),
                )?;
            }
            self.extract_calls(child, caller)?;
        }
        Ok(())
    }

    fn emit_definition(&mut self, definition: &SymbolDefinition) -> Result<()> {
        let key = definition
            .qualified_name()
            .map_or_else(|| definition.name().as_str().to_owned(), |name| name.as_str().to_owned());
        self.emit("definition", &key, SemanticFact::Definition(definition.clone()))?;
        if let Some(container) = definition.container() {
            self.emit(
                "contains",
                &format!("{}:{}", container.as_str(), definition.id().as_str()),
                SemanticFact::Contains(ContainsFact::new(
                    container.clone(),
                    definition.id().clone(),
                )),
            )?;
        }
        if let Some(documentation) = definition.documentation() {
            self.emit(
                "documentation",
                definition.id().as_str(),
                SemanticFact::Documentation(DocumentationFact::new(
                    definition.id().clone(),
                    documentation.clone(),
                )),
            )?;
        }
        for (index, annotation) in definition.annotations().iter().enumerate() {
            self.emit(
                "annotation",
                &format!("{}:{index}", definition.id().as_str()),
                SemanticFact::Annotation(AnnotationFact::new(
                    definition.id().clone(),
                    annotation.clone(),
                    definition.location().clone(),
                )),
            )?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn definition(
        &self,
        node: JavaNode<'_>,
        kind: SymbolKind,
        name: Name,
        qualified: QualifiedName,
        container: Option<&SymbolId>,
        modifiers: Vec<Modifier>,
        annotations: Vec<Annotation>,
    ) -> Result<SymbolDefinition> {
        let id = self.symbol_id(kind, qualified.as_str());
        let (visibility, _, _) = self.metadata(node);
        self.definition_with_id(
            node,
            id,
            kind,
            name,
            Some(qualified),
            container,
            visibility,
            modifiers,
            annotations,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn definition_with_id(
        &self,
        node: JavaNode<'_>,
        id: SymbolId,
        kind: SymbolKind,
        name: Name,
        qualified: Option<QualifiedName>,
        container: Option<&SymbolId>,
        visibility: Visibility,
        modifiers: Vec<Modifier>,
        annotations: Vec<Annotation>,
    ) -> Result<SymbolDefinition> {
        let mut definition = SymbolDefinition::new(id, kind, name, self.location(node)?);
        if let Some(qualified) = qualified {
            definition = definition.with_qualified_name(qualified);
        }
        if let Some(container) = container {
            definition = definition.with_container(container.clone());
        }
        definition = definition.with_visibility(visibility);
        for modifier in modifiers {
            definition = definition.with_modifier(modifier);
        }
        if let Some(documentation) = self.documentation_before(node.start_byte()) {
            definition = definition.with_documentation(documentation);
        }
        for annotation in annotations {
            definition = definition.with_annotation(annotation);
        }
        Ok(definition)
    }

    fn type_reference(&self, node: JavaNode<'_>) -> Result<TypeReference> {
        let text = clean_qualified_name(self.text(node));
        Ok(TypeReference::unresolved(QualifiedName::new(text)?))
    }

    fn parameter_signature(&self, node: JavaNode<'_>) -> String {
        let mut parts = Vec::new();
        for index in 0..node.named_child_count() {
            if let Some(parameter) = node.named_child(index) {
                if let Some(type_node) = parameter.child_by_field_name("type") {
                    parts.push(clean_qualified_name(self.text(type_node)));
                }
            }
        }
        parts.join(",")
    }

    fn node_name(&self, node: JavaNode<'_>) -> Option<String> {
        let name_node = node.child_by_field_name("name").or_else(|| first_name_node(node))?;
        let text = self.text(name_node).trim();
        if text.is_empty() { None } else { Some(last_segment(text).to_owned()) }
    }

    fn metadata(&self, node: JavaNode<'_>) -> (Visibility, Vec<Modifier>, Vec<Annotation>) {
        let mut visibility = Visibility::Unspecified;
        let mut modifiers = Vec::new();
        let mut annotations = Vec::new();
        if let Some(modifier_node) =
            node.child_by_field_name("modifiers").or_else(|| direct_named_child(node, "modifiers"))
        {
            collect_metadata(
                modifier_node,
                self.source,
                &mut visibility,
                &mut modifiers,
                &mut annotations,
            );
        }
        if visibility == Visibility::Unspecified {
            visibility = Visibility::Package;
        }
        (visibility, modifiers, annotations)
    }

    fn location(&self, node: JavaNode<'_>) -> Result<Location> {
        let (start_line, start_column) = node.start_position();
        let (end_line, end_column) = node.end_position();
        let start = Position::new(
            u32::try_from(start_line).unwrap_or(u32::MAX),
            u32::try_from(start_column).unwrap_or(u32::MAX),
            u32::try_from(node.start_byte()).unwrap_or(u32::MAX),
        );
        let end = Position::new(
            u32::try_from(end_line).unwrap_or(u32::MAX),
            u32::try_from(end_column).unwrap_or(u32::MAX),
            u32::try_from(node.end_byte()).unwrap_or(u32::MAX),
        );
        Ok(Location::new(self.document_id.clone(), Range::new(start, end)?))
    }

    fn symbol_id(&self, kind: SymbolKind, qualified: &str) -> SymbolId {
        SymbolId::new(format!(
            "java:{}:{}:{}",
            self.document_id.as_str(),
            kind_tag(kind),
            qualified
        ))
        .expect("stable symbol identity has a non-empty prefix")
    }

    fn text(&self, node: JavaNode<'_>) -> &str {
        self.source.get(node.start_byte()..node.end_byte()).unwrap_or_default()
    }

    fn documentation_before(&self, start: usize) -> Option<Documentation> {
        let prefix = self.source.get(..start)?;
        let end = prefix.rfind("*/")? + 2;
        if !prefix[end..].trim().is_empty() {
            return None;
        }
        let begin = prefix[..end].rfind("/**")?;
        let content = &prefix[begin + 3..end - 2];
        let cleaned = content
            .lines()
            .map(|line| line.trim().trim_start_matches('*').trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Documentation::new(cleaned).ok()
    }

    fn collect_recovery_diagnostics(&mut self, node: JavaNode<'_>) {
        if self.diagnostics.len() < 256 && (node.is_error() || node.is_missing()) {
            let message = if node.is_missing() {
                format!("missing Java syntax node: {}", node.kind())
            } else {
                format!("recoverable Java syntax error near {}", node.kind())
            };
            let location = self.location(node).ok();
            self.diagnostics.push(ExtractionDiagnostic::new(
                ExtractionSeverity::Error,
                message,
                location,
            ));
        }
        for index in 0..node.child_count() {
            if let Some(child) = node.child(index) {
                self.collect_recovery_diagnostics(child);
            }
        }
    }

    fn warn(&mut self, message: impl Into<String>, node: Option<JavaNode<'_>>) {
        self.diagnostics.push(ExtractionDiagnostic::new(
            ExtractionSeverity::Warning,
            message,
            node.and_then(|value| self.location(value).ok()),
        ));
    }

    fn emit(&mut self, kind: &str, key: &str, fact: SemanticFact) -> Result<()> {
        let id = FactId::new(format!("java:{}:{kind}:{key}", self.document_id.as_str()))?;
        self.facts.push(id, fact);
        Ok(())
    }
}

#[derive(Default)]
struct FactBuilder {
    records: Vec<SemanticFactRecord>,
    ids: BTreeSet<String>,
}

impl FactBuilder {
    fn push(&mut self, id: FactId, fact: SemanticFact) {
        if self.ids.insert(id.as_str().to_owned()) {
            self.records.push(SemanticFactRecord::new(id, fact));
        }
    }
}

fn first_name_node(node: JavaNode<'_>) -> Option<JavaNode<'_>> {
    for index in 0..node.named_child_count() {
        let child = node.named_child(index)?;
        if matches!(child.kind(), "identifier" | "type_identifier") {
            return Some(child);
        }
    }
    None
}

fn direct_named_child<'tree>(node: JavaNode<'tree>, kind: &str) -> Option<JavaNode<'tree>> {
    for index in 0..node.named_child_count() {
        let child = node.named_child(index)?;
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

fn collect_metadata(
    node: JavaNode<'_>,
    source: &str,
    visibility: &mut Visibility,
    modifiers: &mut Vec<Modifier>,
    annotations: &mut Vec<Annotation>,
) {
    match node.kind() {
        "public" => *visibility = Visibility::Public,
        "protected" => *visibility = Visibility::Protected,
        "private" => *visibility = Visibility::Private,
        "static" => modifiers.push(Modifier::Static),
        "abstract" => modifiers.push(Modifier::Abstract),
        "final" => modifiers.push(Modifier::Final),
        "synchronized" => modifiers.push(Modifier::Synchronized),
        "native" => modifiers.push(Modifier::Native),
        "default" => modifiers.push(Modifier::Default),
        "sealed" => modifiers.push(Modifier::Sealed),
        "marker_annotation" | "annotation" => {
            if let Some(annotation) = parse_annotation(source, node) {
                annotations.push(annotation);
            }
        }
        _ => {}
    }
    for index in 0..node.named_child_count() {
        if let Some(child) = node.named_child(index) {
            collect_metadata(child, source, visibility, modifiers, annotations);
        }
    }
}

fn parse_annotation(source: &str, node: JavaNode<'_>) -> Option<Annotation> {
    let raw = source.get(node.start_byte()..node.end_byte())?.trim();
    let raw = raw.strip_prefix('@').unwrap_or(raw);
    let name = raw.split('(').next()?.trim();
    let arguments = raw
        .strip_prefix(name)
        .and_then(|value| value.strip_prefix('('))
        .and_then(|value| value.strip_suffix(')'))
        .map(|value| {
            value
                .split(',')
                .filter_map(|argument| {
                    let argument = argument.trim();
                    if argument.is_empty() {
                        return None;
                    }
                    let (name, value) =
                        argument.split_once('=').map_or((None, argument), |(name, value)| {
                            (Some(name.trim()), value.trim())
                        });
                    let value = if value == "true" || value == "false" {
                        AnnotationValue::Boolean(value == "true")
                    } else if value.starts_with('"') && value.ends_with('"') {
                        AnnotationValue::String(value[1..value.len() - 1].to_owned())
                    } else if value.chars().all(|character| character.is_ascii_digit()) {
                        AnnotationValue::Number(value.to_owned())
                    } else {
                        AnnotationValue::Symbol(SymbolReference::unresolved(
                            QualifiedName::new(clean_qualified_name(value)).ok()?,
                        ))
                    };
                    match name {
                        None => Some(AnnotationArgument::positional(value)),
                        Some(name) => Some(AnnotationArgument::named(Name::new(name).ok()?, value)),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Annotation::new(QualifiedName::new(name).ok()?, arguments))
}

fn clean_qualified_name(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("extends")
        .trim_start_matches("implements")
        .trim_end_matches(';')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("")
}

fn join_name(package: Option<&str>, container: Option<&str>, name: &str) -> String {
    match (package.filter(|value| !value.is_empty()), container.filter(|value| !value.is_empty())) {
        (_, Some(container)) => format!("{container}.{name}"),
        (Some(package), None) => format!("{package}.{name}"),
        (None, None) => name.to_owned(),
    }
}

fn last_segment(value: &str) -> &str {
    value.rsplit('.').next().unwrap_or(value)
}

fn kind_tag(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Namespace => "namespace",
        SymbolKind::Package => "package",
        SymbolKind::Module => "module",
        SymbolKind::Type => "type",
        SymbolKind::Interface => "interface",
        SymbolKind::Enum => "enum",
        SymbolKind::Function => "function",
        SymbolKind::Method => "method",
        SymbolKind::Constructor => "constructor",
        SymbolKind::Field => "field",
        SymbolKind::Parameter => "parameter",
        SymbolKind::Annotation => "annotation",
        SymbolKind::EnumVariant => "enum-variant",
        SymbolKind::Constant => "constant",
    }
}

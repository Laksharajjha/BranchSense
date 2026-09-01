//! Semantic snapshot comparison algorithm.

use std::collections::{BTreeMap, BTreeSet};

use branchsense_core::SymbolId;
use branchsense_git::GitSemanticSnapshot;
use branchsense_index::SemanticIndexSnapshot;
use branchsense_semantic::{
    AnalysisProvenance, DependencyKind, EvidenceCompleteness, EvidenceEnvelope, EvidenceIdentity,
    EvidenceKind, EvidenceState, FactId, ParameterFact, SemanticEntityIdentity, SemanticFact,
    SemanticFactRecord, SymbolDefinition, SymbolKind, TypeRelation,
};
use serde::{Deserialize, Serialize};

use crate::change::{
    ChangeKind, DiffStatistics, DocumentChange, FactChange, RelationshipChange, RelationshipKind,
    SymbolChange, SymbolChangeReason,
};

/// Compares two immutable repository snapshots.
#[derive(Clone, Copy, Debug, Default)]
pub struct SemanticDiffer;

impl SemanticDiffer {
    /// Creates a semantic differ.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Produces a deterministic semantic diff without modifying either input.
    #[must_use]
    pub fn diff(
        &self,
        before: &SemanticIndexSnapshot,
        after: &SemanticIndexSnapshot,
    ) -> SemanticDiff {
        let mut statistics = DiffStatistics::default();
        let documents = compare_documents(before, after, &mut statistics);
        let before_records = records(before);
        let after_records = records(after);
        let (facts, unchanged_facts, relationships) = compare_facts(before, after, &mut statistics);
        let symbols = compare_symbols(&before_records, &after_records, &mut statistics);

        let state = if documents.iter().any(|change| change.kind() != ChangeKind::Unchanged)
            || !facts.is_empty()
            || !relationships.is_empty()
        {
            EvidenceState::Observed
        } else {
            EvidenceState::NoEvidence
        };
        let provenance = AnalysisProvenance::new()
            .with_repository(before.identity().repository_id().clone())
            .with_base_revision(before.identity().revision_id().clone())
            .with_revision(after.identity().revision_id().clone());
        let mut evidence = EvidenceEnvelope::new(
            state,
            EvidenceCompleteness::new().with_semantic(state),
            provenance,
        );
        for change in &symbols {
            if let Some(definition) = change.after().or(change.before()) {
                if let Ok(identity) = SemanticEntityIdentity::from_definition(definition) {
                    evidence = evidence.with_identity(EvidenceIdentity::semantic(
                        EvidenceKind::Primary,
                        &identity,
                    ));
                }
            }
        }

        SemanticDiff {
            documents,
            symbols,
            facts,
            unchanged_facts,
            relationships,
            statistics,
            evidence,
        }
    }

    /// Compares two Git-backed snapshots using the same semantic diff engine.
    #[must_use]
    pub fn diff_git(
        &self,
        before: &GitSemanticSnapshot,
        after: &GitSemanticSnapshot,
    ) -> SemanticDiff {
        self.diff(before.semantic(), after.semantic())
    }
}

/// An immutable, deterministically ordered semantic change set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SemanticDiff {
    documents: Vec<DocumentChange>,
    symbols: Vec<SymbolChange>,
    facts: Vec<FactChange>,
    unchanged_facts: Vec<FactId>,
    relationships: Vec<RelationshipChange>,
    statistics: DiffStatistics,
    #[serde(default)]
    evidence: EvidenceEnvelope,
}

impl SemanticDiff {
    /// Returns document changes, including unchanged documents.
    #[must_use]
    pub fn documents(&self) -> &[DocumentChange] {
        &self.documents
    }

    /// Returns symbol comparisons, including unchanged stable symbols.
    #[must_use]
    pub fn symbols(&self) -> &[SymbolChange] {
        &self.symbols
    }

    /// Returns added, removed, and modified facts in stable order.
    #[must_use]
    pub fn facts(&self) -> &[FactChange] {
        &self.facts
    }

    /// Returns stable IDs of facts that were identical in both snapshots.
    #[must_use]
    pub fn unchanged_facts(&self) -> &[FactId] {
        &self.unchanged_facts
    }

    /// Returns relationship changes derived from fact changes.
    #[must_use]
    pub fn relationships(&self) -> &[RelationshipChange] {
        &self.relationships
    }

    /// Returns summary counts for this diff.
    #[must_use]
    pub const fn statistics(&self) -> &DiffStatistics {
        &self.statistics
    }

    /// Returns evidence state, provenance, and lineage for this diff.
    #[must_use]
    pub const fn evidence(&self) -> &EvidenceEnvelope {
        &self.evidence
    }

    /// Returns whether no semantic value changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
            && self.relationships.is_empty()
            && self.documents.iter().all(|change| change.kind() == ChangeKind::Unchanged)
            && self.symbols.iter().all(|change| change.kind() == ChangeKind::Unchanged)
    }
}

fn compare_documents(
    before: &SemanticIndexSnapshot,
    after: &SemanticIndexSnapshot,
    statistics: &mut DiffStatistics,
) -> Vec<DocumentChange> {
    let paths =
        before.documents().keys().chain(after.documents().keys()).cloned().collect::<BTreeSet<_>>();
    paths
        .into_iter()
        .map(|path| {
            let before_document = before.documents().get(&path);
            let after_document = after.documents().get(&path);
            let (kind, before_hash, after_hash) = match (before_document, after_document) {
                (None, Some(document)) => {
                    (ChangeKind::Added, None, Some(document.content_hash().as_str().to_owned()))
                }
                (Some(document), None) => {
                    (ChangeKind::Removed, Some(document.content_hash().as_str().to_owned()), None)
                }
                (Some(before), Some(after)) if before.content_hash() == after.content_hash() => (
                    ChangeKind::Unchanged,
                    Some(before.content_hash().as_str().to_owned()),
                    Some(after.content_hash().as_str().to_owned()),
                ),
                (Some(before), Some(after)) => (
                    ChangeKind::Modified,
                    Some(before.content_hash().as_str().to_owned()),
                    Some(after.content_hash().as_str().to_owned()),
                ),
                (None, None) => unreachable!("path comes from one snapshot"),
            };
            statistics.count_document(kind);
            DocumentChange::new(path, kind, before_hash, after_hash)
        })
        .collect()
}

fn records(snapshot: &SemanticIndexSnapshot) -> Vec<SemanticFactRecord> {
    snapshot
        .documents()
        .values()
        .flat_map(|document| document.facts().facts().iter().cloned())
        .collect()
}

fn compare_facts(
    before: &SemanticIndexSnapshot,
    after: &SemanticIndexSnapshot,
    statistics: &mut DiffStatistics,
) -> (Vec<FactChange>, Vec<FactId>, Vec<RelationshipChange>) {
    let paths =
        before.documents().keys().chain(after.documents().keys()).cloned().collect::<BTreeSet<_>>();
    let mut changes = Vec::new();
    let mut unchanged = Vec::new();
    let mut relationships = Vec::new();

    for path in paths {
        let before_facts =
            before.documents().get(&path).map(branchsense_index::IndexedDocument::facts);
        let after_facts =
            after.documents().get(&path).map(branchsense_index::IndexedDocument::facts);
        let before_map = fact_map(before_facts);
        let after_map = fact_map(after_facts);
        let ids = before_map.keys().chain(after_map.keys()).cloned().collect::<BTreeSet<_>>();
        for id in ids {
            let before_record = before_map.get(&id).copied();
            let after_record = after_map.get(&id).copied();
            let kind = match (&before_record, &after_record) {
                (None, Some(_)) => ChangeKind::Added,
                (Some(_), None) => ChangeKind::Removed,
                (Some(before), Some(after)) if before == after => ChangeKind::Unchanged,
                (Some(_), Some(_)) => ChangeKind::Modified,
                (None, None) => unreachable!("fact ID comes from one snapshot"),
            };
            statistics.count_fact(kind);
            if kind == ChangeKind::Unchanged {
                unchanged.push(id);
                continue;
            }
            let change = FactChange::new(
                path.clone(),
                id,
                kind,
                before_record.cloned(),
                after_record.cloned(),
            );
            if let Some(relationship) = relationship_kind(
                change.before().or(change.after()).expect("non-empty fact change").fact(),
            ) {
                statistics.count_relationship(kind);
                relationships.push(RelationshipChange::new(change.clone(), relationship));
            }
            changes.push(change);
        }
    }
    (changes, unchanged, relationships)
}

fn fact_map(
    facts: Option<&branchsense_semantic::SemanticFactSet>,
) -> BTreeMap<FactId, &SemanticFactRecord> {
    facts.map_or_else(BTreeMap::new, |set| {
        set.facts().iter().map(|record| (record.id().clone(), record)).collect()
    })
}

fn relationship_kind(fact: &SemanticFact) -> Option<RelationshipKind> {
    match fact {
        SemanticFact::Contains(_) => Some(RelationshipKind::Contains),
        SemanticFact::Call(_) => Some(RelationshipKind::Call),
        SemanticFact::Reference(_) => Some(RelationshipKind::Reference),
        SemanticFact::Import(_) => Some(RelationshipKind::Import),
        SemanticFact::TypeRelation(_) => Some(RelationshipKind::TypeRelation),
        SemanticFact::Dependency(_) => Some(RelationshipKind::Dependency),
        SemanticFact::Definition(_)
        | SemanticFact::Parameter(_)
        | SemanticFact::ReturnType(_)
        | SemanticFact::Documentation(_)
        | SemanticFact::Annotation(_) => None,
    }
}

fn compare_symbols(
    before_records: &[SemanticFactRecord],
    after_records: &[SemanticFactRecord],
    statistics: &mut DiffStatistics,
) -> Vec<SymbolChange> {
    let before = definitions(before_records);
    let after = definitions(after_records);
    let mut result = Vec::new();
    let mut before_unmatched = before.clone();
    let mut after_unmatched = after.clone();

    for id in before.keys().filter(|id| after.contains_key(*id)) {
        let before_definition = before.get(id).expect("definition exists");
        let after_definition = after.get(id).expect("definition exists");
        let kind = if before_definition == after_definition {
            ChangeKind::Unchanged
        } else {
            ChangeKind::Modified
        };
        statistics.count_symbol(kind);
        result.push(SymbolChange::new(
            Some((*id).clone()),
            Some((*id).clone()),
            kind,
            Some(before_definition.clone()),
            Some(after_definition.clone()),
            if kind == ChangeKind::Modified {
                symbol_reasons(before_definition, after_definition, before_records, after_records)
            } else {
                Vec::new()
            },
        ));
        before_unmatched.remove(id);
        after_unmatched.remove(id);
    }

    let before_anchors = grouped_anchors(&before_unmatched);
    let after_anchors = grouped_anchors(&after_unmatched);
    let anchors =
        before_anchors.keys().chain(after_anchors.keys()).cloned().collect::<BTreeSet<_>>();
    for anchor in anchors {
        let mut old = before_anchors.get(&anchor).cloned().unwrap_or_default();
        let mut new = after_anchors.get(&anchor).cloned().unwrap_or_default();
        old.sort_by(|left, right| left.0.cmp(&right.0));
        new.sort_by(|left, right| left.0.cmp(&right.0));
        if old.len() == 1 && new.len() == 1 {
            let (before_id, before_definition) = old.pop().expect("one old definition");
            let (after_id, after_definition) = new.pop().expect("one new definition");
            statistics.count_symbol(ChangeKind::Modified);
            result.push(SymbolChange::new(
                Some(before_id),
                Some(after_id),
                ChangeKind::Modified,
                Some(before_definition.clone()),
                Some(after_definition.clone()),
                symbol_reasons(
                    &before_definition,
                    &after_definition,
                    before_records,
                    after_records,
                ),
            ));
        } else {
            for (id, definition) in old {
                statistics.count_symbol(ChangeKind::Removed);
                result.push(SymbolChange::new(
                    Some(id),
                    None,
                    ChangeKind::Removed,
                    Some(definition),
                    None,
                    Vec::new(),
                ));
            }
            for (id, definition) in new {
                statistics.count_symbol(ChangeKind::Added);
                result.push(SymbolChange::new(
                    None,
                    Some(id),
                    ChangeKind::Added,
                    None,
                    Some(definition),
                    Vec::new(),
                ));
            }
        }
    }
    result.sort_by(symbol_change_order);
    result
}

fn definitions(records: &[SemanticFactRecord]) -> BTreeMap<SymbolId, SymbolDefinition> {
    records
        .iter()
        .filter_map(|record| match record.fact() {
            SemanticFact::Definition(definition) => {
                Some((definition.id().clone(), definition.clone()))
            }
            _ => None,
        })
        .collect()
}

fn grouped_anchors(
    definitions: &BTreeMap<SymbolId, SymbolDefinition>,
) -> BTreeMap<String, Vec<(SymbolId, SymbolDefinition)>> {
    let mut grouped = BTreeMap::new();
    for (id, definition) in definitions {
        grouped
            .entry(symbol_anchor(definition))
            .or_insert_with(Vec::new)
            .push((id.clone(), definition.clone()));
    }
    grouped
}

fn symbol_anchor(definition: &SymbolDefinition) -> String {
    let qualified = definition
        .qualified_name()
        .map_or_else(|| definition.name().as_str().to_owned(), |name| name.as_str().to_owned());
    let qualified = if matches!(definition.kind(), SymbolKind::Method | SymbolKind::Constructor) {
        qualified.split_once('(').map_or(qualified.as_str(), |(base, _)| base).to_owned()
    } else {
        qualified
    };
    format!("{}:{}:{}", definition.location().document_id(), definition.kind() as u8, qualified)
}

fn symbol_change_order(left: &SymbolChange, right: &SymbolChange) -> std::cmp::Ordering {
    symbol_sort_key(left).cmp(&symbol_sort_key(right))
}

fn symbol_sort_key(change: &SymbolChange) -> String {
    change
        .after()
        .or(change.before())
        .and_then(|definition| definition.qualified_name())
        .map_or_else(
            || {
                change
                    .after_id()
                    .or(change.before_id())
                    .map_or_else(String::new, ToString::to_string)
            },
            ToString::to_string,
        )
}

fn symbol_reasons(
    before: &SymbolDefinition,
    after: &SymbolDefinition,
    before_records: &[SemanticFactRecord],
    after_records: &[SemanticFactRecord],
) -> Vec<SymbolChangeReason> {
    let mut reasons = BTreeSet::new();
    if before.kind() != after.kind() || before.name() != after.name() {
        reasons.insert(SymbolChangeReason::DefinitionChanged);
    }
    if before.kind() == after.kind()
        && matches!(before.kind(), SymbolKind::Method | SymbolKind::Constructor)
        && before.qualified_name().map(ToString::to_string)
            != after.qualified_name().map(ToString::to_string)
    {
        reasons.insert(SymbolChangeReason::MethodSignatureChanged);
    }
    if before.visibility() != after.visibility() {
        reasons.insert(SymbolChangeReason::VisibilityChanged);
    }
    if sorted_modifiers(before) != sorted_modifiers(after) {
        reasons.insert(SymbolChangeReason::ModifierChanged);
    }
    if before.container() != after.container() {
        reasons.insert(SymbolChangeReason::ContainerChanged);
    }
    if before.documentation() != after.documentation() {
        reasons.insert(SymbolChangeReason::DocumentationChanged);
    }
    if before.annotations() != after.annotations() {
        reasons.insert(SymbolChangeReason::AnnotationChanged);
    }
    if return_type(before.id(), before_records) != return_type(after.id(), after_records) {
        reasons.insert(SymbolChangeReason::ReturnTypeChanged);
    }
    compare_parameters(before.id(), after.id(), before_records, after_records, &mut reasons);
    compare_field_types(before.id(), after.id(), before_records, after_records, &mut reasons);
    compare_relations(before.id(), after.id(), before_records, after_records, &mut reasons);
    if reasons.is_empty() {
        reasons.insert(SymbolChangeReason::DefinitionChanged);
    }
    reasons.into_iter().collect()
}

fn sorted_modifiers(definition: &SymbolDefinition) -> Vec<branchsense_core::Modifier> {
    let mut modifiers = definition.modifiers().to_vec();
    modifiers.sort_unstable();
    modifiers
}

fn return_type(id: &SymbolId, records: &[SemanticFactRecord]) -> Option<String> {
    records.iter().find_map(|record| match record.fact() {
        SemanticFact::ReturnType(fact) if fact.callable() == id => {
            Some(fact.return_type().name().to_string())
        }
        _ => None,
    })
}

fn parameters<'a>(
    id: &SymbolId,
    records: &'a [SemanticFactRecord],
) -> BTreeMap<u32, &'a ParameterFact> {
    records
        .iter()
        .filter_map(|record| match record.fact() {
            SemanticFact::Parameter(fact) if fact.callable() == id => Some((fact.position(), fact)),
            _ => None,
        })
        .collect()
}

fn compare_parameters(
    before_id: &SymbolId,
    after_id: &SymbolId,
    before_records: &[SemanticFactRecord],
    after_records: &[SemanticFactRecord],
    reasons: &mut BTreeSet<SymbolChangeReason>,
) {
    let before = parameters(before_id, before_records);
    let after = parameters(after_id, after_records);
    if before.len() < after.len() {
        reasons.insert(SymbolChangeReason::ParameterAdded);
    }
    if before.len() > after.len() {
        reasons.insert(SymbolChangeReason::ParameterRemoved);
    }
    for position in before.keys().filter(|position| after.contains_key(position)) {
        if before[position].parameter_type().name() != after[position].parameter_type().name() {
            reasons.insert(SymbolChangeReason::ParameterTypeChanged);
        }
    }
}

fn compare_field_types(
    before_id: &SymbolId,
    after_id: &SymbolId,
    before_records: &[SemanticFactRecord],
    after_records: &[SemanticFactRecord],
    reasons: &mut BTreeSet<SymbolChangeReason>,
) {
    let before = field_type(before_id, before_records);
    let after = field_type(after_id, after_records);
    if before != after && (before.is_some() || after.is_some()) {
        reasons.insert(SymbolChangeReason::FieldTypeChanged);
    }
}

fn field_type(id: &SymbolId, records: &[SemanticFactRecord]) -> Option<String> {
    records.iter().find_map(|record| match record.fact() {
        SemanticFact::Dependency(fact)
            if fact.source() == id && fact.kind() == DependencyKind::FieldType =>
        {
            Some(fact.target().name().to_string())
        }
        _ => None,
    })
}

fn compare_relations(
    before_id: &SymbolId,
    after_id: &SymbolId,
    before_records: &[SemanticFactRecord],
    after_records: &[SemanticFactRecord],
    reasons: &mut BTreeSet<SymbolChangeReason>,
) {
    let before = relations(before_id, before_records);
    let after = relations(after_id, after_records);
    if before.get(&TypeRelation::Extends) != after.get(&TypeRelation::Extends) {
        reasons.insert(SymbolChangeReason::SuperclassChanged);
    }
    if before.get(&TypeRelation::Implements).is_some_and(|values| {
        after.get(&TypeRelation::Implements).is_none_or(|other| values != other)
    }) {
        reasons.insert(SymbolChangeReason::InterfaceRemoved);
    }
    if after.get(&TypeRelation::Implements).is_some_and(|values| {
        before.get(&TypeRelation::Implements).is_none_or(|other| values != other)
    }) {
        reasons.insert(SymbolChangeReason::InterfaceAdded);
    }
}

fn relations(
    id: &SymbolId,
    records: &[SemanticFactRecord],
) -> BTreeMap<TypeRelation, BTreeSet<String>> {
    let mut result = BTreeMap::new();
    for record in records {
        if let SemanticFact::TypeRelation(fact) = record.fact() {
            if fact.source() == id {
                result
                    .entry(fact.relation())
                    .or_insert_with(BTreeSet::new)
                    .insert(fact.target().name().to_string());
            }
        }
    }
    result
}

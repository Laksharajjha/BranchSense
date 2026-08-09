# Java Semantic Extractor

`branchsense-extractor-java` translates the adapter-owned Java node surface
into the language-independent facts from `branchsense-semantic`.

## Mapping

| Java construct | Semantic output |
| --- | --- |
| `package` | `SymbolDefinition` with `SymbolKind::Package` |
| `import` | `ImportFact` |
| `class` | `SymbolDefinition` with `SymbolKind::Type` |
| `interface` | `SymbolDefinition` with `SymbolKind::Interface` |
| `enum` | `SymbolDefinition` with `SymbolKind::Enum` |
| Constructor | `SymbolDefinition` with `SymbolKind::Constructor` |
| Method | `SymbolDefinition` with `SymbolKind::Method` and `ReturnTypeFact` |
| Formal parameter | `ParameterFact` and parameter definition |
| Field declarator | `SymbolDefinition` with `SymbolKind::Field` |
| `extends` | `TypeRelationFact::Extends` |
| `implements` | `TypeRelationFact::Implements` |
| Nested declaration | `ContainsFact` |
| Javadoc | `DocumentationFact` |
| Annotation | `AnnotationFact` |
| Method invocation | `CallFact` |

Visibility and supported Java modifiers are stored on each definition. Type
and symbol references retain their qualified spelling and remain unresolved
when this file-local stage cannot prove an identity.

## Recovery

Error and missing nodes are emitted as structured extraction diagnostics. The
extractor continues traversing named children, so declarations before and
around malformed syntax remain available to later analysis.

## Boundary

The extractor does not import Tree-sitter, build a graph, resolve a classpath,
inspect Git, or predict conflicts. A future workspace semantic pass can
consume these facts and perform cross-file resolution without changing the
Java mapping contract.

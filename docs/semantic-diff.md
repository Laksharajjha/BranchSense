# Semantic Diff

`branchsense-diff` compares two immutable
`branchsense-index::SemanticIndexSnapshot` values and explains semantic
changes without parsing source or invoking Git.

## Why it exists

A textual diff describes changed lines. A semantic diff describes changed
documents, declarations, facts, and relationships. Moving a method without
changing its semantic facts does not create a symbol change, while adding a
parameter creates a signature change even when the surrounding source layout
is unchanged.

## Comparison model

`SemanticDiffer::diff` compares:

1. Repository-relative document paths and their content hashes.
2. Fact IDs and fact payloads from each indexed document.
3. Symbol definitions from definition facts.
4. Relationship facts such as calls, imports, references, type relations, and
   dependencies.

Documents and symbols are returned in deterministic order. Fact changes are
ordered by document path and fact ID. Unchanged fact IDs are available through
`unchanged_facts`; unchanged documents and stable symbols remain in their
respective collections with `ChangeKind::Unchanged`.

## Symbol changes

Stable symbol IDs are used whenever they remain stable. Java callable IDs
currently include the signature, so the differ conservatively pairs an old and
new callable only when their document, kind, and adapter-supplied qualified
name, including overload signatures, form an unambiguous anchor. It then reports structured reasons
such as `MethodSignatureChanged`, `ParameterAdded`, `ParameterRemoved`,
`ParameterTypeChanged`, `ReturnTypeChanged`, `VisibilityChanged`,
`ModifierChanged`, and `DocumentationChanged`.

Field type, inheritance, implementation, import, call, and reference changes
remain fact-level relationship changes. The differ does not infer semantic
meaning that the current fact model cannot prove.

## Rename limitation

Perfect rename detection is intentionally not implemented. A rename may appear
as one removed symbol and one added symbol unless the conservative callable
anchor can prove an unambiguous modification. Future Git-aware identity and
history analysis can improve rename attribution.

## Immutability and persistence

The differ clones only values needed in its result and never mutates either
input snapshot. Readers may continue using both snapshots after comparison.

Snapshots are currently produced in memory by `branchsense-index`; there is no
stable snapshot file format yet. A persistent `branchsense diff` command is
therefore intentionally deferred rather than implemented with a temporary
serialization format. Git integration will consume `SemanticDiff` after
revision and snapshot persistence are established.

## Performance

The initial implementation uses ordered maps and linear fact collection for
deterministic behavior. It is designed as a correctness baseline. The
`semantic_diff` benchmark measures empty and 100-file repository comparisons;
optimizations should be justified by those measurements before changing the
comparison strategy.

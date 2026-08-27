# Contributor Responsibility Evidence

`branchsense-ownership` reports historical contribution evidence for a bounded
Git history. It is intentionally not an ownership detector: a contributor
share describes observed changes attributed to a commit author, not expertise,
authority, current responsibility, or future behavior.

## Attribution

The analyzer walks the selected revision's history newest-first and compares
each commit with its first parent. A contributor receives at most one count per
entity per commit, regardless of how many semantic facts changed. Merge commits
use the first-parent comparison, matching the historical analysis policy.

Author identity is the pair of trimmed name and lower-cased email. Email case
normalization is safe and deterministic; different email addresses remain
different identities even when names match. GitHub accounts and external
services are not consulted.

## Scopes

Symbol evidence is emitted when the semantic diff identifies a changed
declaration or the source of a changed relationship. Cross-revision identity
uses document path, semantic kind, and a qualified name with signatures
removed. This avoids treating revision-specific `SymbolId` values as stable.

File evidence is emitted for changed documents and is kept in a separate
collection. It must not be read as symbol-level evidence. A file may change
without the available semantic diff establishing which declaration changed.

## Metrics

- **Commit count** is the number of distinct analyzed commits attributed to a
  contributor for one entity.
- **Contribution share** is `contributor commit count / total attributed commit
  counts` for that entity. Shares sum to one when evidence exists.
- **Concentration** reports the top contributor share and the number of active
  contributors. It is descriptive, not a risk score.
- **Recent contributors** are contributors whose newest attribution falls in
  the configurable recent commit window.

All result collections and nested contributor lists have deterministic ordering.
The analyzer is read-only and never checks out, resets, merges, rebases, or
updates repository state.

```sh
branchsense ownership --repo . --revision main --max-commits 500
branchsense ownership --repo . --revision main --max-commits 500 --json
```

Renames, moves, generated sources, and changes that cannot be mapped to a
semantic source remain limitations. The analyzer does not merge identities
based on similar names and does not reconstruct historical merge conflicts.

# Historical signals

Historical analysis provides independent evidence for future collision models.
It does not modify `CollisionAssessment`, calculate BCS, infer ownership, or
claim that a recent or frequently changed symbol will conflict.

## Architecture

```text
Git revision
    ↓ bounded read-only history
Git snapshots and semantic diffs
    ↓
HistoricalSignals
    ↓ future BCS input
```

The analyzer is pinned to an analysis revision and reports the exact number of
commits considered. It uses the existing Git tree snapshot indexer, so no
checkout, ref update, shell merge, database, or permanent cache is involved.

## Signals

### Change frequency

`ChangeFrequencySignal` counts the commits in the selected window that change
a semantic symbol. It reports the denominator, oldest observed revision, and
newest observed revision. It is evidence of activity, not a risk score.

### Recency

`RecencySignal` records the newest changing revision, its committer timestamp,
and age in analyzed commit positions. Commit age is deterministic and is the
primary comparison measure; timestamps are retained as Git metadata and are not
interpreted as elapsed wall-clock risk.

### Semantic co-change

`CoChangeSignal` counts pairs of semantic symbols changed in the same commit.
Each pair is counted at most once per commit and includes supporting revision
IDs. Symbol matching uses a best-effort key made from:

```text
repository-relative document path + symbol kind + signature-independent qualified name
```

This avoids assuming that revision-specific opaque `SymbolId` values are
stable forever. Signature suffixes are removed only for method matching;
renames, ambiguous declarations, and moves are not claimed to be identical.

### File co-change

`FileCoChangeSignal` is deliberately separate from symbol evidence. It is
calculated from changed document paths and must not be presented as proof that
the symbols inside those files are coupled.

### Hotness

No combined hotness score is emitted. Consumers can independently inspect
frequency and recency, while the future BCS work decides whether a validated
combination is justified.

## History windows

The current API uses a required positive `max_commits` bound. The default CLI
window is 500 commits:

```sh
branchsense history --repo . --revision main --max-commits 500
branchsense history --repo . --revision main --max-commits 500 --json
```

Each commit is compared with its first parent. Root commits use the symbols and
files present in their snapshot as initial changes. Merge commits are read in
Git's revision walk and compared to their first parent; this conservative rule
avoids inventing a combined-parent interpretation.

## Conflict history

The engine does not emit a historical conflict signal. Ordinary Git history
does not preserve every conflict developers encountered, and merge commits do
not reliably identify conflicts that occurred during their creation. A
trustworthy conflict source would need repository-integrated records or
external process data; fabricating it from commit messages would be unsafe.

## Determinism and performance

Results are sorted through ordered maps and carry supporting revision IDs. The
same repository revision and options produce the same serialized result.

Local release-profile benchmark medians for a one-file synthetic history were:

| Window | Median |
| ---: | ---: |
| 10 commits | 2.98 ms |
| 100 commits | 29.3 ms |
| 500 commits | 152 ms |
| 1000 commits | 309 ms |

These measurements include bounded snapshot/diff analysis and exclude process
startup. They are development baselines, not performance guarantees. The
linear cost is currently dominated by indexing historical Java snapshots;
persistence and incremental reuse remain future work.

## Boundaries

Historical signals remain independent from collision evidence. Ownership,
adaptive scoring, calibrated probability, AI prediction, and final BCS design
are intentionally not part of this milestone.

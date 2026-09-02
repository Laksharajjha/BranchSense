# Git Integration

BranchSense treats Git revisions as immutable semantic inputs. The
`branchsense-git` crate owns the boundary between gitoxide and BranchSense
domain types; callers do not receive `gix` objects.

## Identity

Git does not define a universal repository UUID. The first Git milestone uses
the canonical Git directory path as a local repository identity. Repeated
discovery of the same Git directory is stable. Moving or cloning a repository
may produce a different identity. This is deliberately not described as a
cryptographic global identity.

Filesystem indexing retains its existing path-scoped identity and is not
silently changed by Git support.

## Revisions and refs

`GitRevision` contains the commit ID, tree ID, parent IDs, author, committer,
and commit message. `GitRepository` resolves `HEAD`, branches, tags, remote
refs, and revision expressions through gitoxide.

Merge-base discovery returns no result, one result, or all best merge bases.
Multiple merge bases are never reduced to an arbitrary single commit.

## Git-backed snapshots

`GitSnapshotIndexer::index_revision` reads Java blobs directly from the commit
tree, then delegates parsing, extraction, graph construction, and reporting to
`branchsense-index`. It does not checkout or materialize files into the
working tree.

The returned `GitSemanticSnapshot` retains both the Git revision and the
existing `SemanticIndexSnapshot`. Persistence is intentionally deferred.

## CLI

```text
branchsense git info <path>
branchsense git branches <path>
branchsense git refs <path>
branchsense git merge-base <branch-a> <branch-b> --path <path>
branchsense diff --repo <path> --before <revision> --after <revision>
```

The diff command builds both snapshots in memory and delegates comparison to
`branchsense-diff`. It does not create a snapshot file.

## Guarantees and limitations

Git operations are read-only: no checkout, reset, merge, commit, branch
creation, ref update, index write, or working-tree write is performed. The
first implementation indexes Java blobs only. Binary or invalid UTF-8 Java
blobs are skipped with a per-file `Read` diagnostic while valid files remain
available for analysis; bytes are never lossily decoded. Persistent snapshots, Git-backed
fact provenance inside the existing index snapshot, and classpath-aware
resolution remain future work.

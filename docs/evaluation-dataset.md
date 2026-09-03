# BCS Evaluation Dataset

The BCS evaluation dataset is a future, versioned research artifact. It is not
telemetry, does not require network collection, and should begin as a small
curated set of reproducible repository histories.

## Record contract

Each record should contain:

- dataset version;
- repository identity;
- base revision, branch revisions, and merge base;
- semantic diffs, impacts, overlap, and collision evidence;
- historical and responsibility observations, where available;
- evidence states, completeness, and provenance;
- algorithm and configuration versions;
- ordinal assessment, when generated;
- separate textual-merge, build, test, and semantic-integration outcomes;
- manual conflict-resolution metadata;
- outcome confidence and label provenance.

Entire subsystem payloads need not be duplicated if immutable references to
content-addressed records can reproduce them. The record must nevertheless
retain the exact input and configuration identities required for replay.

## Outcomes

The following labels are independent:

- `textual_merge_conflict`: the recorded three-way Git merge required manual
  textual conflict resolution;
- `build_failure`: the merged result failed the defined build command;
- `test_failure`: the merged result failed the defined test scope;
- `semantic_integration_issue`: a separately adjudicated semantic integration
  problem.

One outcome does not prove another. General post-merge defects and unexplained
reverts are weak evidence unless their relationship to the branch interaction
is documented.

## Splitting

Use temporal splits before considering random splits. Development,
calibration, validation, and held-out test records should be ordered by the
prediction-time revision. Repositories or future time windows must not leak
into earlier predictions through history, contributors, generated features, or
duplicate branch pairs.

The first evaluation should measure ranking quality against textual merge
conflicts and separately report semantic integration outcomes. Probability
metrics are not applicable until a calibrated probability target and dataset
exist.


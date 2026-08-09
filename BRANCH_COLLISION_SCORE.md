# Branch Collision Score (BCS)

**Status:** Superseded research draft; retained for design history  
**Version:** 0.1  
**Category:** Deterministic semantic conflict prediction

> **Supersession notice:** The initial heuristic score in this document is not the
> current research proposal. Its skeptical review and replacement, BCS-C, are in
> `BCS_REVIEW_AND_REVISION.md`. BCS-C uses certificate-backed operation
> commutativity and explicit lower/upper risk bounds to avoid correlated-signal
> double counting and unsupported predictive claims.

## Abstract

Git detects textual merge conflicts only after two histories are brought together. BranchSense predicts **branch collisions** while work is still in progress: future integration failures, incompatible API evolution, duplicate implementation, and changes whose coordination cost is high even when Git can merge the text.

This document introduces the **Branch Collision Score (BCS)**, a deterministic score over two branch-local change capsules. A capsule is a revisioned summary of what a branch changed, what those changes semantically affect, and the evidence available to BranchSense. BCS combines direct mutation overlap, transitive semantic impact overlap, API exposure, architectural coupling, temporal concurrency, divergence, ownership coordination, and bounded historical friction. Its output is a 0–100 risk score, a separate 0–1 confidence value, and an additive explanation ledger.

BCS is neither a classifier nor a probability learned from data. It is a fixed, inspectable composition of evidence pathways. Every point is traceable to symbols, edges, revisions, or declared repository policy. The key idea is to compare **counterfactual merge surfaces** rather than changed lines.

## 1. Problem statement

At time `t`, let `a` and `b` be two Git workstreams: branches, worktrees, or unpushed developer overlays. Each is based on a commit in a Git DAG and has a current semantic workspace revision. BranchSense must estimate whether independently integrating their current work is likely to require coordination or produce a failure.

The required estimate is made before a three-way merge is attempted, and potentially before either workstream is pushed. The estimate must cover both:

- **textual collision:** the branches alter overlapping merge regions; and
- **semantic collision:** the branches modify the same declarations, depend on incompatible APIs, or perturb overlapping behavior through graph relationships.

BCS treats a textual collision as one kind of evidence, not the definition of risk. A clean merge that breaks callers is a collision. Two edits to different lines of a public method can be a collision. Two edits to the same comment usually are not.

## 2. Definitions

### 2.1 Semantic workspace graph

For a workspace revision `r`, let:

`G_r = (V_r, E_r, τ_V, τ_E, p)`

where `V_r` is the set of typed nodes, `E_r` is the set of typed directed edges, `τ_V` and `τ_E` assign node and edge kinds, and `p` contains typed properties. `G` is the shared fact store projected into Symbol and Dependency Graphs as defined in `ARCHITECTURE.md`.

Nodes include documents, packages, modules, declarations, build targets, and external symbols. Relevant edge kinds include references, calls, imports, exports, type use, inheritance, implementation, and build dependencies. Edges retain direction and provenance.

### 2.2 Change capsule

A **change capsule** is the semantic delta of a workstream relative to its merge base:

`K_a = (B_a, r_a, M_a, R_a, Φ_a, A_a, P_a, q_a)`

| Term | Meaning |
| --- | --- |
| `B_a` | Merge-base commit selected from the Git DAG for comparison with another workstream |
| `r_a` | Current workspace revision and content identities used to form the capsule |
| `M_a` | Typed mutation set, indexed by stable semantic entity identity |
| `R_a` | Directly changed semantic roots (symbols, modules, build targets, documents) |
| `Φ_a` | Sparse counterfactual merge surface: influence weights over affected graph entities |
| `A_a` | Public/API surface-change summary and compatibility classification |
| `P_a` | Provenance/coverage summary: parsed, resolved, generated, stale, or unavailable facts |
| `q_a` | Activity summary: observation times, edit velocity, and branch divergence metadata |

Capsules are not source snapshots. They can be shared with a collaboration service under the data policy described by the architecture: source text is unnecessary for the core score.

### 2.3 Typed mutation

For every changed root `x`, BranchSense emits a mutation record:

`m = (id(x), kind(m), scope(m), visibility(m), compatibility(m), δ(m), origin(m))`

`kind` is one of create, delete, rename, move, signature-change, contract-change, body-change, reference-change, import-change, build-change, generated-change, or documentation-only. `scope` is local, member, type, module, project, or workspace. `compatibility` is a deterministic adapter result: compatible, source-incompatible, binary-incompatible, behavior-unknown, or unknown. `δ(m)` is an adapter-defined severity in `[0,1]`, and `origin(m)` identifies file revision and source range.

The mutation model makes BCS sensitive to what changed, rather than merely where a diff occurred.

## 3. Counterfactual merge surface

The central BCS object is a branch’s **counterfactual merge surface** (CMS): the weighted set of entities whose assumptions could be affected if the branch is integrated.

For each changed root `s ∈ R_a` and target node `v`, define a path influence:

`I(s, v) = max_{π:s↝v, |π|≤h} [ ρ(s) · Π_{e∈π} λ(τ_E(e), dir(e)) · Π_{u∈π} κ(u) ]`

where:

| Variable | Meaning |
| --- | --- |
| `π` | A directed or permitted reverse semantic path from `s` to `v` |
| `h` | Fixed maximum traversal depth, set per analysis tier |
| `ρ(s)` | Root severity, derived from its mutation kinds and API exposure |
| `λ(edge kind, direction)` | Fixed attenuation for traversing an edge kind in a given direction |
| `κ(node)` | Node trust/availability factor in `[0,1]`; unresolved/external nodes attenuate influence |

`max`, rather than a sum over all paths, avoids path-count inflation in highly connected codebases. It answers: what is the strongest independently meaningful semantic route from the edit to this entity?

The surface weight is:

`Φ_a(v) = max_{s∈R_a} I(s, v)`

Only values above `ε` and the top `K` targets per root are retained. `Φ_a` is therefore a sparse vector. It includes changed roots at weight 1 and can include downstream callers, dependent modules, overridden contracts, build targets, and reverse dependencies depending on relation policy.

### 3.1 Relation policy

Each language adapter maps facts to a common relation policy. An illustrative—not normative—ordering is:

| Relation traversal | Typical attenuation `λ` | Rationale |
| --- | ---: | --- |
| Same symbol or declaration replacement | 1.00 | Direct shared semantic object |
| Override/implementation contract | 0.90 | Contract changes propagate strongly |
| Exported API to direct consumer | 0.85 | Caller assumptions are likely relevant |
| Type use or direct call | 0.75 | Strong but not necessarily breaking |
| Module/build dependency | 0.65 | Coarser relationship |
| Same architectural component | 0.45 | Useful weak proximity, not proof |
| Generic package proximity | 0.20 | Fallback only |

These are repository-policy constants, versioned in configuration and disclosed in explanations. They are not model parameters trained from outcomes. A project may override them only through reviewable configuration.

## 4. The Branch Collision Score

For a pair `(a, b)`, BranchSense computes nine normalized evidence signals `x_i(a,b) ∈ [0,1]`. Higher means stronger independent evidence of a future coordination collision.

`BCS(a,b) = 100 · [1 - Π_{i=1}^{9} (1 - w_i · x_i(a,b))]`

where each `w_i ∈ [0,1]` is a fixed policy weight and `Σw_i` is not required to equal one.

This **noisy-OR composition** is chosen for three properties:

1. A strong direct collision remains high even if other evidence is absent.
2. Several weak, distinct risk pathways accumulate without unbounded linear growth.
3. The marginal contribution of every signal is computable and explainable.

BCS is a risk index, not a calibrated claim that a merge will fail with `BCS%` probability. It ranks and prioritizes observed collision pressure.

### 4.1 Signal 1: direct semantic mutation collision `D`

`D(a,b) = Σ_{z∈Z} ω_z · min( μ_a(z), μ_b(z) ) / Σ_{z∈Z} ω_z · max( μ_a(z), μ_b(z) )`

`Z` is the union of stable semantic IDs and merge-region anchors touched by either capsule. `μ_a(z)` is the maximum compatibility-aware mutation severity for entity `z` on branch `a`. `ω_z` weights entity kinds: public contracts exceed private locals; generated artifacts are discounted.

If both branches replace the same method signature, `D` approaches 1. If they change separate private methods, it is 0. Rename and move hints map an old and a new identity into the same temporary equivalence class, preventing an artificial zero after refactoring.

**Why it exists:** direct shared mutations are the strongest predictor of a merge or semantic conflict. The weighted Jaccard form is bounded, symmetric, and resists a large branch diluting a specific collision with unrelated edits.

### 4.2 Signal 2: counterfactual merge-surface overlap `S`

`S(a,b) = Σ_{v∈V} η(v) · min(Φ_a(v), Φ_b(v)) / Σ_{v∈V} η(v) · max(Φ_a(v), Φ_b(v))`

`η(v)` weights semantic significance (e.g., exported symbols and build targets higher than locals). The sparse union of the two retained surfaces is evaluated; no full graph scan is required.

**Why it exists:** many collisions occur in different declarations that affect the same contracts or consumers. This detects overlapping intent without assuming that equal text is meaningful.

### 4.3 Signal 3: API contract pressure `A`

Let `E_a` be the public API entries altered by `a`. For an entry `e`, let `χ_a(e)` be its exposure severity: visibility, compatibility classification, number of resolved direct consumers, and whether the change is deletion/signature/contract/body-only. Let `U_b(e)` be branch `b`’s normalized reliance on `e`, derived from `Φ_b` and its direct mutation/reference records.

`A(a,b) = max( max_{e∈E_a} χ_a(e)·U_b(e), max_{e∈E_b} χ_b(e)·U_a(e) )`

An incompatible public signature change with active edits in direct consumers tends to 1. A private implementation change yields 0 unless it propagates through the surface signal.

**Why it exists:** a non-overlapping caller edit and callee API edit often merge cleanly but fail integration. Directionality is essential, so `A` evaluates both producer-to-consumer directions.

### 4.4 Signal 4: architectural coupling `C`

Partition the project graph into declared architecture components—modules, build targets, or configured boundaries. Let `comp(x)` be a root’s component and `d_c` a weighted component-graph distance. Define:

`C(a,b) = max_{r∈R_a, s∈R_b} [ ξ(r,s) · exp(-d_c(comp(r),comp(s))/τ_c) ]`

`ξ` is 1 for an explicit inter-component dependency, reduced for weak/package-only ties, and 0 for declared independent domains. Components owned by a common build target receive an explicit coupling edge.

**Why it exists:** graph resolution can be incomplete, particularly during build changes or language transitions. Architecture is a stable, low-resolution prior that catches risky changes before detailed resolution finishes. It is deliberately capped by `w_C` so it cannot dominate direct semantic evidence.

### 4.5 Signal 5: concurrent evolution `T`

Let `l_a` and `l_b` be most recent observation times, `ν_a` and `ν_b` recent semantic mutation rates in mutations/minute over a fixed window, and `W` the collaboration freshness window. Then:

`T(a,b) = exp(-|l_a-l_b|/τ_t) · min(1, sqrt(ν_a·ν_b)/ν_ref)`

The time component is zeroed when no branch activity is known within `W`. Open local overlays and recently synchronized workstreams can contribute; an inactive historical branch cannot.

**Why it exists:** the same semantic overlap is more actionable when both sides are actively changing. The geometric mean prevents one noisy branch from making a quiet branch look concurrent.

### 4.6 Signal 6: divergence pressure `G`

Let `d_a` and `d_b` be the number of non-merge commits from their selected base and `f_a`, `f_b` be normalized semantic footprint sizes. Define:

`G(a,b) = 1 - exp(-[(d_a+d_b)/τ_d + log(1+f_a+f_b)/τ_f])`

**Why it exists:** integration becomes less predictable as independent histories and semantic scope grow. `G` is weak evidence: it raises urgency for existing overlap, but it must not claim risk for two distant, unrelated branches by itself.

### 4.7 Signal 7: ownership coordination deficit `O`

For the affected surface intersection `J = supp(Φ_a) ∩ supp(Φ_b)`, let `owner(v)` be the configured code-owner distribution for `v`. Let `sim` be weighted distribution overlap and `H(J)` normalized ownership entropy.

`O(a,b) = S(a,b) · [1 - sim(owner_a(J), owner_b(J))] · [0.5 + 0.5·H(J)]`

**Why it exists:** semantic overlap maintained by different groups is more likely to require unplanned coordination. Multiplying by `S` avoids penalizing teams merely because they own adjacent components. Ownership is advisory; absence of ownership data reduces confidence rather than being treated as “no owner.”

### 4.8 Signal 8: historical friction `H`

For a stable scope key `k` (ordered component pair plus mutation-kind pair), let `c_k` be prior confirmed collisions and `n_k` the number of observed completed integrations in a fixed retention window. With fixed Laplace pseudocounts `α=1`, `β=4`:

`h(k) = (c_k + α) / (n_k + α + β)`

`H(a,b) = S(a,b) · h(k(a,b)) · min(1, n_k/n_min)`

Only events explicitly confirmed by a merge conflict, failed integration test causally linked to an API change, or maintainer resolution are counted. User dismissals are never silently treated as negative examples.

**Why it exists:** some boundaries are mechanically fragile despite similar graph structure. The sample gate prevents a small history from creating a strong prior. This is deterministic descriptive accounting, not learned weight fitting.

### 4.9 Signal 9: textual merge-anchor collision `L`

`L(a,b)` is a weighted Jaccard overlap of Git-compatible changed merge anchors: file identity, parent declaration, and normalized hunk context. Comments and formatting-only changes are given near-zero severity; a deletion/rewrite of the same declaration is high.

**Why it exists:** BCS must not discard the inexpensive, reliable evidence Git already provides. It is intentionally only one signal so that text does not overpower semantics.

## 5. Normalization and policy weights

All signals are bounded by construction. Counts are transformed with `1-exp(-x/τ)` or `log(1+x)` before composition; raw LOC, reference count, and commit count never enter directly. This prevents monorepo size or generated code from mechanically inflating risk.

The default policy is:

| Signal | Weight | Reason for ceiling |
| --- | ---: | --- |
| `D` direct mutation collision | 0.78 | Strongest immediate evidence |
| `S` merge-surface overlap | 0.64 | Principal semantic prediction signal |
| `A` API contract pressure | 0.72 | Breaking integrations are high impact |
| `C` architectural coupling | 0.24 | Useful but coarse prior |
| `T` concurrent evolution | 0.22 | Prioritizes action, not semantics |
| `G` divergence pressure | 0.16 | Scope proxy; never dominant |
| `O` ownership deficit | 0.18 | Coordination evidence, not code evidence |
| `H` historical friction | 0.14 | Bounded to prevent path dependence |
| `L` textual anchor collision | 0.52 | Strong known signal, subordinate to semantics |

Weights are versioned policy constants chosen through design review, synthetic adversarial cases, and held-out **evaluation**, not through optimization against repository outcomes. A repository can publish an alternative policy profile, but profiles must name changed weights and rationale. The engine reports the policy version with every score.

The explanation ledger gives each signal’s marginal contribution:

`contrib_i = 100 · w_i x_i · Π_{j≠i}(1-w_j x_j)`

Because noisy-OR has overlap among evidence pathways, contributions do not sum exactly to BCS. The response therefore also exposes ordered counterfactual deltas: “score without signal `i`.”

## 6. Confidence is separate from score

Risk and observability are different. A low score from unresolved code must not be read as safety. Let `c_i(a,b)` be each signal’s evidence coverage: fraction of contributing roots/paths whose parse, resolution, branch freshness, and ownership facts meet the signal’s requirements. Define:

`Conf(a,b) = Σ_i α_i · c_i / Σ_i α_i`

where `α_i` are fixed evidence-importance weights, with direct semantic and API coverage largest. `Conf` is reported in `[0,1]`, alongside explicit coverage gaps.

Examples:

- A score of 12 with confidence 0.96 means BranchSense saw strong evidence of low collision pressure.
- A score of 12 with confidence 0.31 means the score is provisional; missing classpath or unseen branch activity may change it.

The UI and API must never label low-confidence low-risk results as “safe.” Suggested states are `high-risk`, `watch`, `low-observed-risk`, and `insufficient-evidence`.

## 7. Incremental algorithm

### 7.1 Persistent indexes

Each workspace maintains these revisioned, incrementally updated indexes:

- `MutationIndex`: semantic entity/merge anchor → active branch capsules that mutate it.
- `SurfaceIndex`: graph entity/component → branch capsules with nonzero `Φ` weight.
- `ApiConsumerIndex`: public API entry → branch capsules that mutate or rely on it.
- `ComponentIndex`: component → active branch capsules and component-neighbor capsules.
- `ActivityIndex`: active branch → timestamp and rate summary.
- `OwnershipIndex` and `FrictionIndex`: declared ownership and bounded confirmed history.

Index values are compact branch handles with quantized surface weights. Exact values remain in capsules. All index updates are committed with the workspace revision that generated them.

### 7.2 Update after a semantic delta

When a `GraphDelta` changes branch `a`:

1. Remove the previous document contribution from `M_a`, roots, and sparse surface index entries.
2. Reclassify only changed declarations and relations into typed mutations.
3. Recompute influence only from changed roots and from roots whose resolution dependencies changed. Use a bounded priority traversal ordered by current path influence; stop below `ε`, beyond depth `h`, or after `K` retained targets.
4. Update `Φ_a` by max-aggregation. If a removed path supplied a maximum, recompute only the affected target from its small reverse contributor set.
5. Collect candidates from the union of inverted-index postings for changed mutations, changed surface targets, API entries, components, and active related branches.
6. Recompute the nine signals only for `(a,b)` pairs in that candidate set, then publish a revisioned BCS delta.

The candidate set is not “all branches.” It is bounded by semantic relevance. A policy-controlled low-priority fallback may compare active branches in the same component to avoid blind spots from incomplete resolution.

### 7.3 Keystroke semantics

Every keystroke creates a document revision, but score publication follows the semantic pipeline’s existing interactive rules:

- If the incremental parser emits a stable partial tree, BranchSense creates a **provisional capsule update** immediately.
- If a token-level edit does not alter a declaration, reference, import, or mutation fingerprint, the previous score remains valid and only freshness metadata changes.
- If a parse error obscures a root, BranchSense retracts only facts whose origin is invalid and lowers confidence; it does not discard unrelated file facts.
- Superseded parser/resolver work is cancelled by document version. A score is never published for an older revision after a newer revision is committed.

This makes “after every keystroke” precise: BCS updates after every committed semantic change attributable to a keystroke, not after wasteful recomputation of unchanged graph facts. A cursor move causes no score work.

### 7.4 Complexity

Let `Δ` be changed semantic roots, `E_Δ` the edges visited by their bounded influence traversals, `K` the retained targets per root, and `B_c` the candidate branches returned by indexes.

| Operation | Complexity | Notes |
| --- | --- | --- |
| Mutation classification | `O(|Δ|)` | Adapter output is document-local |
| Surface update | `O(E_Δ log K)` | Bounded by `h`, `ε`, and `K` |
| Index maintenance | `O(|Δ|K)` | Sparse insert/delete postings |
| Candidate retrieval | `O(P + B_c)` | `P` is touched posting-list size |
| Pair scoring | `O(B_c · (K_a + K_b))` worst case | Sparse sorted-vector/intersection operations |
| Score composition | `O(9)` | Constant |

No update contains a term in total repository LOC, total graph edges, or total branch count except in deliberately scheduled background reconciliation. This is the scaling property: steady-state cost follows semantic fanout, not monorepo size.

For hot high-fanout APIs, the surface traversal returns top `K` consumers in the interactive tier and marks fanout truncation in confidence. A background tier expands the rest in chunks. This trades a temporary confidence reduction for latency instead of freezing the editor.

## 8. Worked interpretation

Suppose branch `a` changes a public Java interface method from `parse(String)` to `parse(Path)`. Branch `b` adds a call site to `parse(String)` in a different module. Their text hunks do not overlap.

- `D` is near zero: no shared declaration mutation.
- `S` is positive because the modified interface and the new caller meet at the interface/consumer relation.
- `A` is high because an incompatible public API change targets a consumer changed by the other branch.
- `C` may be moderate if the modules have a declared dependency.
- `T` raises urgency if both workstreams are active.

BCS reports a high score with a concrete explanation: “incompatible API change to `parse(String)` intersects new dependent call on branch `b`.” Git’s textual merge prediction cannot produce this explanation.

## 9. Errors, ambiguity, and adversarial cases

### 9.1 False positives

BCS can over-report when:

- two edits intentionally coordinate through a shared interface and are already compatible;
- static dispatch approximation adds call edges that cannot execute at runtime;
- generated code creates broad but harmless apparent dependencies;
- a branch is observed as active but has already been abandoned;
- ownership differs while the teams are actively coordinating outside the tool;
- historical friction describes a component that has since been redesigned.

Mitigations are explicit compatibility classifications, generated-code discounts, freshness expiry, user-visible acknowledgement with a reason, and expiration/decay of history. A dismissal is recorded as an annotation, not treated as an implicit training label. BCS should prefer a concise, explainable watch signal over interrupting developers for low-confidence proximity.

### 9.2 False negatives

BCS can miss collisions when:

- another developer’s unshared local work is invisible to the system;
- reflection, dynamic loading, code generation, or runtime configuration hides dependencies;
- build resolution is absent or stale;
- a behavioral contract changes without a syntactically visible API change;
- branches interact through databases, protocols, feature flags, or deployment state not modeled in the graph;
- candidate truncation omits a long-tail high-fanout consumer in the interactive tier.

Mitigations include conservative confidence reduction, adapters for build/configuration facts, explicit external-contract nodes, and background fanout expansion. BranchSense must expose “unknown due to dynamic dependency” rather than invent certainty.

### 9.3 Edge cases

| Case | Required behavior |
| --- | --- |
| Rename/move | Use adapter/Git rename hints to preserve identity; otherwise emit conservative remove/add with reduced confidence |
| Rebase or force-push | Recompute merge base, invalidate capsule lineage, retain only content-identical contributions |
| Octopus/stacked branches | Score each direct integration pair and expose a separately composed stack view; do not pretend pairwise scores prove n-way safety |
| Dirty worktree | Treat open-buffer overlay as highest-priority branch revision; never wait for a commit |
| Parse error | Preserve valid facts, mark affected roots provisional, lower confidence |
| Generated/vendor code | Keep provenance, discount risk by default, permit opt-in analysis |
| Deleted symbol | Surface reverse references strongly; unresolved reference is evidence, not an absence of evidence |
| Unrelated repositories | Return no score/zero comparable evidence, not “safe” |
| Mass formatting | Classify formatting-only changes and suppress their mutation severity while preserving textual-anchor visibility |
| Security-sensitive code | Policy may raise significance `η` or routing priority without changing score semantics |

## 10. Why BCS exceeds text-based merge detection

Text-based detection operates over file paths and line ranges. It cannot distinguish a comment change from a public interface deletion, cannot link callers to changed callees, and cannot compare changes across files or modules except by accidental textual overlap.

BCS instead operates on stable semantic identity and impact paths. It predicts four classes of issue that text overlap misses:

1. **Clean-but-breaking merges:** producer API changes versus consumer changes.
2. **Distributed intent collision:** two branches implement competing behavior in different layers connected by a shared contract.
3. **Architectural collision:** independent edits cross a module/build boundary with shared downstream assumptions.
4. **Coordination collision:** related active work by separate ownership groups merits early contact even if a merge will succeed mechanically.

It still includes merge anchors, so it is a strict extension rather than a rejection of Git’s strongest syntactic evidence.

## 11. Scale argument

BCS scales because it does not rebuild a whole-project comparison after every edit. Parsing and graph updates are incremental; capsules retain only changed roots and sparse influence vectors; inverted indexes find candidate branches; scoring is pair-local and bounded.

The algorithm’s primary memory cost is `O(Σ_a (|M_a| + |Φ_a|))` for active capsules plus graph/index memory already required by BranchSense. Retention policies compact inactive capsule detail into commit-level summaries. The primary interactive CPU cost follows bounded graph fanout from the changed semantic roots.

The principal pathological case is a universally used public API. BCS handles it with tiered analysis: direct and highest-impact consumers score immediately, long-tail consumers refine asynchronously, and confidence states the truncation. The algorithm remains responsive rather than falsely claiming complete instantaneous knowledge.

## 12. Evaluation protocol

BCS should be evaluated as a deterministic systems algorithm, not trained. Build a corpus of repository histories with ground truth from merge conflicts, API breakages, reverted integrations, CI failures linked by maintainers, and accepted coordinated changes.

Report:

- lead time before integration for top-ranked collisions;
- precision/recall at policy thresholds, separated by textual and semantic collision class;
- confidence-conditioned reliability (outcomes only for high/medium/low confidence strata);
- score stability across harmless formatting and incremental parse states;
- p50/p95/p99 update time, candidate count, and surface fanout;
- ablations that remove each deterministic signal;
- fairness across code ownership groups and repository sizes.

Evaluation may motivate a reviewed policy change, but the release artifact remains a named deterministic policy. Historical data must never silently tune per-user behavior.

## 13. Future consumers without algorithm change

BCS emits a stable fact bundle:

`CollisionAssessment = { pair, workspaceRevision, policyVersion, score, confidence, signals, contributions, evidence, coverageGaps }`

Future systems can consume this bundle for dashboards, pull-request routing, editor annotations, notifications, or optional external analysis. They can summarize or prioritize BCS evidence, but they must not alter BCS computation or become required for it. This preserves reproducibility: the same graph revisions and policy version always produce the same assessment.

## 14. Research claims and limits

BCS makes a deliberately limited claim: given observable semantic and branch-evolution evidence, counterfactual merge-surface overlap is a better early warning primitive than text overlap alone. It does not claim to decide whether a merge is correct, infer developer intent, or predict runtime behavior perfectly.

The research hypothesis is falsifiable: compared with text-only baselines at equal alert budgets, BCS should provide earlier detection of confirmed integration collisions, especially clean-text semantic failures, while retaining deterministic explanations and interactive latency. If a signal does not improve this outcome in published ablation results, it should be removed or demoted rather than preserved as complexity.

BCS is therefore the core algorithmic contract of BranchSense: **semantic collision pressure, computed incrementally, explained causally, and never hidden behind an opaque model.**

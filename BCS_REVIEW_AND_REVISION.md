# From BCS to BCS-C: Reviewer Rejection and Revision

**Status:** Research revision memo  
**Supersedes:** The scoring model in `BRANCH_COLLISION_SCORE.md` for research claims  
**Proposed name:** **BCS-C — Branch Collision Score with Certificates**

## 1. Program-committee recommendation: reject

**Recommendation: Reject; encourage resubmission after major revision.**

The original BCS document is well-motivated and unusually clear about incremental engineering. It is not, however, publishable as an ICSE/PLDI research contribution. It presents an appealing score but lacks a defensible connection between that score and an integration outcome. Much of the proposed signal set is a collection of plausible heuristics, and the headline equation incorrectly treats highly correlated observations as independent evidence.

Most seriously, it claims to predict *semantic conflicts* using reachability and proximity, while semantic conflict work distinguishes such suspicion from evidence that the composed program changes behavior. Static interference analysis has only moderate measured performance, and behavioral techniques use tests or symbolic execution precisely because graph overlap alone is insufficient. [Static-analysis interference detection](https://arxiv.org/abs/2310.04269) reports F1 around 0.50 on its evaluation; [SAM](https://www.sciencedirect.com/science/article/pii/S0164121224001158) targets semantic conflicts through generated tests; [symbolic-execution work](https://www.researchgate.net/publication/374502976_Symbolic_Execution_to_Detect_Semantic_Merge_Conflicts) makes the same distinction.

The paper should be rejected in its current form for the following reasons.

### 1.1 The outcome is not defined

“Collision” conflates at least four distinct outcomes:

1. a Git textual conflict;
2. a structured-merge conflict;
3. a build/type/test failure after a clean merge;
4. a human coordination cost or duplicate feature.

These outcomes have different observability, prevalence, costs, and oracles. A score can rank all of them for a product, but a research paper cannot claim predictive accuracy without fixing a target. Historical merge-conflict prediction evaluates mechanical conflicts and is often most successful at identifying *safe* merges, not conflicting ones: Owhadi-Kareshk et al. report F1 of 0.57–0.68 for conflicting scenarios despite high safe-merge performance. [Paper](https://arxiv.org/abs/1907.06274).

**Required correction:** define separate claims and oracles. BCS-C predicts **non-commutativity evidence** and **conditional integration risk**, not arbitrary “conflicts.” Textual conflict, build failure, behavioral regression, and coordination concern become separately labelled downstream outcomes.

### 1.2 Noisy-OR is mathematically unjustified

`S`, `A`, `C`, `O`, and `H` are derived from the same impact surface or its intersection. Multiplying their complements double-counts one causal observation. A public API edit, its consumer path, its component edge, and a different owner may turn one fact into five “independent” reasons. The score can rise purely by adding metadata. The stated marginal contributions are also order-dependent counterfactual explanations of a model with no validated causal semantics.

**Required correction:** replace independent-signal accumulation with a deduplicated evidence graph. Each risk item must be attached to a concrete pair of operations and a witness path, and each operation can contribute to at most one primary collision certificate in the aggregate.

### 1.3 The counterfactual merge surface is an uncalibrated proximity heuristic

Max-product traversal provides neither soundness nor a useful completeness claim. Attenuation constants, cutoff `ε`, depth `h`, and top-`K` selection are arbitrary. In particular, max paths discard multiple weaker but jointly relevant paths, while a high-degree public interface produces a broad surface even when changes commute. There is no proof that a graph path preserves behavioral influence.

**Required correction:** use graph traversal only as a candidate generator. Elevate a candidate into high-risk evidence only through an operation-pair commutativity rule or a bounded semantic checker. Report candidate truncation as an upper-risk bound, not as absence of risk.

### 1.4 Semantic identity is assumed rather than solved

The original proposal assumes stable `SymbolId`s, rename hints, and compatible language resolution. The difficult cases are precisely refactorings, overloaded methods, generated sources, partial programs, reflection, classpath variation, and cross-language APIs. Structured/refactoring-aware merge tools demonstrate that matching and move handling remain major sources of errors. [RefMerge](https://arxiv.org/abs/2112.10370) reduced or resolved conflicts in only a subset of studied refactoring scenarios and could itself introduce move-related conflicts. [Spork](https://arxiv.org/abs/2202.05329) likewise documents structured-merge tradeoffs.

**Required correction:** make identity uncertainty first-class. A certificate may depend on an entity correspondence with a measured/declared confidence; uncertain correspondences cannot create a high-severity proof-like certificate.

### 1.5 Scalability claims are unsupported and potentially false

The document claims no total-branch term, but `SurfaceIndex` stores a sparse impact vector for every active branch. In a monorepo with thousands of developer overlays and a high-fanout API, active surface postings can still be `Θ(number of branches × fanout)`. Removal under max aggregation needs provenance for every winner and can cascade after a deletion. The claim that a candidate set is bounded by relevance does not bound the relevance of a ubiquitous node.

Further, full Java resolution requires build and classpath state. Tree parsing can be incremental, but dependency resolution, generated source, annotation processing, and build-file changes are not reliably sub-100 ms. Existing structured merge work explicitly reports runtime as a central limitation; JDime’s own evaluation frames auto-tuning as a precision/performance tradeoff. [JDime](https://www.se.cs.uni-saarland.de/projects/jdime/).

**Required correction:** separate exact local evidence from deferred global analysis; cap and shard active work; use branch summaries and deterministic heavy-hitter handling; do not advertise an end-to-end latency theorem.

### 1.6 The historical and ownership signals threaten validity and fairness

Historical friction has severe selection bias: only visible, merged, and confirmed incidents are counted. A redesigned component inherits its historical stigma. Ownership mismatch measures organization, not technical incompatibility, and may cause alert disparities for cross-team work. These signals can amplify process inequity and are easy to game.

**Required correction:** remove history and ownership from the core collision score. They may rank delivery notifications *after* a technical assessment, behind an explicit policy boundary and fairness audit.

### 1.7 Pairwise branch analysis does not cover real integration topology

Pairwise non-conflict does not imply an n-way merge is valid. Stacked branches, cherry-picks, rebases, and non-linear ancestry invalidate naive “merge base” comparisons. The original document acknowledges octopus merges but supplies no algorithm.

**Required correction:** give BCS-C a pairwise guarantee only. For a merge queue, recompute each candidate against the evolving virtual integration base; do not compose pair scores as proof of queue safety.

### 1.8 Evaluation is not a research design

The prior evaluation proposal mixes confirmed conflicts, test failures, reverts, and maintainer judgements. These labels cannot be pooled. It lacks temporal splits, project holdout, baselines, an annotation protocol, and a false-alert cost model. A paper comparing against Git alone would be unconvincing: structured tools such as JDime, Spork, RefMerge, IntelliMerge, and newer language-agnostic structured merging already set a strong baseline. [Recent comparative study](https://homes.cs.washington.edu/~mernst/pubs/merge-evaluation-ase2024.pdf) warns that evaluation results differ materially by dataset and tool version.

**Required correction:** define task-specific datasets, preregister thresholds, compare both to merge tools and to history-based predictors, and evaluate lead time separately from detection accuracy.

## 2. Redesigned thesis

BCS-C does **not** estimate an unobservable universal probability of conflict. It answers a narrower, falsifiable question:

> Given two branch deltas and an explicit semantic abstraction, what independent evidence shows that their operations cannot commute, and what bounded unresolved integration risk remains?

The algorithm returns a scalar priority score only as a deterministic projection of an evidence interval. Its primary output is an **evidence ledger**: collision certificates, unresolved conditional obligations, and coverage limits. This makes the system useful before merge while avoiding a false claim of behavioral certainty.

## 3. Branch Collision Calculus

### 3.1 Semantic operations

An adapter converts a branch delta relative to its selected merge base into a finite set of normalized semantic operations:

`Ω_a = { o = (target, action, pre, post, footprint, provenance, identity-confidence) }`

| Field | Meaning |
| --- | --- |
| `target` | Stable entity or guarded entity-correspondence set |
| `action` | Add, delete, rename, move, alter-signature, alter-contract, alter-body, alter-reference, alter-build, or alter-config |
| `pre` | Extractable assumptions required by this operation: declaration shape, API signature, module availability, build constraint |
| `post` | Extractable facts established by the operation |
| `footprint` | Read/write sets over semantic facts and explicit external-contract nodes |
| `provenance` | Base/current revision, document range, adapter version, and extraction state |
| `identity-confidence` | Exact, matched, or ambiguous; only exact/matched under a threshold can support strong certificates |

`pre` and `post` are not arbitrary program specifications. They are a small adapter-independent vocabulary of resolvable semantic facts. This deliberately limits claimed completeness but makes rule application auditable.

### 3.2 Commutativity relation

For each candidate pair `(o_a, o_b)`, a language-neutral rule engine plus optional adapter rule evaluates:

`Comm(o_a,o_b) ∈ { PROVEN_COMMUTE, PROVEN_COLLIDE, CONDITIONALLY_COLLIDE, UNKNOWN }`

with a witness `w` and a soundness class.

- `PROVEN_COMMUTE` means their read/write footprints are disjoint **within the modeled abstraction**, or a declared rule proves the operations commute.
- `PROVEN_COLLIDE` means one postcondition contradicts the other’s postcondition, one operation deletes/renames a target required by the other, or both write incompatible values to the same modeled fact.
- `CONDITIONALLY_COLLIDE` means a finite, queryable obligation remains, such as “there exists a changed consumer of removed overload `f(String)`” or “dispatch may target changed override.”
- `UNKNOWN` means evidence is inadequate; it cannot be converted into a collision certificate.

Examples of deterministic `PROVEN_COLLIDE` rules:

| Pair | Witness |
| --- | --- |
| Delete public symbol / add or alter reference to it | Unsatisfied symbol-existence precondition |
| Incompatible signature change / change to a resolved caller using old signature | Failed call-resolution precondition |
| Rename/move / edit resolved against pre-rename identity | Identity-preserving target conflict |
| Two distinct edits set an exclusive build property | Contradictory build-constraint facts |
| Add distinct implementations of an exclusive registration key | Duplicate key invariant |

Rules produce claims only at the fidelity of their facts. An adapter can add precise Java rules—for overload resolution, override contracts, and module exports—without changing the calculus.

### 3.3 Collision certificates and obligations

A **collision certificate** is:

`γ = (o_a, o_b, mode, witness, severity, coverage, provenance)`

where `mode` is textual, symbol, API, build, registration, or adapter-defined; `witness` is the conflicting fact/path; `severity ∈ [0,1]` comes from a public policy table keyed by mode and visibility; and `coverage` records the exact analysis prerequisites.

A **conditional obligation** is:

`ζ = (o_a, o_b, predicate, search-scope, bound, provenance)`

It is a request for a bounded semantic check. Examples include resolving a virtual dispatch target or searching a dependent module for a stale call. Obligations become certificates only when a witness is found. If budget expires, they become explicit unresolved mass, not synthetic positive evidence.

## 4. BCS-C score and bounds

### 4.1 Deduplicated certificate matching

Construct a bipartite graph `Q_ab = (Ω_a, Ω_b, Γ_ab)`, with an edge for each certificate. Compute a maximum-weight matching `Match(Γ_ab)` using certificate severity as edge weight, with a deterministic tie break on operation identity.

This prevents one API edit from being counted once per caller, per module, per owner, and per surface path. The score measures independent conflicting operation pairs, while the evidence ledger retains all affected callers for explanation.

Let:

`W = Σ_{γ∈Match(Γ_ab)} severity(γ) · coverage(γ)`

and define the certified lower bound:

`L = 1 - exp(-W)`

`L` is not a failure probability. It is a saturating, monotone measure of independent observed non-commutativity. One severe public API certificate can be sufficient for a high `L`; unrelated certificates accumulate without unbounded growth.

### 4.2 Conditional upper bound

Let `Z` be deduplicated conditional obligations, also matched by their source operations. Each obligation receives a policy maximum `u(ζ)` based on its mode and semantic scope. Let `M_Z` be its maximum-weight matching after excluding operations already used by a certificate. Let `U_gap` summarize unobserved but relevant state (unresolved classpath, incomplete branch visibility, truncated fanout) under fixed caps.

`U = min(1, L + (1-L) · [1 - exp(-Σ_{ζ∈M_Z}u(ζ) - U_gap)])`

Thus `L ≤ U`. The interval `[L,U]` is central: a score with a wide interval tells the developer what BranchSense cannot presently establish.

### 4.3 Scalar BCS-C

For alert ordering only:

`BCS-C(a,b) = 100 · [ L + λ · (U-L) ]`

where `λ` is a named, fixed risk posture (`0` evidence-only, `0.5` balanced default, `1` conservative). The score is always emitted together with `[L,U]`, posture, certificate count, obligation count, and coverage. A consumer must not use the scalar without the interval.

This is composable without a false independence assumption: certificates and obligations are deduplicated through matching, and the unresolved contribution has an explicit upper-bound meaning. Social/ownership/history data are not terms in BCS-C.

### 4.4 Confidence and coverage

Replace the prior weighted-average confidence with a vector:

`Coverage = (parse, identity, resolution, branch-visibility, fanout, external-contract)`

Each component is a measured fraction of required facts/targets analyzed, not a subjective belief. A displayed scalar confidence is the minimum required component for the certificate/obligation being shown. This prevents excellent parsing coverage from hiding an absent classpath.

## 5. Incremental computation and scale controls

### 5.1 Candidate generation is not evidence

Use direct footprint intersections, reverse precondition indexes, and bounded graph reachability only to find operation pairs. The candidate set may include false positives freely; it affects work, not score correctness. A pair without a commutativity rule or fulfilled conditional witness produces `UNKNOWN`, not a positive signal.

### 5.2 Incremental state

Maintain, per active branch:

- normalized operations keyed by origin revision;
- fact read/write postings and precondition reverse indexes;
- a compact summary per component: Bloom filter for negative candidate pruning plus exact heavy-hitter operation IDs;
- exact certificate/obligation edges only for active candidate pairs.

Bloom filters are never used to establish an intersection, only to skip impossible work; false positives affect latency only. High-fanout symbols use a deterministic two-tier policy: exact direct consumers and changed consumers are interactive; remaining consumers are partitioned and processed in background. Unprocessed partitions increase `U_gap` and lower fanout coverage.

### 5.3 Update algorithm

After a committed document semantic delta on branch `a`:

1. Retract only operations whose provenance belongs to the replaced document revision.
2. Extract replacement operations and update their fact postings.
3. Obtain candidate operations from exact direct postings and component summaries. Do not enumerate all branch surfaces.
4. Re-evaluate commutativity only for pairs touching retracted/replaced operations.
5. Repair the affected dynamic matching component. Use a local augmenting-path update; fall back to recomputing that connected component when its size crosses a bounded threshold.
6. Recompute `L`, `U`, and BCS-C for only impacted branch pairs; publish the certificate delta and coverage changes.

The interactive path contains no transitive whole-repository traversal. Transitive resolution runs as named obligations under a deadline.

### 5.4 Complexity

For `d` changed operations, `p` exact posting entries reached, and `q` certificate/obligation edges in the impacted bipartite component, update cost is:

`O(d + p + q√|Ω|)` worst-case for matching repair,

with a strict configured component-size cap. In the common case of a small component, local augmenting paths dominate. When a cap is reached, BCS-C publishes the exact lower bound from completed certificates plus a widened upper bound and schedules full matching in the background.

Memory is capped by retaining exact operations only for active workstreams and recent commits. Inactive branches retain component summaries and selected public-operation facts. Reopening or comparing an old branch lazily reconstructs its exact capsule from the Git/object and persistent semantic cache. This trades cold-start latency for a real bound on daemon memory.

No claim is made that full build resolution or a global high-fanout scan completes in 100 ms. The claim is narrower and testable: for a warm workspace, local extraction, direct certificate checks, and score publication obey a configured interactive budget; deferred obligations are visible in the interval.

## 6. Multi-branch and Git-DAG semantics

BCS-C is defined for a **directed integration attempt** `(target, incoming, base)`, not an abstract unordered branch pair. The target’s current virtual integration base is part of every operation provenance. Symmetric display is produced by evaluating both directions when necessary.

For a merge queue, BranchSense builds an ephemeral virtual base after each accepted candidate and recomputes the next candidate against that base. It never infers n-way safety from pairwise BCS-C values. Rebase, cherry-pick, and force-push invalidate operations whose base provenance no longer matches; content-identical operations may be reused only after fact revalidation.

## 7. What remains impossible

BCS-C cannot soundly detect arbitrary behavioral semantic conflicts without a behavioral specification or sufficient dynamic/static analysis. Reflection, native calls, data migrations, feature flags, environment configuration, and unshared local edits remain incomplete. The algorithm represents these as external-contract nodes, unresolved obligations, or absent branch-visibility coverage.

This is not a defect to hide. It defines a clean research boundary: BCS-C provides soundness *relative to its operation abstraction* for `PROVEN_COLLIDE` and `PROVEN_COMMUTE` rules, while all other cases are conditional or unknown. The soundness proof obligation for a rule belongs to the language adapter and its fact semantics.

## 8. Publishable evaluation plan

### Research questions

- **RQ1 (validity):** Do BCS-C certificates predict independently adjudicated mechanical/build/API integration failures with high precision at fixed alert budgets?
- **RQ2 (lead time):** How much earlier are valid alerts available than merge-queue or pull-request integration checks?
- **RQ3 (cost):** Does bounded incremental checking meet interactive budgets, and how often is the interval too wide to be useful?
- **RQ4 (ablation):** What precision/lead-time is lost when operation rules, conditional obligations, or identity confidence are removed?

### Corpora and labels

Create disjoint, time-ordered train-free development and evaluation corpora. Labels are never pooled:

1. Git/structured-merge conflict from deterministic replay.
2. Compilation/build failure after a clean replayed merge.
3. API incompatibility with a documented dependent call witness.
4. Behavioral regression, only where a test or formal oracle establishes it.

Report each separately, then report a product utility curve with predeclared costs. Manually adjudicate a stratified sample of false positives and false negatives with blinded dual review. Hold out entire repositories, ecosystem families, and Java versions. Include incomplete-build and generated-code strata.

### Baselines

Compare against Git three-way merge, a line/hunk overlap warning, AST/structured merge, refactoring-aware merge, and a lightweight Git-history conflict predictor. Relevant baselines include [JDime](https://www.se.cs.uni-saarland.de/projects/jdime/), [Spork](https://arxiv.org/abs/2202.05329), [RefMerge](https://arxiv.org/abs/2112.10370), and the [ESEM conflict predictor](https://arxiv.org/abs/1907.06274). Where tools cannot run on a scenario, report coverage rather than silently excluding it.

Do not compare against neural systems as a central baseline: BCS-C makes a deterministic, explainable-systems claim. Test-generation or symbolic-execution semantic analyses belong as expensive confirmation-tier comparators, not substitutes for early incremental warning.

### Acceptance criteria

The paper should claim no general victory unless it demonstrates all of the following on held-out repositories:

- certificate precision substantially above a hunk-overlap baseline at the same alert budget;
- nontrivial lead time for at least one non-textual labelled outcome;
- measured interactive latency and memory under a stated active-branch/fanout workload;
- interval calibration: higher unresolved width must correlate with reduced outcome certainty;
- no material alert-rate disparity attributable solely to ownership metadata, which is excluded from the core score.

## 9. Second reviewer pass

The revision resolves the original fatal flaws:

| Original rejection | BCS-C response | Remaining limitation |
| --- | --- | --- |
| Undefined outcome | Certificate modes and separate label oracles | Behavioral outcomes still need external oracles |
| Correlated noisy-OR | Operation-pair evidence graph and matching | Severity policy remains a normative ranking choice |
| Proximity presented as proof | Reachability only generates candidates | Candidate recall depends on adapters/indexes |
| Arbitrary low confidence | Explicit lower/upper bounds and coverage vector | Bounds are abstraction-relative, not universal probability bounds |
| Unbounded per-branch surfaces | Active exact state, summaries, tiered fanout, cold reconstruction | Adversarial hot components defer work |
| Social bias in core score | Ownership/history removed from BCS-C | Separate routing policy still needs governance |
| Pairwise overclaim | Directed virtual-base semantics | N-way integration remains sequentially evaluated |

No honest reviewer can say “no major weaknesses remain” before a prototype and empirical study. The revised paper is publishable in *form* because its claims are precise, falsifiable, and aligned with its evidence. Its remaining risks are measurable research questions rather than hidden assumptions. The next artifact must be a rule catalogue, a reproducible replay harness, and a preregistered evaluation—not another scoring heuristic.

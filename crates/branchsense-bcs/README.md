# branchsense-bcs

The deterministic Branch Collision Score (BCS) assessment engine.

This crate consumes normalized, explainable evidence from the `branchsense-semantic`, `branchsense-impact`, `branchsense-history`, and `branchsense-ownership` subsystems to produce a final, ordinal, risk-assessed semantic collision band.

## Core Principles
1. **Deterministic**: Given identical evidence and configuration, the engine will always produce the exact same ordinal band.
2. **Explainable**: All decisions are tied directly to an immutable ledger of evidence. No probability vectors or opaque confidence scalars are used.
3. **Traceable**: The provenance of the score can be mathematically proven by walking the causal lineage back to the originating Git revision.
4. **Abstaining**: If the underlying semantic data is unavailable or incomplete, the BCS engine abstains (returns `Indeterminate`). It does not silently drop evidence or guess.

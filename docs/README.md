# Biological Computing Engine — Design

The source-of-truth design document (`biological_computing_engine_design.md`)
should be copied here. The scaffolding plan references it as the design this
library implements.

Until that file is present, see `biomimicry_scaffolding_plan.md` at the repo root
for the module map and milestone sequence.

## M1 hygiene note (pending design-doc drop-in)

When `biological_computing_engine_design.md` lands, correct the stale rev-6
wording in Part II.2's DNA row and the matching Resolved-fork bullet:

- **Was:** `8 primitive node types` / “eight primitives in four polar pairs”
- **Is:** `4 primitive node types + polarity` (rev 7, matching Part II.0 and the
  infinite-db role table)

M1 implements the 4-primitive model; this is documentation hygiene only.

## M2 hygiene note

When the design doc lands, confirm Part II.5 uses **"Coupled cluster"** as the
alias for engine `Scope::Cluster`, and that Layer-1 lifecycle `Differentiating`
is documented as distinct from Layer-2 behavioral mode `Differentiating`.

## M3 hygiene note

Under determinism, the outer loop is **queue-driven only** (no wall-clock on the
replay path). Scheduler "drained" ≠ attractor convergence (M5).

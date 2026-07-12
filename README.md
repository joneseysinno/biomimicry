# biomimicry

Biological computing engine: **computation as settling** in a living hypergraph.

There is no `main()` and no orchestrator. You build an `Organism`, `perturb` it, and let it `settle`.

> Status: **M0 scaffold** — full module tree as documented stubs. Subsystems land per the milestone plan in [`biomimicry_scaffolding_plan.md`](./biomimicry_scaffolding_plan.md).

## Workspace

| Crate | Role |
|-------|------|
| `biomimicry` | Core engine library |
| `biomimicry-substrate` | Optional `infinite-db` `Store` (Milestone 7) |
| `biomimicry-aec` | AEC reference app (Milestone 9) |
| `biomimicry-inspector` | Causal DAG / landscape inspector (Milestone 5) |

## Module convention

- Every `<module>.rs` holds **only** module docs, `mod` declarations, and `pub use` re-exports.
- Every type and function lives in a **leaf file** named for its single concern (e.g. `genesis/compile.rs`).
- **No `mod.rs` files.** A module `foo` with children is `foo.rs` + a `foo/` directory beside it.

## Feature flags

```toml
[features]
default = ["memory-store", "determinism"]
memory-store = []   # in-memory Store (fast tests)
infinite-db = []    # marker — use the biomimicry-substrate crate for the real backend
determinism = []    # seeded, replayable scheduling
inspector = []      # emit DAG/landscape traces for the inspector tool
```

The `Store` trait lives in `biomimicry::substrate`. The concrete `infinite-db` implementation is a **separate crate** (`biomimicry-substrate`) so the core never hard-depends on persistence and never forms a circular crate dependency.

## Quick start

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

```rust
use biomimicry::prelude::*;

// M0: Store + types exist; organism assembly lands across M1–M5.
let store = MemoryStore::new();
let _ = store;
```

## Milestones

See [`biomimicry_scaffolding_plan.md`](./biomimicry_scaffolding_plan.md). **M5 (first settle)** is the keystone.

## License

MIT

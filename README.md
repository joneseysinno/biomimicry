# biomimicry

Biological computing engine: **computation as settling** in a living gene regulatory network (GRN).

There is no `main()` and no orchestrator. You build an `Organism`, `perturb` (or `ingress`) it, and let it `settle`.

Engine DNA types speak biology (`Cistron`, `Grn`); infinite-db's `Hyperedge` / `Space` names are translated only in `biomimicry-substrate`.

> Status: **M10 / gardenable** — API **0.2.0** (`Cistron`/`Grn` rename). Iterate on rulesets by observing where they settle. Research seams (Hilbert, infinite-db Spaces, provenance learning) remain post-0.1.

## Workspace

| Crate | Role |
|-------|------|
| `biomimicry` | Core engine library |
| `biomimicry-substrate` | Optional durable `Store` (MemoryStore blob; Space schema deferred) |
| `biomimicry-aec` | AEC wall-move reference app (Part VIII) |
| `biomimicry-inspector` | Causal DAG / landscape / scenario traces |

## Quick start

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
cargo bench -p biomimicry          # Criterion (manual; not in CI by default)
cargo run -p biomimicry-inspector -- --scenario cascade
```

```rust
use biomimicry::prelude::*;
use biomimicry::organism::{settle_ready, trigger_signal};

fn main() {
    let mut org = settle_ready(42);
    org.perturb(trigger_signal()).expect("perturb");
    let status = org.settle(32).expect("settle");
    println!("{status:?}");
}
```

## Example gallery

| Example | Package | Story |
|---------|---------|-------|
| `minimal_organism` | biomimicry | Scaffold construct |
| `first_settle` | biomimicry | Cascade perturb → settle |
| `echo_ingress` | biomimicry | M8 matched-signaling echo |
| `replay_checkpoint` | biomimicry | M7 checkpoint / restore |
| `wall_move` | biomimicry-aec | M9 Part VIII reflex |

```bash
cargo run -p biomimicry --example first_settle
cargo run -p biomimicry --example echo_ingress
cargo run -p biomimicry --example replay_checkpoint
cargo run -p biomimicry-aec --example wall_move
```

## Module convention

- Every `<module>.rs` holds **only** module docs, `mod` declarations, and `pub use` re-exports.
- Every type and function lives in a **leaf file** named for its single concern.
- **No `mod.rs` files.**

## Feature flags

```toml
[features]
default = ["memory-store", "determinism"]
memory-store = []
infinite-db = []    # marker — real backend in biomimicry-substrate
determinism = []
inspector = []
```

## License

MIT

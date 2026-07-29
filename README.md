# biomimicry

Biological computing engine: **computation as settling** in a living gene regulatory network (GRN).

There is no `main()` and no orchestrator. You build an `Organism`, `perturb` (or `ingress`) it, and let it `settle`.

Engine DNA types speak biology (`Cistron`, `Grn`); infinite-db's `Hyperedge` / `Space` names are translated only in `biomimicry-substrate`.

> Status: **M12 / gardenable** — API **0.4.0** (block linker: compose organisms from DNA fragments via a TOML manifest). The manifest *is* the application. `0.3.x` genotypes remain content-addressed but compositions are new.

## Workspace

| Crate | Role |
|-------|------|
| `biomimicry` | Core engine library |
| `biomimicry-substrate` | Optional durable `Store` (MemoryStore blob; Space schema deferred) |
| `biomimicry-aec` | AEC wall-move reference app (Part VIII) |
| `biomimicry-genomes` | Default genomes library (`engineer_calculator`, …) |
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

## Manifest composition (M12)

An application is a **manifest** naming blocks — not orchestration code. Link fragments into one GRN, compile once:

```toml
[[blocks]]
name = "sum"
version = "1.0.0"

[[blocks]]
name = "scale"
version = "1.0.0"

[[blocks]]
name = "sink"
version = "1.0.0"
```

```rust
use biomimicry::prelude::*;
use biomimicry::blocks::{
    pipeline_blocks, pipeline_manifest, link_and_compile, linked_organism,
};
use biomimicry::ganglion::stimulate;
use biomimicry::signal::Value;

fn main() {
    let (linked, genome) = link_and_compile(&pipeline_blocks(), &pipeline_manifest())
        .expect("link");
    let (mut org, handles) = linked_organism(&linked, genome, 42);
    let sum = *handles.get(&BlockName::new("sum")).unwrap();
    let scale = *handles.get(&BlockName::new("scale")).unwrap();
    stimulate(&mut org, sum, Value::record_from([
        ("a", Value::Int(3000)),
        ("b", Value::Int(4000)),
    ]).unwrap(), 64).unwrap();
    stimulate(&mut org, scale, Value::record_from([
        ("factor", Value::Int(2000)),
    ]).unwrap(), 64).unwrap();
    // sink.result effector holds 14000 (millis)
}
```

Authors write local kind names (`total`); the linker qualifies them (`sum::total`) and synthesises bridge cistrons for wires. Unsatisfied imports fail at **link time** with a precise error — before `compile` is ever attempted.

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

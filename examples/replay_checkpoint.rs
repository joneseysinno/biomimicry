//! Replay checkpoint demo — flush, checkpoint, restore (M7).

use biomimicry::organism::{settle_ready, trigger_signal};
use biomimicry::substrate::Store;

fn main() {
    let mut org = settle_ready(42);
    org.perturb(trigger_signal()).expect("perturb");
    let status = org.settle(32).expect("settle");
    println!("settled: {status:?}");

    org.open_commit_gate();
    let meta = org.checkpoint("demo", false).expect("checkpoint");
    let before = org.load_causal_dag().expect("dag");
    println!("checkpoint id={} nodes={}", meta.id.0, before.len());

    org.store
        .replace_causal_dag(biomimicry::causality::CausalDag::new())
        .expect("clear");
    org.restore_checkpoint(meta.id).expect("restore");
    let after = org.load_causal_dag().expect("dag");
    assert_eq!(before, after);
    println!("restore ok — dag nodes={}", after.len());
}

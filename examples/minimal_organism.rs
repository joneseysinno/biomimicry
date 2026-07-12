//! Minimal organism perturbation demo.

use biomimicry::expression::m4_handles;
use biomimicry::metabolism::Cadence;
use biomimicry::organism::{OrganismBuilder, trigger_signal};
use std::sync::Arc;

fn main() {
    let handles = m4_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(1)
        .cadence(Cadence::new(2))
        .population_size(2)
        .seed_gene(handles.cascade_path)
        .without_pop_loop()
        .build()
        .expect("build");
    org.perturb(trigger_signal()).expect("perturb");
    let status = org.settle(16).expect("settle");
    println!("minimal_organism → {status:?}");
}

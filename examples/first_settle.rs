//! First-settle demo — perturb a small organism and print settle status.

use biomimicry::attractor::settle_trace;
use biomimicry::expression::{NetworkRegulator, m4_handles, m4_network, m4_transducer};
use biomimicry::metabolism::Cadence;
use biomimicry::organism::{OrganismBuilder, trigger_signal};
use std::sync::Arc;

fn main() {
    let handles = m4_handles();
    let mut org = OrganismBuilder::new()
        .genome(Arc::clone(&handles.genome))
        .seed(42)
        .cadence(Cadence::new(2))
        .population_size(2)
        .target_population(2)
        .seed_gene(handles.cascade_path)
        .regulator(NetworkRegulator::new(m4_network(handles.downstream)))
        .transducer(m4_transducer(handles.cascade_path))
        .build()
        .expect("build");

    org.perturb(trigger_signal()).expect("perturb");
    let status = org.settle(32).expect("settle");
    println!("SettleStatus::{status:?}");
    print!(
        "{}",
        settle_trace(&org.scheduler.log, org.trajectory(), status)
    );
}

//! Settle / echo ingress throughput benches (M10).
#![allow(missing_docs)]

use biomimicry::membrane::{echo_ready, echo_request};
use biomimicry::organism::{settle_ready, trigger_signal};
use criterion::{Criterion, criterion_group, criterion_main};

fn settle_cascade(c: &mut Criterion) {
    c.bench_function("settle_cascade", |b| {
        b.iter(|| {
            let mut org = settle_ready(42);
            org.perturb(trigger_signal()).expect("perturb");
            let _ = org.settle(32).expect("settle");
        });
    });
}

fn ingress_echo_settle(c: &mut Criterion) {
    c.bench_function("ingress_echo_settle", |b| {
        b.iter(|| {
            let mut org = echo_ready(7);
            let _ = org.ingress(echo_request(100)).expect("ingress");
            let _ = org.settle(32).expect("settle");
        });
    });
}

criterion_group!(benches, settle_cascade, ingress_echo_settle);
criterion_main!(benches);

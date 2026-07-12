//! Echo ingress demo — matched signaling boundary (M8).

use biomimicry::attractor::settle_trace;
use biomimicry::causality::causal_order_dot;
use biomimicry::membrane::{ResponseMode, echo_ready, echo_request};

fn main() {
    let mut org = echo_ready(42);
    let mode = org.ingress(echo_request(100)).expect("ingress");
    assert_eq!(mode, ResponseMode::Reflex);
    let status = org.settle(32).expect("settle");
    println!("echo ingress → SettleStatus::{status:?}");
    print!(
        "{}",
        settle_trace(&org.scheduler.log, org.trajectory(), status)
    );
    print!("{}", causal_order_dot(&org.scheduler.log));
}

//! Wall-move demo — Part VIII reflex via biomimicry-aec (M9).

use biomimicry_aec::{cascade_evidence, run_reflex};

fn main() {
    let (_org, report) = run_reflex(42, 32).expect("wall-move reflex");
    println!("scenario={}", report.scenario);
    println!("settle={:?}", report.settle);
    println!("cascade_fired={}", report.cascade_fired);
    assert!(cascade_evidence(&report.reflex_tags));
    println!("tags={:?}", report.reflex_tags);
}

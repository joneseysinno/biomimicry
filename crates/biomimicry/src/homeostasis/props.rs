//! Property / unit tests for homeostasis (M5).

use crate::homeostasis::{HomeostaticLoop, PopulationSizeLoop};

#[test]
fn p2_cull_pending_when_over_target() {
    let mut loop_ = PopulationSizeLoop::new(2);
    loop_.current = 4;
    let err = loop_.step(0).unwrap();
    assert!(err < 0);
    assert_eq!(loop_.take_cull(), 1);
    assert_eq!(loop_.take_recruit(), 0);
}

#[test]
fn p3_recruit_pending_when_under_target() {
    let mut loop_ = PopulationSizeLoop::new(4);
    loop_.current = 2;
    let _ = loop_.step(0).unwrap();
    assert_eq!(loop_.take_recruit(), 1);
}

#[test]
fn undamped_kick_at_setpoint() {
    let mut loop_ = PopulationSizeLoop::undamped(2);
    loop_.current = 2;
    let _ = loop_.step(0).unwrap();
    assert_eq!(loop_.take_recruit(), 2);
}

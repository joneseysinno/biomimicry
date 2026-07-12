//! Property / unit tests for attractor detectors (M5).

use crate::attractor::{
    Basin, Landscape, SettleStatus, detect_convergence, detect_divergence, fingerprint_cells,
};
use crate::cell::{Cell, CellId, LifecycleState};
use crate::genesis::{compile, toy_dna};
use std::sync::Arc;

#[test]
fn p1_convergence_window_determinism() {
    let t = vec![1u128, 9, 9, 9];
    assert_eq!(detect_convergence(&t, 3), SettleStatus::Converged);
    assert_eq!(detect_convergence(&t, 3), detect_convergence(&t, 3));
}

#[test]
fn p6_basin_contains_center() {
    let b = Basin::new(42, 0);
    assert!(b.contains(42));
    let mut land = Landscape::new();
    land.insert(42, 0);
    assert_eq!(land.potential(42), 0);
}

#[test]
fn fingerprint_stable_for_same_expression() {
    let genome = compile(&toy_dna()).unwrap();
    let mut a = Cell::new(CellId(1), Arc::clone(&genome));
    let mut b = Cell::new(CellId(1), Arc::clone(&genome));
    a.try_transition(LifecycleState::Differentiating).unwrap();
    a.try_transition(LifecycleState::Active).unwrap();
    b.try_transition(LifecycleState::Differentiating).unwrap();
    b.try_transition(LifecycleState::Active).unwrap();
    let spike = genome
        .iter()
        .find(|g| g.cistron.kind.as_str() == "sensory_spike")
        .map(|g| g.id)
        .unwrap();
    a.activate(spike);
    b.activate(spike);
    assert_eq!(fingerprint_cells(&[a]), fingerprint_cells(&[b]));
}

#[test]
fn p5_divergence_none_on_fixed_point() {
    assert_eq!(detect_divergence(&[3, 3, 3, 3]), None);
}

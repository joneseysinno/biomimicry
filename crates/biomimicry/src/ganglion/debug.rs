//! Inspector dump for a ganglion.

use crate::cell::Cell;
use crate::ganglion::{Ganglion, GanglionView};

/// Human-readable ganglion profile.
#[must_use]
pub fn inspect(ganglion: &Ganglion, population: &[Cell]) -> String {
    let living = ganglion
        .members
        .iter()
        .filter(|id| {
            population
                .iter()
                .find(|c| c.id == **id)
                .is_some_and(|c| c.lifecycle() != crate::cell::LifecycleState::Dead)
        })
        .count();
    format!(
        "ganglion {name:?} handle={h:?} health={health:?} members={m} living={living} capacity={cap} k={k}",
        name = ganglion.name,
        h = ganglion.handle,
        health = ganglion.health,
        m = ganglion.members.len(),
        living = living,
        cap = ganglion.capacity,
        k = ganglion.space.k,
    )
}

/// Build a [`GanglionView`] snapshot.
#[must_use]
pub fn view(ganglion: &Ganglion, population: &[Cell]) -> GanglionView {
    let living = ganglion
        .members
        .iter()
        .filter(|id| {
            population
                .iter()
                .find(|c| c.id == **id)
                .is_some_and(|c| c.lifecycle() != crate::cell::LifecycleState::Dead)
        })
        .count();
    GanglionView {
        handle: ganglion.handle,
        name: ganglion.name.clone(),
        health: ganglion.health,
        members: ganglion.members.clone(),
        living,
        capacity: ganglion.capacity,
    }
}

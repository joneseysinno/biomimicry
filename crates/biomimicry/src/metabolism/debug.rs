//! Inspector helpers for the scheduler.

use super::Scheduler;

/// Human-readable scheduler trace: cycles, queue depths, drained flag.
#[must_use]
pub fn trace(scheduler: &Scheduler) -> String {
    format!(
        "scheduler\n  seed: {seed}\n  outer: {outer}\n  inner: {inner}\n  \
         k: {k}\n  phase1_depth: {p1}\n  phase2_depth: {p2}\n  drained: {drained}\n  \
         log_events: {log}\n",
        seed = scheduler.seed,
        outer = scheduler.outer_cycles,
        inner = scheduler.inner_cycles,
        k = scheduler.cadence.k,
        p1 = scheduler.phase1.len(),
        p2 = scheduler.phase2.len(),
        drained = scheduler.is_drained(),
        log = scheduler.log.len(),
    )
}

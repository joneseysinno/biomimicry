//! `CascadeTransducer` — real Phase 2 brain behind the M3 `Transducer` seam.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::cell::{Cell, CellId, Operation};
use crate::effector::EffectorId;
use crate::error::{BiomimicryError, Result};
use crate::genesis::{GeneId, Genome};
use crate::metabolism::Transducer;
use crate::signal::{CausalStamp, Signal, SignalId, Value};
use crate::transduction::arith::is_unary;
use crate::transduction::fold::{FoldState, fold_signals};
use crate::transduction::{
    ArithOp, Cascade, FoldSpec, TransductionFn, TransductionKind, cascade_from_spec,
    emit_from_cascade,
};

/// An effector write queued during transduction (drained by the organism).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingEffect {
    /// Target effector.
    pub id: EffectorId,
    /// Value to write.
    pub value: Value,
    /// Causal stamp of the write.
    pub stamp: CausalStamp,
    /// Parent signal that caused the write.
    pub parent: SignalId,
    /// Cell that performed the write.
    pub cell: CellId,
}

#[derive(Debug, Default)]
struct TransducerScratch {
    pending_effects: Vec<PendingEffect>,
    fold_states: BTreeMap<(CellId, GeneId), FoldState>,
    /// First operand waiting for a binary arith step (second delivery completes).
    pending_binary: BTreeMap<(CellId, GeneId), Signal>,
    /// Last value emitted per output kind (for ganglion port readout).
    last_outputs: BTreeMap<String, Value>,
}

/// Phase 2 transducer driven by per-gene cascades.
#[derive(Debug, Clone, Default)]
pub struct CascadeTransducer {
    /// Cascades keyed by gene id.
    pub cascades: BTreeMap<GeneId, Cascade>,
    /// Shared pending effects / fold / binary buffers (interior mutability through `&self`).
    scratch: Arc<Mutex<TransducerScratch>>,
}

impl CascadeTransducer {
    /// Create an empty transducer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Primary constructor: prefer compiled [`Genome::cascades`], else walk cistron specs.
    ///
    /// Genes without a [`crate::transduction::TransductionSpec`] are skipped
    /// (no cascade registered). Host [`Self::with_cascade`] remains a
    /// test-and-override affordance only.
    #[must_use]
    pub fn from_genome(genome: &Genome) -> Self {
        if !genome.cascades().is_empty() {
            return Self {
                cascades: genome.cascades().clone(),
                scratch: Arc::new(Mutex::new(TransducerScratch::default())),
            };
        }
        let mut cascades = BTreeMap::new();
        for gene in genome.iter() {
            if let Some(spec) = &gene.cistron.transduction {
                cascades.insert(gene.id, cascade_from_spec(spec));
            }
        }
        Self {
            cascades,
            scratch: Arc::new(Mutex::new(TransducerScratch::default())),
        }
    }

    /// Builder: register a cascade for a gene.
    ///
    /// Prefer [`Self::from_genome`] in production; this is a test/override hook.
    #[must_use]
    pub fn with_cascade(mut self, gene: GeneId, cascade: Cascade) -> Self {
        self.cascades.insert(gene, cascade);
        self
    }

    /// Drain effector writes queued since the last drain.
    pub fn drain_pending_effects(&self) -> Vec<PendingEffect> {
        self.scratch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pending_effects
            .drain(..)
            .collect()
    }

    /// Snapshot of the latest value emitted per signal kind.
    #[must_use]
    pub fn last_outputs(&self) -> BTreeMap<String, Value> {
        self.scratch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last_outputs
            .clone()
    }

    /// Run transduction for `gene`, or return a typed error if missing.
    ///
    /// # Errors
    ///
    /// Returns [`BiomimicryError::CascadeUnavailable`] when no cascade is
    /// registered for an active gene. Propagates cascade step failures.
    pub fn transduce_checked(
        &self,
        cell: &Cell,
        sig: &Signal,
        gene: GeneId,
    ) -> Result<Vec<Operation>> {
        if !cell.expression.is_active(gene) {
            return Ok(Vec::new());
        }
        let Some(cascade) = self.cascades.get(&gene) else {
            return Err(BiomimicryError::CascadeUnavailable { gene });
        };
        let outputs = self.run_pipeline(cascade, cell.id, gene, sig)?;
        {
            let mut scratch = self
                .scratch
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for out in &outputs {
                if let Ok(value) = out.payload.value() {
                    scratch
                        .last_outputs
                        .insert(out.kind.as_str().to_owned(), value);
                }
            }
        }
        let stamp = cell.peek_stamp();
        let signals = emit_from_cascade(outputs, cell.id, stamp)?;
        Ok(signals.into_iter().map(Operation::Emit).collect())
    }

    fn run_pipeline(
        &self,
        cascade: &Cascade,
        cell: CellId,
        gene: GeneId,
        input: &Signal,
    ) -> Result<Vec<Signal>> {
        if cascade.steps.is_empty() {
            return Ok(Vec::new());
        }
        let mut current = vec![input.clone()];
        for step in &cascade.steps {
            current = self.run_step(step, &current, cell, gene, input)?;
            if current.is_empty() {
                break;
            }
        }
        Ok(current)
    }

    fn run_step(
        &self,
        step: &TransductionFn,
        inputs: &[Signal],
        cell: CellId,
        gene: GeneId,
        trigger: &Signal,
    ) -> Result<Vec<Signal>> {
        if !step.enabled {
            return Ok(Vec::new());
        }
        match &step.kind {
            TransductionKind::Effect(id) => {
                self.queue_effect(*id, inputs, cell, trigger)?;
                Ok(Vec::new())
            }
            TransductionKind::Fold(spec) => self.run_fold(step, spec, inputs, cell, gene),
            TransductionKind::Arith(op) if !is_unary(*op) => {
                self.run_arith_binary(step, *op, inputs, cell, gene)
            }
            _ => step.call_many(inputs),
        }
    }

    fn queue_effect(
        &self,
        id: EffectorId,
        inputs: &[Signal],
        cell: CellId,
        trigger: &Signal,
    ) -> Result<()> {
        let src = inputs.last().unwrap_or(trigger);
        let value = src.payload.value()?;
        let mut scratch = self
            .scratch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        scratch.pending_effects.push(PendingEffect {
            id,
            value,
            stamp: src.stamp,
            parent: src.id,
            cell,
        });
        Ok(())
    }

    fn run_fold(
        &self,
        step: &TransductionFn,
        spec: &FoldSpec,
        inputs: &[Signal],
        cell: CellId,
        gene: GeneId,
    ) -> Result<Vec<Signal>> {
        // Multi-input in one call: fold immediately (unit / fan-in path).
        if inputs.len() >= 2 {
            return match fold_signals(spec, inputs, &step.name)? {
                Some(value) => {
                    let src = inputs
                        .iter()
                        .max_by(|a, b| a.stamp.cmp(&b.stamp).then(a.id.cmp(&b.id)))
                        .expect("len >= 2");
                    Ok(vec![emit_value(step, src, value)])
                }
                None => Ok(Vec::new()),
            };
        }
        let Some(input) = inputs.first() else {
            return Ok(Vec::new());
        };
        let value = input.payload.value()?;
        let mut scratch = self
            .scratch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = (cell, gene);
        let state = scratch
            .fold_states
            .entry(key)
            .or_insert_with(|| FoldState::new(spec.init.clone()));
        state.apply(spec.op, value, input.stamp, &step.name)?;
        if state.barrier_met(spec.barrier) {
            let acc = state.acc.clone();
            scratch.fold_states.remove(&key);
            drop(scratch);
            Ok(vec![emit_value(step, input, acc)])
        } else {
            Ok(Vec::new())
        }
    }

    fn run_arith_binary(
        &self,
        step: &TransductionFn,
        op: ArithOp,
        inputs: &[Signal],
        cell: CellId,
        gene: GeneId,
    ) -> Result<Vec<Signal>> {
        if inputs.len() >= 2 {
            return step.call_many(inputs);
        }
        let Some(input) = inputs.first() else {
            return Err(BiomimicryError::ValueTypeMismatch {
                function: step.name.clone(),
                expected: format!("two Int inputs for Arith({op:?})"),
                got: "none".into(),
            });
        };
        let mut scratch = self
            .scratch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = (cell, gene);
        if let Some(prior) = scratch.pending_binary.remove(&key) {
            drop(scratch);
            let pair = [prior, input.clone()];
            return step.call_many(&pair);
        }
        scratch.pending_binary.insert(key, input.clone());
        Ok(Vec::new())
    }
}

fn emit_value(step: &TransductionFn, input: &Signal, value: Value) -> Signal {
    Signal::new(
        crate::signal::SignalType::Operational,
        step.output_kind.clone(),
        step.output_scope,
        crate::signal::Payload::of(value),
        input.source,
        input.stamp,
    )
}

impl Transducer for CascadeTransducer {
    fn transduce(&self, cell: &Cell, sig: &Signal, gene: GeneId) -> Vec<Operation> {
        self.transduce_checked(cell, sig, gene).unwrap_or_default()
    }
}

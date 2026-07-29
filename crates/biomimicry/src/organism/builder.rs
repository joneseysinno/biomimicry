//! Assemble an organism from a genome + config.

use std::sync::Arc;

use crate::cell::{Cell, CellId, LifecycleState};
use crate::error::{BiomimicryError, Result};
use crate::expression::NetworkRegulator;
use crate::genesis::{GeneId, Genome};
use crate::homeostasis::PopulationSizeLoop;
use crate::metabolism::{Cadence, Population, Scheduler};
use crate::organism::Organism;
use crate::organism::root::new_organism;
use crate::substrate::MemoryStore;
use crate::transduction::CascadeTransducer;

/// Builder for [`Organism`].
#[derive(Debug)]
pub struct OrganismBuilder {
    genome: Option<Arc<Genome>>,
    seed: u64,
    cadence: Cadence,
    /// Initial living population size.
    population_size: usize,
    /// Gene activated on each seed / recruited cell.
    seed_gene: Option<GeneId>,
    /// Population-size set-point (defaults to `population_size`).
    target_population: Option<usize>,
    /// Install population-size homeostasis (default true).
    enable_pop_loop: bool,
    /// Use undamped aggressive population loop (oscillation demos).
    undamped_pop: bool,
    regulator: Option<NetworkRegulator>,
    transducer: Option<CascadeTransducer>,
    /// Auto-flush causal DAG after settle (default false).
    persist_on_settle: bool,
}

impl Default for OrganismBuilder {
    fn default() -> Self {
        Self {
            genome: None,
            seed: 0,
            cadence: Cadence::default(),
            population_size: 2,
            seed_gene: None,
            target_population: None,
            enable_pop_loop: true,
            undamped_pop: false,
            regulator: None,
            transducer: None,
            persist_on_settle: false,
        }
    }
}

impl OrganismBuilder {
    /// Create a builder with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the compiled genome.
    #[must_use]
    pub fn genome(mut self, genome: impl Into<Arc<Genome>>) -> Self {
        self.genome = Some(genome.into());
        self
    }

    /// Set the determinism seed.
    #[must_use]
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Set scheduler cadence.
    #[must_use]
    pub fn cadence(mut self, cadence: Cadence) -> Self {
        self.cadence = cadence;
        self
    }

    /// Initial number of Active cells.
    #[must_use]
    pub fn population_size(mut self, n: usize) -> Self {
        self.population_size = n.max(1);
        self
    }

    /// Gene activated on seed and recruited cells.
    #[must_use]
    pub fn seed_gene(mut self, gene: GeneId) -> Self {
        self.seed_gene = Some(gene);
        self
    }

    /// Population-size homeostatic set-point.
    #[must_use]
    pub fn target_population(mut self, n: usize) -> Self {
        self.target_population = Some(n.max(1));
        self
    }

    /// Disable the population-size loop (expression-only settle).
    #[must_use]
    pub fn without_pop_loop(mut self) -> Self {
        self.enable_pop_loop = false;
        self
    }

    /// Undamped aggressive population loop (limit-cycle demos).
    #[must_use]
    pub fn undamped_population(mut self) -> Self {
        self.undamped_pop = true;
        self.enable_pop_loop = true;
        self
    }

    /// Install a Phase 1 rule-network brain.
    #[must_use]
    pub fn regulator(mut self, regulator: NetworkRegulator) -> Self {
        self.regulator = Some(regulator);
        self
    }

    /// Install a Phase 2 cascade brain.
    #[must_use]
    pub fn transducer(mut self, transducer: CascadeTransducer) -> Self {
        self.transducer = Some(transducer);
        self
    }

    /// Auto-flush the causal DAG after a successful settle (default off).
    #[must_use]
    pub fn persist_on_settle(mut self, enabled: bool) -> Self {
        self.persist_on_settle = enabled;
        self
    }

    /// Build the organism.
    ///
    /// # Errors
    ///
    /// Returns an error if required configuration is missing.
    pub fn build(self) -> Result<Organism<MemoryStore>> {
        let genome = self.genome.ok_or_else(|| {
            BiomimicryError::Organism("OrganismBuilder: genome is required".into())
        })?;
        let seed_gene = self.seed_gene.ok_or_else(|| {
            BiomimicryError::Organism("OrganismBuilder: seed_gene is required".into())
        })?;

        let mut cells = Vec::with_capacity(self.population_size);
        for i in 0..self.population_size {
            let mut cell = Cell::new(CellId(i as u64 + 1), Arc::clone(&genome));
            cell.try_transition(LifecycleState::Differentiating)?;
            cell.try_transition(LifecycleState::Active)?;
            cell.activate(seed_gene);
            cells.push(cell);
        }
        let population = Population::from_cells(cells);
        let next_cell_id = self.population_size as u64 + 1;

        let mut scheduler = Scheduler::try_new(self.seed, self.cadence)?;
        if let Some(r) = self.regulator {
            scheduler.with_regulator(r);
        }
        let transducer = self
            .transducer
            .unwrap_or_else(|| CascadeTransducer::from_genome(&genome));
        scheduler.with_transducer(transducer);
        scheduler.enable_divide(next_cell_id, Some(seed_gene));

        let target = self.target_population.unwrap_or(self.population_size);
        let pop_loop = if self.enable_pop_loop {
            Some(if self.undamped_pop {
                let mut loop_ = PopulationSizeLoop::undamped(target);
                loop_.current = self.population_size;
                loop_
            } else {
                let mut loop_ = PopulationSizeLoop::new(target);
                loop_.current = self.population_size;
                loop_
            })
        } else {
            None
        };

        let mut org = new_organism(
            genome,
            population,
            scheduler,
            MemoryStore::new(),
            pop_loop,
            Some(seed_gene),
            next_cell_id,
        );
        org.persist_on_settle = self.persist_on_settle;
        Ok(org)
    }
}

pub mod data_store;
pub mod gene_pool;
pub mod types;
pub mod utils;
use std::{cell::Cell, rc::Rc};

use serde::Serialize;

use crate::{
    biology::{
        experiments::sim_runner::{ExperimentSimRunner, SimRunnerGenomeEntry},
        genome::framed::common::{CompiledFramedGenome, RawFramedGenome},
    },
    simulation::{
        common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod},
        fitness::FitnessScore,
        unit::{UnitAttributeValue, UnitResourceAmount},
    },
};

use self::{
    data_store::MultiPoolExperimentDataStore,
    gene_pool::ExperimentGenePool,
    types::{
        GenePoolSettings, MultiPoolExperimentLogger, MultiPoolExperimentSettings,
        MultiPoolExperimentState, MultiPoolLoggingSettings,
    },
};

pub struct MultiPoolExperiment {
    pub state: MultiPoolExperimentState,
    pub settings: MultiPoolExperimentSettings,
    // pub store: Box<dyn MultiPoolExperimentDataStore>,
    _logger: Option<MultiPoolExperimentLogger>,
}

impl MultiPoolExperiment {
    pub fn new(
        settings: MultiPoolExperimentSettings,
        gene_pool_settings: Vec<GenePoolSettings>,
    ) -> Self {
        let mut s = Self {
            state: MultiPoolExperimentState {
                current_tick: 0,
                gene_pools: vec![],
            },
            _logger: settings
                .logging_settings
                .clone()
                .map(|settings| make_logger(&settings)),

            settings,
        };

        s.initialize_gene_pools(gene_pool_settings);

        s
    }

    pub fn initialize_gene_pools(&mut self, gene_pool_settings: Vec<GenePoolSettings>) {
        self.state.gene_pools = gene_pool_settings
            .iter()
            .enumerate()
            .map(|(i, settings)| ExperimentGenePool::new(i, settings.clone()))
            .collect::<Vec<_>>();
    }

    pub fn resume(&mut self) {
        if self.state.gene_pools.len() == 0 {
            panic!("Gene pools initialized");
        }

        while self.state.current_tick < self.settings.max_iterations {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        for gene_pool in self.state.gene_pools.iter_mut() {
            gene_pool.execute_with_points(self.settings.evaluation_points_per_tick);
        }

        self.execute_reference_evaluation();
    }

    pub fn execute_reference_evaluation(&mut self) {
        let sim_settings = self.settings.reference_sim_settings.clone();
        let fitness_key = self.settings.reference_fitness_calculation_key.clone();
        let chemistry_builder = sim_settings.chemistry_options.clone();

        let entries = self
            .state
            .gene_pools
            .iter()
            .map(|gene_pool| {
                let (id, genome) = gene_pool
                    .genomes
                    .iter()
                    .enumerate()
                    .max_by(|(i, genome1), (j, genome2)| {
                        genome1.current_rank_score.cmp(&genome2.current_rank_score)
                    })
                    .unwrap();

                SimRunnerGenomeEntry {
                    gene_pool_id: gene_pool.id,
                    genome_idx: id,
                    genome_uid: genome.uid,
                    genome: genome.compiled_genome.as_ref().clone(),
                    execution_stats: genome.previous_execution_stats.clone(),
                }
            })
            .collect::<Vec<_>>();

        let mut runner =
            ExperimentSimRunner::new(chemistry_builder, entries, sim_settings, fitness_key);

        let result = runner.run_evaluation_for_uids();

        // TODO: log result somehow
    }
}

pub fn make_logger(settings: &MultiPoolLoggingSettings) -> MultiPoolExperimentLogger {
    MultiPoolExperimentLogger {}
}

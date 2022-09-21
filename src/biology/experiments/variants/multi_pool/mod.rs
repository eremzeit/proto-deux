pub mod builder;
pub mod data_store;
pub mod gene_pool;
pub mod logger;
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
    logger::MultiPoolExperimentLogger,
    types::{GenePoolSettings, MultiPoolExperimentSettings, MultiPoolExperimentState},
};

pub struct MultiPoolExperiment {
    pub state: MultiPoolExperimentState,
    pub settings: MultiPoolExperimentSettings,
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
                .map(|settings| MultiPoolExperimentLogger {
                    settings: settings.clone(),
                }),

            settings,
        };

        s.initialize_gene_pools(gene_pool_settings);

        s
    }

    fn initialize_gene_pools(&mut self, gene_pool_settings: Vec<GenePoolSettings>) {
        self.state.gene_pools = gene_pool_settings
            .iter()
            .enumerate()
            .map(|(i, settings)| ExperimentGenePool::new(i, settings.clone()))
            .collect::<Vec<_>>();
    }

    pub fn initialize(&mut self) {
        if let Some(logger) = &self._logger {
            logger.init(&self.state.gene_pools);
        }

        // self.populate_initial_genomes();
    }

    pub fn start(&mut self) {
        self.resume();
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

        println!("Experiment tick: {}", self.state.current_tick);

        if self.state.current_tick % 20 == 0 {
            self.shuffle_genomes_across_gene_pools();
        }

        if self.state.current_tick % 1 == 0 {
            self.print_fitness_summary();

            if let Some(logger) = &self._logger {
                for gene_pool in &self.state.gene_pools {
                    logger.log_gene_pool_summary(gene_pool);
                    logger.log_gene_pool_fitness_percentiles(gene_pool, self.state.current_tick);
                }
            }
        }

        self.state.current_tick += 1;
    }

    pub fn print_fitness_summary(&self) {
        let best_scores = self
            .state
            .gene_pools
            .iter()
            .map(|gene_pool| {
                let i = gene_pool._highest_fitness_idx();
                (
                    gene_pool.settings.name_key.clone(),
                    gene_pool.state.genome_entries[i].max_fitness_metric,
                )
                // gene_pool
                //     .state
                //     .genome_entries
                //     .iter()
                //     .map(|g| (gene_pool.id, g.uid, g.max_fitness_metric))
                //     .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        println!("best scores: {:?}", best_scores)
    }

    pub fn shuffle_genomes_across_gene_pools(&mut self) {
        let best_genomes = self
            .state
            .gene_pools
            .iter()
            .map(|gene_pool| {
                let i = gene_pool._highest_fitness_idx();
                gene_pool.state.genome_entries[i]
                    .compiled_genome
                    .as_ref()
                    .clone()
            })
            .collect::<Vec<_>>();

        for gene_pool in self.state.gene_pools.iter_mut() {
            if gene_pool.settings.receive_external_genomes {
                gene_pool
                    .state
                    .external_genomes_queue
                    .append(&mut best_genomes.clone());
            }
        }
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
                    .state
                    .genome_entries
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

        let results = runner.run_evaluation_for_uids();

        if let Some(logger) = &self._logger {
            logger.log_reference_eval_results(&results, self.state.current_tick);
        }
    }
}

use std::{cell::Cell, rc::Rc};

use serde::Serialize;

use crate::{
    biology::{
        experiments::{
            alterations::CompiledAlterationSet,
            fitness::FitnessRankAdjustmentMethod,
            types::{CullStrategy, ExperimentGenomeUid, ExperimentSimSettings, SeedGenomeSettings},
        },
        genome::framed::{
            annotated::FramedGenomeExecutionStats,
            common::{CompiledFramedGenome, RawFramedGenome},
        },
    },
    simulation::{
        common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod},
        fitness::FitnessScore,
        unit::{UnitAttributeValue, UnitResourceAmount},
    },
};

use super::gene_pool::ExperimentGenePool;

// use super::FitnessCycleStrategy;
#[derive(Clone)]
pub struct MultiPoolExperimentState {
    pub current_tick: u64,
    pub gene_pools: Vec<ExperimentGenePool>,
}

#[derive(Serialize, Clone)]
pub struct MultiPoolExperimentSettings {
    pub max_iterations: u64,
    pub chemistry_key: String,
    pub experiment_key: String,
    pub logging_settings: Option<MultiPoolLoggingSettings>,
    pub evaluation_points_per_tick: usize,

    pub reference_sim_settings: ExperimentSimSettings,
    pub reference_fitness_calculation_key: String,
}

#[derive(Serialize, Clone)]
pub struct MultiPoolLoggingSettings {}

#[derive(Clone)]
pub struct GenePoolSettings {
    pub sim_settings: ExperimentSimSettings,
    pub num_genomes: usize,
    pub alteration_specs: CompiledAlterationSet,
    pub fitness_calculation_key: String,
    pub fitness_cycle_strategy: FitnessCycleStrategy,
    pub name_key: String,
    pub fitness_rank_adjustment_method: FitnessRankAdjustmentMethod,
    pub seed_genome_settings: SeedGenomeSettings,
    pub cull_strategy: CullStrategy,
}

pub struct MultiPoolExperimentLogger {}

#[derive(Clone)]
pub enum FitnessCycleStrategy {
    // every genome is tested every cycle
    Exaustive {
        group_scramble_pct: f32,
    },

    // a subset of genomes are tested each cycle
    RandomSubset {
        percent_of_genomes: f32,
        group_scramble_pct: f32,
    }, // every genome is tested every cycle
}

// #[derive(Clone)]
// pub struct GenomeExperimentEntry {
//     pub last_fitness_metrics: Vec<FitnessScore>,
//     pub max_fitness_metric: Option<FitnessScore>,
//     pub num_evaluations: usize,
//     pub raw_genome: RawFramedGenome,
//     pub uid: ExperimentGenomeUid,
//     pub current_rank_score: usize,
//     pub compiled_genome: Rc<CompiledFramedGenome>,
// }

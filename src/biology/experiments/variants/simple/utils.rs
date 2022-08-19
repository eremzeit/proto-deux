use crate::biology::experiments::alterations;
use crate::biology::phenotype::framed::common::*;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use std::fmt::{Debug, Formatter, Result};
use std::path::PathBuf;

use super::logger::LoggingSettings;

pub type _ExperimentGenomeId = usize;
pub type ExperimentGenomeUid = usize;

pub type GenomeEntryId = usize;

pub enum CullStrategy {
    WorstFirst,
}

#[derive(Clone)]
pub struct GenomeExperimentEntry {
    pub last_fitness_metrics: Vec<FitnessScore>,
    pub max_fitness_metric: Option<FitnessScore>,
    pub num_evaluations: usize,
    pub genome: RawFramedGenome,
    pub uid: ExperimentGenomeUid,
    pub current_rank_score: usize,
}

pub struct ExperimentSimSettings {
    pub num_simulation_ticks: u64,
    pub grid_size: (usize, usize),
    pub num_genomes_per_sim: usize,
    // pub iterations: usize,
    pub default_unit_resources: Vec<(&'static str, UnitResourceAmount)>,
    pub default_unit_attr: Vec<(&'static str, UnitAttributeValue)>,
}

pub struct SimpleExperimentSettings {
    pub experiment_key: String,
    pub logging_settings: Option<LoggingSettings>,
    pub num_genomes: usize,
    pub iterations: usize,
    pub sim_settings: ExperimentSimSettings,
    pub alteration_set: alterations::AlterationTypeSet,
    pub fitness_calculation_key: String, // needed?  should this be a trait object?  how will fitness calculation change?
    pub cull_strategy: CullStrategy,
    pub specs: SimulationSpecs,
}

const DATA_DIR_NAME: &str = "data";

pub fn get_data_dir() -> PathBuf {
    find_folder::Search::Parents(1)
        .for_folder(DATA_DIR_NAME)
        .expect("cant find data dir")
        .to_path_buf()
}

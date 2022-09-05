use crate::biology::experiments::alterations;
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result};
use std::path::PathBuf;

use super::logger::LoggingSettings;

/**
 * Uniquely identifies a genome over the course of the entire experiment
 */
pub type ExperimentGenomeUid = usize;

/**
 * Stores the current offset of this genome in the genome entries table.  That table
 * changes so this is a value of ephemeral semantics.
 */
pub type GenomeEntryId = usize;

#[derive(Serialize, Deserialize)]
pub enum CullStrategy {
    WorstFirst,
}

#[derive(Clone)]
pub struct GenomeExperimentEntry {
    pub last_fitness_metrics: Vec<FitnessScore>,
    pub max_fitness_metric: Option<FitnessScore>,
    pub num_evaluations: usize,
    pub raw_genome: RawFramedGenome,
    pub uid: ExperimentGenomeUid,
    pub current_rank_score: usize,
    pub compiled_genome: Rc<CompiledFramedGenome>,
}

// impl GenomeExperimentEntry {
//     pub fn compile(&mut self, gm: &GeneticManifest) -> Rc<CompiledFramedGenome> {
//         if self.compiled_genome.is_none() {
//             self.compiled_genome = Some(FramedGenomeCompiler::compile(self.raw_genome.clone(), gm));
//         }

//         self.compiled_genome.clone().unwrap()
//     }
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct ExperimentSimSettings {
    pub num_simulation_ticks: u64,
    pub grid_size: (usize, usize),
    pub num_genomes_per_sim: usize,
    pub default_unit_resources: Vec<(String, UnitResourceAmount)>,
    pub default_unit_attr: Vec<(String, UnitAttributeValue)>,
    pub place_units_method: PlaceUnitsMethod,
    pub chemistry_options: ChemistryBuilder,
}

pub type MaybeLoggingSettings = Option<LoggingSettings>;

#[derive(Builder)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct SimpleExperimentSettings {
    pub experiment_key: String,
    pub logging_settings: MaybeLoggingSettings,
    pub num_genomes: usize,
    pub iterations: usize,
    pub sim_settings: ExperimentSimSettings,
    pub alteration_set: alterations::CompiledAlterationSet,
    pub fitness_calculation_key: String, // needed?  should this be a trait object?  how will fitness calculation change?
    pub cull_strategy: CullStrategy,
    // pub gm: Rc<GeneticManifest>, // note: eventually this might be defined on a per-genome basis
}

// pub mod builder {
//     use super::*;

//     pub struct SimpleExperimentSettings {
//         pub experiment_key: String,
//         pub logging_settings: MaybeLoggingSettings,
//         pub num_genomes: usize,
//         pub iterations: usize,
//         pub sim_settings: ExperimentSimSettings,
//         pub alteration_set: alterations::AlterationTypeSet,
//         pub fitness_calculation_key: String,
//         pub cull_strategy: CullStrategy,
//         pub chemistry_options: ChemistryBuilder,
//         pub gm: GeneticManifest,
//     }
// }

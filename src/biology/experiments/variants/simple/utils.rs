use crate::biology::experiments::alterations;
use crate::biology::experiments::types::{
    CullStrategy, ExperimentGenomeUid, ExperimentSimSettings,
};
use crate::biology::experiments::variants::multi_pool::types::FitnessCycleStrategy;
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::annotated::FramedGenomeExecutionStats;
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

use super::logger::SimpleExperimentLoggingSettings;

pub type MaybeLoggingSettings = Option<SimpleExperimentLoggingSettings>;

#[derive(Builder)]
#[builder(pattern = "owned", setter(strip_option))]
pub struct SimpleExperimentSettings {
    pub experiment_key: String,
    pub logging_settings: MaybeLoggingSettings,
    pub num_genomes: usize,
    pub iterations: u64,
    pub sim_settings: ExperimentSimSettings,
    pub alteration_set: alterations::CompiledAlterationSet,
    pub fitness_calculation_key: String, // needed?  should this be a trait object?  how will fitness calculation change?
    pub cull_strategy: CullStrategy,
    pub fitness_cycle_strategy: FitnessCycleStrategy, // pub gm: Rc<GeneticManifest>, // note: eventually this might be defined on a per-genome basis
}

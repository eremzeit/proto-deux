use crate::biology::experiments::alterations;
use crate::biology::experiments::variants::simple::utils::{
    ExperimentGenomeUid, GenomeExperimentEntry,
};
use crate::biology::unit_behavior::framed::common::*;
use crate::perf::{perf_timer_start, perf_timer_stop};
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use std::fmt::{Debug, Formatter, Result};
use std::ops::{Add, Div};
use std::time::Duration;

use super::logger::SimpleExperimentLogger;
use super::utils::{
    CullStrategy, ExperimentSimSettings, MaybeLoggingSettings, SimpleExperimentSettings,
};
use crate::biology::genome::framed::samples;

pub struct SimpleExperimentData {
    pub is_paused: bool,
    pub is_initialized: bool,
    pub genome_entries: Vec<GenomeExperimentEntry>,
    pub is_headless: bool,
    pub current_tick: u64,
    pub settings: SimpleExperimentSettings,
    pub _last_entry_id: usize,

    _logger: Option<SimpleExperimentLogger>,
    _seed_genomes: Option<Vec<RawFramedGenome>>,
}

pub struct SimpleExperimentSettingsData {
    pub experiment_key: String,
    pub logging_settings: MaybeLoggingSettings,
    pub num_genomes: usize,
    pub iterations: usize,
    pub sim_settings: ExperimentSimSettings,
    pub alteration_set: alterations::CompiledAlterationSet,
    pub fitness_calculation_key: String, // needed?  should this be a trait object?  how will fitness calculation change?
    pub cull_strategy: CullStrategy,
    pub chemistry_options: ChemistryBuilder,
    pub gm: Rc<GeneticManifest>,
}

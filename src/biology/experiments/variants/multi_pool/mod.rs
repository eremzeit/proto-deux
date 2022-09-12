use std::{cell::Cell, rc::Rc};

use serde::Serialize;

use crate::{
    biology::{
        experiments::alterations::AlterationManifest,
        genome::framed::common::{CompiledFramedGenome, RawFramedGenome},
    },
    simulation::{
        common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod},
        fitness::FitnessScore,
        unit::{UnitAttributeValue, UnitResourceAmount},
    },
};

use super::simple::utils::{CullStrategy, ExperimentGenomeUid};

pub struct MultiPoolExperiment {
    state: MultiPoolExperimentState,
    settings: MultiPoolExperimentSettings,
    store: Box<dyn MultiPoolExperimentDataStore>,
    // _gm: Rc<GeneticManifest>, // a cached copy.  note that this might eventually change depending on the genome.
    _logger: Option<MultiPoolExperimentLogger>,
    _seed_genomes: Option<Vec<RawFramedGenome>>,
}

pub trait MultiPoolExperimentDataStore {
    fn save_snapshot(&mut self, experiment: &MultiPoolExperiment);
    fn load_snapshot(&mut self, experiment_key: &str) -> MultiPoolExperiment;

	fn save_genepool()
}

#[derive(Clone)]
pub struct MultiPoolExperimentState {
    pub current_tick: u64,
    pub gene_pools: Vec<ExperimentGenePool>,
}

#[derive(Serialize, Clone)]
pub struct MultiPoolExperimentSettings {
    pub chemistry_key: String,
    pub experiment_key: String,
    // pub logging_settings: MaybeLoggingSettings,
}

pub struct MultiPoolExperimentLogger {}

#[derive(Clone)]
pub struct ExperimentGenePool {
	pub id: usize,
    pub genomes: Vec<GenomeExperimentEntry>,
    _last_entry_id: usize,
    pub settings: GenePoolSimSettings,
}

#[derive(Clone)]
pub struct GenePoolSimSettings {
    pub num_simulation_ticks: u64,
    pub grid_size: (usize, usize),
    pub num_genomes_per_sim: usize,
    pub default_unit_resources: Vec<(String, UnitResourceAmount)>,
    pub default_unit_attr: Vec<(String, UnitAttributeValue)>,
    pub place_units_method: PlaceUnitsMethod,
    pub chemistry_options: ChemistryBuilder,
}

pub struct GenePoolSettings {
    pub sim_settings: GenePoolSimSettings,
    pub num_genomes: usize,
    pub alteration_specs: AlterationManifest,
    pub fitness_calculation_key: String,
    // pub cull_strategy: CullStrategy,
    pub fitness_cycle_strategy: FitnessCycleStrategy,
}

#[derive(Clone)]
pub struct GenomeExperimentEntry {
    pub last_fitness_metrics: Vec<FitnessScore>,
    pub max_fitness_metric: Option<FitnessScore>,
    pub num_evaluations: usize,
    pub uid: ExperimentGenomeUid,
    pub current_rank_score: usize,
    pub compiled_genome: Rc<CompiledFramedGenome>,
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

pub enum FitnessCycleStrategy {
    // every genome is tested every cycle
    Exaustive {
        cull_strategy: CullStrategy,
    },

    // a subset of genomes are tested each cycle
    RandomSubset {
        percent: f32,
        cull_strategy: CullStrategy,
    }, // every genome is tested every cycle
}
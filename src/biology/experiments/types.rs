use std::{
    fmt::{Debug, Formatter, Result},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    biology::genome::framed::{
        annotated::FramedGenomeExecutionStats, common::CompiledFramedGenome,
    },
    simulation::{
        common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod, UnitEntryId},
        fitness::FitnessScore,
        unit::{UnitAttributeValue, UnitResourceAmount},
    },
};

use super::variants::multi_pool::gene_pool::GenePoolId;

/**
 * Uniquely identifies a genome over the course of the entire experiment
 */
pub type ExperimentGenomeUid = usize;

/**
 * Stores the current offset of this genome in the genome entries table.  That table
 * changes so this is a value of ephemeral semantics.
 */
pub type GenomeEntryId = usize;

#[derive(Serialize, Deserialize, Clone)]
pub enum CullStrategy {
    WorstFirst { percent: f32 },
    RandomTiers { percent_per_tercile: [f32; 3] },
}

#[derive(Clone)]
pub enum SeedGenomeSettings {
    Random { min_size: usize, max_size: usize },
}

#[derive(Clone)]
pub struct GenomeExperimentEntry {
    pub last_fitness_metrics: Vec<FitnessScore>,
    pub max_fitness_metric: Option<FitnessScore>,
    pub num_evaluations: usize,
    pub uid: ExperimentGenomeUid,
    pub current_rank_score: usize,
    pub compiled_genome: Rc<CompiledFramedGenome>,
    pub previous_execution_stats: FramedGenomeExecutionStats,
}

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

#[derive(Clone)]
pub struct TrialResultItem {
    pub sim_unit_entry_id: UnitEntryId,

    // Refers to the index of the genome in the current experiment genome listing.  As such,
    // TrialResultItems are only valid data until the genome listing is updated.
    pub genome_idx: GenomeEntryId,
    pub experiment_genome_uid: ExperimentGenomeUid,
    pub fitness_score: FitnessScore,
    pub stats: FramedGenomeExecutionStats,
    pub gene_pool_id: GenePoolId,
}

impl Debug for TrialResultItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "TrialResult(uid: {}, score: {})",
            self.experiment_genome_uid, self.fitness_score
        )
    }
}

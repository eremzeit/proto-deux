use crate::{
    chemistry::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod, ChemistryConfiguration,
    },
    simulation::unit::{UnitAttributeValue, UnitResourceAmount},
};

use super::{
    alterations::{default_alteration_set, CompiledAlterationSet},
    fitness::FitnessRankAdjustmentMethod,
    types::{CullStrategy, ExperimentSimSettings, SeedGenomeSettings},
    variants::multi_pool::types::{FitnessCycleStrategy, GenePoolSettings},
};

#[derive(Builder, Clone)]
#[builder(
    name = "ExperimentSimSettingsBuilder",
    pattern = "mutable",
    setter(strip_option),
    build_fn(skip)
)]
pub struct MultiPoolExperimentSettingsBuilderTemplate {
    pub num_simulation_ticks: u64,
    pub grid_size: (usize, usize),
    pub num_genomes_per_sim: usize,
    pub default_unit_resources: Vec<(String, UnitResourceAmount)>,
    pub default_unit_attr: Vec<(String, UnitAttributeValue)>,
    pub place_units_method: PlaceUnitsMethod,
    pub chemistry_key: String,
    pub chemistry_configuration: ChemistryConfiguration,
}

impl ExperimentSimSettingsBuilder {
    pub fn build(&mut self) -> ExperimentSimSettings {
        let mut chemistry_builder =
            ChemistryBuilder::with_key(&self.chemistry_key.clone().unwrap());
        if self.chemistry_configuration.is_some() {
            chemistry_builder =
                chemistry_builder.config(self.chemistry_configuration.clone().unwrap());
        }

        ExperimentSimSettings {
            num_simulation_ticks: self.num_simulation_ticks.unwrap(),
            grid_size: self.grid_size.unwrap(),
            num_genomes_per_sim: self.num_genomes_per_sim.unwrap(),
            default_unit_resources: self.default_unit_resources.clone().unwrap_or(vec![]),
            default_unit_attr: self.default_unit_attr.clone().unwrap_or(vec![]).clone(),
            place_units_method: self
                .place_units_method
                .clone()
                .unwrap_or(PlaceUnitsMethod::Default),
            chemistry_options: chemistry_builder,
        }
    }
}

#[derive(Builder)]
#[builder(
    name = "GenePoolSettingsBuilder",
    pattern = "mutable",
    setter(strip_option),
    build_fn(skip)
)]
pub struct GenePoolSettingsBuilderTemplate {
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

impl GenePoolSettingsBuilder {
    pub fn build(&mut self) -> GenePoolSettings {
        GenePoolSettings {
            sim_settings: self.sim_settings.clone().unwrap(),
            num_genomes: self.num_genomes.clone().unwrap(),
            alteration_specs: self
                .alteration_specs
                .clone()
                .unwrap_or(default_alteration_set()),
            fitness_calculation_key: self.fitness_calculation_key.clone().unwrap(),
            fitness_cycle_strategy: self.fitness_cycle_strategy.clone().unwrap(),
            name_key: self.name_key.clone().unwrap_or("".to_string()),
            fitness_rank_adjustment_method: self.fitness_rank_adjustment_method.clone().unwrap(),
            seed_genome_settings: self.seed_genome_settings.clone().unwrap(),
            cull_strategy: self.cull_strategy.clone().unwrap(),
        }
    }
}

use crate::{
    biology::experiments::{
        alterations,
        types::{CullStrategy, ExperimentSimSettings},
        variants::{
            multi_pool::types::FitnessCycleStrategy,
            simple::{
                logger::SimpleExperimentLoggingSettings, utils::SimpleExperimentSettings,
                SimpleExperiment,
            },
        },
    },
    simulation::common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod},
};

use crate::biology::genome::framed::builders::*;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::*;
use std::rc::Rc;

pub fn test_fitness(key: &str) {
    let exp_key = key.to_string();
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let gm = GeneticManifest::from_chemistry(&chemistry_builder.build()).wrap_rc();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst { percent: 0.30 },
        fitness_calculation_key: "total_cheese_acquired".to_string(),
        num_genomes: 4,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 10,
            grid_size: (10, 1),
            num_genomes_per_sim: 4,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
            chemistry_options: chemistry_builder,
        },

        iterations: 1,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: exp_key.clone(),
        logging_settings: None,
        fitness_cycle_strategy: FitnessCycleStrategy::Exaustive {
            group_scramble_pct: 0.30,
        },
    };

    let mut exp = SimpleExperiment::new(settings);

    exp.with_seed_genomes(vec![]);

    // exp.genome_entries[0].genome = genome_vals1.clone();
    // exp.genome_entries[1].genome = genome_vals2.clone();
    // exp.genome_entries[2].genome = genome_vals3.clone();
    // exp.genome_entries[3].genome = genome_vals4.clone();

    exp.start();
}

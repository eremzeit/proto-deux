use crate::{
    biology::experiments::{
        alterations,
        variants::simple::{
            logger::LoggingSettings,
            utils::{CullStrategy, ExperimentSimSettings, SimpleExperimentSettings},
            SimpleExperiment,
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
    let gm = GeneticManifest::defaults(&chemistry_builder.manifest()).wrap_rc();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "total_cheese_consumed".to_string(),
        num_genomes: 4,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 10,
            grid_size: (10, 1),
            num_genomes_per_sim: 4,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        },

        iterations: 1,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: exp_key.clone(),
        logging_settings: None,
        chemistry_options: chemistry_builder,
        gm,
    };

    let mut exp = SimpleExperiment::new(settings);

    exp.with_seed_genomes(vec![]);

    // exp.genome_entries[0].genome = genome_vals1.clone();
    // exp.genome_entries[1].genome = genome_vals2.clone();
    // exp.genome_entries[2].genome = genome_vals3.clone();
    // exp.genome_entries[3].genome = genome_vals4.clone();

    exp.start();
}

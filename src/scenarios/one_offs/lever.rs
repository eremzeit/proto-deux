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
    let gm = GeneticManifest::from_chemistry(&chemistry_builder.build()).wrap_rc();

    let genome_vals1 = frame_from_single_channel(vec![gene(
        if_any!(if_all!(conditional!(is_truthy, 1))),
        then_do!(pull_lever, 1),
    )])
    .build(&gm);

    let genome_vals2 = frame_from_single_channel(vec![gene(
        if_any!(if_all!(conditional!(is_truthy, 1))),
        then_do!(pull_lever, 5),
    )])
    .build(&gm);

    let genome_vals3 = frame_from_single_channel(vec![gene(
        if_any!(if_all!(conditional!(is_truthy, 1))),
        then_do!(pull_lever, 10),
    )])
    .build(&gm);

    let genome_vals4 = frame_from_single_channel(vec![gene(
        if_any!(if_all!(conditional!(is_truthy, 1))),
        then_do!(pull_lever, 20),
    )])
    .build(&gm);

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "lever_pulls".to_string(),
        num_genomes: 4,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 10,
            grid_size: (10, 1),
            num_genomes_per_sim: 4,
            // iterations: 5,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        },

        iterations: 1,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: exp_key.clone(),
        logging_settings: Some(LoggingSettings {
            experiment_key: exp_key.clone(),
            allow_overwrite: true,
            checkpoint_interval: 1,
        }),
        gm,
        chemistry_options: chemistry_builder,
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();

    exp.genome_entries[0].genome = genome_vals1.clone();
    exp.genome_entries[1].genome = genome_vals2.clone();
    exp.genome_entries[2].genome = genome_vals3.clone();
    exp.genome_entries[3].genome = genome_vals4.clone();

    exp.resume();
}

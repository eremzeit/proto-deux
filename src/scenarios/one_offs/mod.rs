use crate::{
    biology::experiments::{
        alterations,
        variants::simple::{
            logger::LoggingSettings,
            utils::{CullStrategy, ExperimentSimSettings, SimpleExperimentSettings},
            SimpleExperiment,
        },
    },
    simulation::{common::helpers::place_units::PlaceUnitsMethod, specs::SimulationSpecs},
};

use crate::biology::genome::framed::builders::*;
use crate::biology::phenotype::framed::common::*;
use crate::simulation::common::*;
use std::rc::Rc;

pub fn run_one_off(scenario_key: &str) {
    match scenario_key {
        "test_fitness" => {
            test_fitness();
        }
        _ => {
            panic!("Scenario key not found: {}", scenario_key);
        }
    };
}

pub fn test_fitness() {
    let exp_key = "one_off_lever_exp".to_string();
    let specs = SimulationSpecs {
        chemistry_key: "lever".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };

    let (cm, sm, gm) = specs.context();

    let genome_vals1 = frame(
        0,
        vec![gene(
            if_any!(if_all!(conditional!(is_truthy, 1))),
            then_do!(pull_lever, 1),
        )],
    )
    .build(&sm, &cm, &gm);

    let genome_vals2 = frame(
        0,
        vec![gene(
            if_any!(if_all!(conditional!(is_truthy, 1))),
            then_do!(pull_lever, 5),
        )],
    )
    .build(&sm, &cm, &gm);

    let genome_vals3 = frame(
        0,
        vec![gene(
            if_any!(if_all!(conditional!(is_truthy, 1))),
            then_do!(pull_lever, 10),
        )],
    )
    .build(&sm, &cm, &gm);

    let genome_vals4 = frame(
        0,
        vec![gene(
            if_any!(if_all!(conditional!(is_truthy, 1))),
            then_do!(pull_lever, 20),
        )],
    )
    .build(&sm, &cm, &gm);

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
        },

        iterations: 1,
        specs: specs,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: exp_key.clone(),
        logging_settings: Some(LoggingSettings {
            experiment_key: exp_key.clone(),
            allow_overwrite: true,
            checkpoint_interval: 1,
        }),
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();

    exp.genome_entries[0].genome = genome_vals1.clone();
    exp.genome_entries[1].genome = genome_vals2.clone();
    exp.genome_entries[2].genome = genome_vals3.clone();
    exp.genome_entries[3].genome = genome_vals4.clone();

    exp.resume();
}

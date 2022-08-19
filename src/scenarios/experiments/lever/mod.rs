use crate::{
    biology::experiments::{
        alterations::{self, AlterationTypeSet},
        variants::simple::{
            logger::LoggingSettings,
            utils::{CullStrategy, ExperimentSimSettings, SimpleExperimentSettings},
            SimpleExperiment,
        },
    },
    runners::ExperimentRunnerArgs,
    simulation::{
        common::{
            get_chemistry_by_key, helpers::place_units::PlaceUnitsMethod, ChemistryConfiguration,
            SensorManifest,
        },
        specs::SimulationSpecs,
    },
    tests::GeneticManifest,
};

pub fn get_experiment_scenario(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
    match runner_args.experiment_scenario_key.as_str() {
        "simple" => simple_experiment(runner_args.clone()),
        _ => panic!("scenario not defined"),
    }
}

pub fn alterations() -> AlterationTypeSet {
    AlterationTypeSet::from_keys(&vec![
        "insertion".to_string(),
        "point_mutation".to_string(),
        // "deletion".to_string(),
    ])
}

pub fn simple_experiment(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
    let specs = SimulationSpecs {
        chemistry_key: "lever".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };

    let (cm, sm, gm) = specs.context();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "lever_pulls".to_string(),
        num_genomes: 10,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 100,
            grid_size: (100, 100),
            num_genomes_per_sim: 10,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
        },

        iterations: 5000,
        specs: specs,
        alteration_set: alterations(),
        experiment_key: runner_args.experiment_name_key.to_string(),
        logging_settings: Some(LoggingSettings {
            experiment_key: runner_args.experiment_name_key.to_string(),
            allow_overwrite: true,
            checkpoint_interval: 1000,
        }),
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();
    // exp.resume();
    exp
}

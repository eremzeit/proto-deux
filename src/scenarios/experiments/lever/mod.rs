use crate::{
    biology::experiments::{
        alterations::{self, CompiledAlterationSet},
        types::{CullStrategy, ExperimentSimSettings},
        variants::simple::{
            logger::LoggingSettings, utils::SimpleExperimentSettings, SimpleExperiment,
        },
    },
    runners::ExperimentRunnerArgs,
    simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod, ChemistryConfiguration,
        GeneticManifest, GeneticManifestData, SensorManifest,
    },
};

pub fn alterations() -> CompiledAlterationSet {
    CompiledAlterationSet::from_keys(&vec![
        "insertion".to_string(),
        "point_mutation".to_string(),
        // "deletion".to_string(),
    ])
}

pub fn simple_experiment(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
    let chemistry_builder = ChemistryBuilder::with_key("lever");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst { percent: 0.30 },
        fitness_calculation_key: "lever_pulls".to_string(),
        num_genomes: 10,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 10,
            grid_size: (100, 100),
            num_genomes_per_sim: 10,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
            chemistry_options: chemistry_builder,
        },

        iterations: 5000,
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

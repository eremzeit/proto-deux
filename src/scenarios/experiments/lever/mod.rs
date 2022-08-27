use crate::{
    biology::experiments::{
        alterations::{self, CompiledAlterationSet},
        variants::simple::{
            logger::LoggingSettings,
            utils::{CullStrategy, ExperimentSimSettings, SimpleExperimentSettings},
            SimpleExperiment,
        },
    },
    runners::ExperimentRunnerArgs,
    simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod, ChemistryConfiguration,
        GeneticManifest, SensorManifest,
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
    let gm = GeneticManifest::defaults(chemistry.get_manifest()).wrap_rc();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "lever_pulls".to_string(),
        num_genomes: 10,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 10,
            grid_size: (100, 100),
            num_genomes_per_sim: 10,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        },

        iterations: 5000,
        alteration_set: alterations(),
        experiment_key: runner_args.experiment_name_key.to_string(),
        logging_settings: Some(LoggingSettings {
            experiment_key: runner_args.experiment_name_key.to_string(),
            allow_overwrite: true,
            checkpoint_interval: 1000,
        }),
        chemistry_options: chemistry_builder,
        gm: gm,
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();
    // exp.resume();
    exp
}

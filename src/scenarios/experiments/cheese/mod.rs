use crate::genome::framed::samples;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::{GeneticManifest, GeneticManifestData};
use crate::{
    biology::experiments::{
        alterations::{self, CompiledAlterationSet},
        variants::simple::{
            logger::LoggingSettings,
            utils::{
                CullStrategy, ExperimentSimSettings, SimpleExperimentSettings,
                SimpleExperimentSettingsBuilder,
            },
            SimpleExperiment,
        },
    },
    runners::ExperimentRunnerArgs,
    simulation::common::{
        helpers::place_units::PlaceUnitsMethod, ChemistryConfiguration, SensorManifest,
    },
};

// pub fn get_experiment_scenario(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
//     match runner_args.experiment_scenario_key.as_str() {
//         "simple" => simple_experiment(runner_args.clone()),
//         _ => panic!("scenario not defined"),
//     }
// }

pub fn alterations() -> CompiledAlterationSet {
    CompiledAlterationSet::from_keys(&vec![
        "insertion".to_string(),
        "point_mutation".to_string(),
        "deletion".to_string(),
        "crossover".to_string(),
        "random_region_insert".to_string(),
        // "swap_frames".to_string(), // might have a bug
    ])
}

pub fn simple_experiment(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "total_cheese_consumed".to_string(),
        num_genomes: 30,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 70,
            // num_simulation_ticks: 2,
            grid_size: (20, 20),
            num_genomes_per_sim: 10,
            default_unit_resources: vec![("cheese".to_owned(), 100)],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::Default,
            chemistry_options: chemistry_builder,
        },

        iterations: 10001,
        // iterations: 100000000,
        alteration_set: alterations(),
        experiment_key: runner_args.experiment_name_key.to_string(),
        logging_settings: Some(LoggingSettings {
            experiment_key: runner_args.experiment_name_key.to_string(),
            allow_overwrite: true,
            checkpoint_interval: 2000,
        }),
    };

    let mut exp = SimpleExperiment::new(settings);

    // use crate::biology::genome::framed::samples::cheese::get_genome1_raw;
    // exp.with_seed_genomes(vec![get_genome1_raw(&gm)]);

    // exp.initialize();
    // exp.resume();
    exp
}

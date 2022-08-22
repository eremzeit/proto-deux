use crate::genome::framed::samples;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::GeneticManifest;
use crate::{
    biology::experiments::{
        alterations::{self, AlterationTypeSet},
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

pub fn alterations() -> AlterationTypeSet {
    AlterationTypeSet::from_keys(&vec![
        "insertion".to_string(),
        "point_mutation".to_string(),
        // "deletion".to_string(),
    ])
}
pub fn simple_experiment(runner_args: ExperimentRunnerArgs) -> SimpleExperiment {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::defaults(chemistry.get_manifest()).wrap_rc();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "total_cheese_consumed".to_string(),
        num_genomes: 20,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 100,
            grid_size: (20, 20),
            num_genomes_per_sim: 10,
            default_unit_resources: vec![("cheese", 200)],
            default_unit_attr: vec![],
            place_units_method: PlaceUnitsMethod::Default,
        },

        iterations: 100000,
        alteration_set: alterations(),
        experiment_key: runner_args.experiment_name_key.to_string(),
        logging_settings: Some(LoggingSettings {
            experiment_key: runner_args.experiment_name_key.to_string(),
            allow_overwrite: true,
            checkpoint_interval: 2000,
        }),

        chemistry_options: chemistry_builder,
        gm: gm.clone(),
    };

    let mut exp = SimpleExperiment::new(settings);

    // use crate::biology::genome::framed::samples::cheese::get_genome1_raw;
    // exp.with_seed_genomes(vec![get_genome1_raw(&gm)]);

    // exp.initialize();
    // exp.resume();
    exp
}

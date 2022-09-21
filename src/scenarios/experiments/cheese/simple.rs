use crate::biology::experiments::alterations::default_alteration_set;
use crate::biology::experiments::fitness::FitnessRankAdjustmentMethod;
use crate::biology::experiments::types::SeedGenomeSettings;
use crate::biology::experiments::variants::multi_pool::builder::MultiPoolExperimentSettingsBuilder;
use crate::biology::experiments::variants::multi_pool::logger::MultiPoolExperimentLoggingSettings;
use crate::biology::experiments::variants::multi_pool::types::{
    FitnessCycleStrategy, GenePoolSettings,
};
use crate::biology::experiments::variants::multi_pool::MultiPoolExperiment;
use crate::biology::experiments::{
    builders::{ExperimentSimSettingsBuilder, GenePoolSettingsBuilder},
    types::{CullStrategy, ExperimentSimSettings},
};
use crate::chemistry::ChemistryConfigBuilder;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::GeneticManifest;
use crate::{
    biology::experiments::{
        alterations::CompiledAlterationSet,
        variants::simple::{
            logger::SimpleExperimentLoggingSettings, utils::SimpleExperimentSettings,
            SimpleExperiment,
        },
    },
    runners::ExperimentRunnerArgs,
    simulation::common::helpers::place_units::PlaceUnitsMethod,
};

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
        experiment_key: runner_args.experiment_name_key.to_string(),
        logging_settings: Some(SimpleExperimentLoggingSettings {
            experiment_key: runner_args.experiment_name_key.to_string(),
            allow_overwrite: true,
            checkpoint_interval: 2000,
        }),
        num_genomes: 100,
        iterations: 1000000,
        // iterations: 10001,
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
        // iterations: 100000000,
        alteration_set: alterations(),
        fitness_calculation_key: "total_cheese_acquired".to_string(),
        cull_strategy: CullStrategy::WorstFirst { percent: 0.30 },
        fitness_cycle_strategy: FitnessCycleStrategy::Exaustive {
            group_scramble_pct: 0.40,
        },
    };

    let mut exp = SimpleExperiment::new(settings);

    // use crate::biology::genome::framed::samples::cheese::get_genome1_raw;
    // exp.with_seed_genomes(vec![get_genome1_raw(&gm)]);

    // exp.initialize();
    // exp.resume();
    exp
}

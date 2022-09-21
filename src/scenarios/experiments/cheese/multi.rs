use std::sync::Arc;

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

pub fn base_sim_settings() -> ExperimentSimSettingsBuilder {
    let mut sim_settings = ExperimentSimSettingsBuilder::default();
    sim_settings
        .num_simulation_ticks(50)
        .grid_size((30, 30))
        .num_genomes_per_sim(20)
        .default_unit_resources(vec![("cheese".to_owned(), 100)])
        .chemistry_key("cheese".to_string())
        .chemistry_configuration(
            ChemistryConfigBuilder::new()
                .set_float_amount("cheese_dispenser_odds", 0.20)
                .set_float_amount("milk_source_odds", 0.20)
                .set_resource_amount("max_milk_in_position", 800)
                .set_resource_amount("max_make_cheese_amount", 100)
                .build(),
        );

    sim_settings
}

pub fn base_gene_pool_settings(sim_settings: ExperimentSimSettings) -> GenePoolSettingsBuilder {
    let mut base_gene_pool = GenePoolSettingsBuilder::default();
    base_gene_pool
        // .sim_settings(known_settings.clone())
        .sim_settings(sim_settings)
        .receive_external_genomes(false)
        .num_genomes(20)
        .name_key("default".to_string())
        .alteration_specs(alterations())
        .fitness_calculation_key("total_cheese_acquired".to_string())
        .fitness_cycle_strategy(FitnessCycleStrategy::Exaustive {
            group_scramble_pct: 0.30,
        })
        .fitness_rank_adjustment_method(FitnessRankAdjustmentMethod::Absolute)
        // .fitness_rank_adjustment_method(FitnessRankAdjustmentMethod::Incremental {
        //     pct_jump: 0.50,
        //     min_jump: 3,
        // })
        .seed_genome_settings(SeedGenomeSettings::Random {
            min_size: 20,
            max_size: 200,
        })
        .cull_strategy(CullStrategy::WorstFirst { percent: 0.30 });
    // .cull_strategy(CullStrategy::RandomTiers {
    //     percent_per_tercile: [0.80, 0.20, 0.05],
    // });

    base_gene_pool
}

pub fn multi_pool_cheese_experiment_just_one(
    runner_args: ExperimentRunnerArgs,
) -> MultiPoolExperiment {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    let settings = MultiPoolExperimentSettingsBuilder::default()
        .max_iterations(1000)
        // .max_iterations(100)
        .chemistry_key("cheese".to_owned())
        .experiment_key(runner_args.experiment_name_key.clone())
        .logging_settings(MultiPoolExperimentLoggingSettings {
            experiment_key: runner_args.experiment_name_key.clone(),
            allow_overwrite: true,
            checkpoint_interval: 1000,
        })
        .evaluation_points_per_tick(5000)
        .reference_sim_settings(base_sim_settings().build())
        .reference_fitness_calculation_key("total_cheese_acquired".to_owned())
        .build();

    // let mut sim_settings = ExperimentSimSettingsBuilder::default();
    // sim_settings
    //     .num_simulation_ticks(100)
    //     .grid_size((30, 30))
    //     .num_genomes_per_sim(20)
    //     .default_unit_resources(vec![("cheese".to_owned(), 100)])
    //     .chemistry_key("cheese".to_string())
    //     .chemistry_configuration(
    //         ChemistryConfigBuilder::new()
    //             .set_float_amount("cheese_dispenser_odds", 0.20)
    //             .set_float_amount("milk_source_odds", 0.20)
    //             .set_resource_amount("max_milk_in_position", 800)
    //             .set_resource_amount("max_make_cheese_amount", 100)
    //             .build(),
    //     );

    let mut base_gene_pool = base_gene_pool_settings(base_sim_settings().build());
    let gene_pool_settings = vec![base_gene_pool.clone().build()];

    let mut exp = MultiPoolExperiment::new(settings, gene_pool_settings);
    exp.initialize();
    exp
}

pub fn multi_pool_cheese_experiment_vary_chemistry_config(
    runner_args: ExperimentRunnerArgs,
) -> MultiPoolExperiment {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    let settings = MultiPoolExperimentSettingsBuilder::default()
        .max_iterations(10000000)
        // .max_iterations(100)
        .chemistry_key("cheese".to_owned())
        .experiment_key(runner_args.experiment_name_key.clone())
        .logging_settings(MultiPoolExperimentLoggingSettings {
            experiment_key: runner_args.experiment_name_key.clone(),
            allow_overwrite: true,
            checkpoint_interval: 1000,
        })
        .evaluation_points_per_tick(5000)
        .reference_sim_settings(base_sim_settings().build())
        .reference_fitness_calculation_key("total_cheese_acquired".to_owned())
        .build();

    let mut sparse_sim_settings = base_sim_settings().clone();
    sparse_sim_settings.chemistry_configuration(
        ChemistryConfigBuilder::new()
            .set_float_amount("cheese_dispenser_odds", 0.30)
            .set_float_amount("milk_source_odds", 0.10)
            .set_resource_amount("max_milk_in_position", 1000)
            .set_resource_amount("max_make_cheese_amount", 100)
            .build(),
    );

    let mut rich_sim_settings = base_sim_settings().clone();
    rich_sim_settings.chemistry_configuration(
        ChemistryConfigBuilder::new()
            .set_float_amount("cheese_dispenser_odds", 0.70)
            .set_float_amount("milk_source_odds", 0.70)
            .set_resource_amount("max_milk_in_position", 100)
            .set_resource_amount("max_make_cheese_amount", 50)
            .build(),
    );

    let mut base_gene_pool = GenePoolSettingsBuilder::default();
    base_gene_pool
        // .sim_settings(known_settings.clone())
        .sim_settings(base_sim_settings().build())
        .receive_external_genomes(false)
        .num_genomes(20)
        .name_key("default".to_string())
        .alteration_specs(alterations())
        .fitness_calculation_key("total_cheese_acquired".to_string())
        .fitness_cycle_strategy(FitnessCycleStrategy::Exaustive {
            group_scramble_pct: 0.30,
        })
        .fitness_rank_adjustment_method(FitnessRankAdjustmentMethod::Absolute)
        .seed_genome_settings(SeedGenomeSettings::Random {
            min_size: 20,
            max_size: 100,
        })
        .cull_strategy(CullStrategy::WorstFirst { percent: 0.30 });

    // chemistry_configuration(ChemistryConfigBuilder::new().)
    let gene_pool_settings = vec![
        base_gene_pool.clone().build(),
        base_gene_pool
            .clone()
            .name_key("sparse_resources".to_string())
            .sim_settings(sparse_sim_settings.build())
            .build(),
        base_gene_pool
            .clone()
            .name_key("rich".to_string())
            .sim_settings(rich_sim_settings.build())
            .build(),
    ];

    let mut exp = MultiPoolExperiment::new(settings, gene_pool_settings);
    exp.initialize();
    exp
}
// GenePoolSettings {
//         sim_settings: ExperimentSimSettingsBuilder::default()
//             .num_simulation_ticks(100)
//             .grid_size((30, 30))
//             .num_genomes_per_sim(20)
//             .place_units_method(PlaceUnitsMethod::Default)
//             .chemistry_key("cheese".to_string())
//             .build(),
//         num_genomes: 20,
//         alteration_specs: default_alteration_set(),
//         fitness_calculation_key: "aoeu".to_string(),
//         fitness_cycle_strategy: FitnessCycleStrategy::Exaustive {
//             group_scramble_pct: 0.30,
//         },
//         name_key: todo!(),
//         fitness_rank_adjustment_method: todo!(),
//         seed_genome_settings: todo!(),
//         cull_strategy: todo!(),
//     }

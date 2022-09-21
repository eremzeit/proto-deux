use crate::{
    biology::experiments::{
        alterations,
        types::{CullStrategy, ExperimentSimSettings},
        variants::{
            multi_pool::types::FitnessCycleStrategy,
            simple::{
                logger::SimpleExperimentLoggingSettings, utils::SimpleExperimentSettings,
                SimpleExperiment,
            },
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
        cull_strategy: CullStrategy::WorstFirst { percent: 0.30 },
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
            chemistry_options: chemistry_builder,
        },

        iterations: 1,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: exp_key.clone(),
        logging_settings: Some(SimpleExperimentLoggingSettings {
            experiment_key: exp_key.clone(),
            allow_overwrite: true,
            checkpoint_interval: 1,
        }),
        fitness_cycle_strategy: FitnessCycleStrategy::Exaustive {
            group_scramble_pct: 0.30,
        },
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();

    exp.genome_entries[0].compiled_genome =
        Rc::new(FramedGenomeCompiler::compile(genome_vals1, &gm));
    exp.genome_entries[1].compiled_genome =
        Rc::new(FramedGenomeCompiler::compile(genome_vals2, &gm));
    exp.genome_entries[2].compiled_genome =
        Rc::new(FramedGenomeCompiler::compile(genome_vals3, &gm));
    exp.genome_entries[3].compiled_genome =
        Rc::new(FramedGenomeCompiler::compile(genome_vals4, &gm));

    // exp.genome_entries[0].compiled_genome.raw_values = genome_vals1.clone();
    // exp.genome_entries[1].compiled_genome.raw_values = genome_vals2.clone();
    // exp.genome_entries[2].compiled_genome.raw_values = genome_vals3.clone();
    // exp.genome_entries[3].compiled_genome.raw_values = genome_vals4.clone();

    exp.resume();
}

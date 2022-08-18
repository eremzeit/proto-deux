use crate::biology::experiments::variants::simple::utils::{
    CullStrategy, ExperimentSimSettings, SimpleExperimentSettings,
};
use crate::biology::experiments::variants::simple::SimpleExperiment;
use crate::biology::experiments::*;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::*;

pub fn evolve_lever() {
    let specs = SimulationSpecs {
        chemistry_key: "lever".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };

    let (cm, sm, gm) = specs.context();

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "lever_pulls".to_string(),
        num_genomes: 1000,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 1,
            grid_size: (10, 1),
            num_genomes_per_sim: 3,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
        },

        iterations: 10000,
        specs: specs,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: "my_experiment".to_string(),
        logging_settings: None,
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();

    exp.resume();
}

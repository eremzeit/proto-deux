use crate::biology::experiments::variants::simple::utils::{
    CullStrategy, ExperimentSimSettings, SimpleExperimentSettings,
};
use crate::biology::experiments::variants::simple::SimpleExperiment;
use crate::biology::experiments::*;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::*;

pub fn evolve_lever() {
    let chemistry_builder = ChemistryBuilder::with_key("lever");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::defaults(chemistry.get_manifest()).wrap_rc();

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
            place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        },

        iterations: 10000,
        alteration_set: alterations::default_alteration_set(),
        experiment_key: "my_experiment".to_string(),
        logging_settings: None,
        chemistry_options: chemistry_builder,
        gm,
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();

    exp.resume();
}

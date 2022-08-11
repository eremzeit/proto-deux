use crate::biology::experiments::*;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::*;

pub fn evolve_lever() {
    let chemistry_key = "lever".to_string();
    let chemistry = get_chemistry_by_key(
        &chemistry_key,
        PlaceUnitsMethod::SimpleDrop { attributes: None },
    );
    let gm = GeneticManifest::new();
    let cm = chemistry.get_manifest();
    let sm = SensorManifest::with_default_sensors(&cm);

    let settings = SimpleExperimentSettings {
        cull_strategy: CullStrategy::WorstFirst,
        fitness_calculation_key: "lever_pulls".to_string(),
        num_genomes: 1000,
        sim_settings: ExperimentSimSettings {
            num_simulation_ticks: 1,
            grid_size: (10, 1),
            num_genomes_per_sim: 3,
            iterations: 5,
            default_unit_resources: vec![],
            default_unit_attr: vec![],
        },

        iterations: 1,
        genetic_manifest: gm.clone(),
        sensor_manifest: sm.clone(),
        chemistry_key,
        alteration_set: alterations::default_alterations(),
    };

    let mut exp = SimpleExperiment::new(settings);
    exp.initialize();

    exp.resume();
}

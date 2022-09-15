pub mod cheese;
pub mod lever;

use crate::biology::unit_behavior::mouse::*;
use crate::runners::SimulationRunnerArgs;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use std::rc::Rc;

/*
 * modes:
 * -- simple simulation (no ui)
 * 	-- sim config key
 *  -- genome config keys
 * -- simple simulation (with ui)
 * 	-- sim config key
 *  -- genome config keys
 * 	-- ui config key
 * -- experiment (no ui)
 * 	-- experiment config key
 * 	-- genome config key
 * -- experiment (with ui)
 * 	-- experiment config key
 * 	-- genome config key
 * 	-- ui config key
*/

pub fn get_simulation_scenario(sim_args: &SimulationRunnerArgs) -> Simulation {
    let (chemistry_key, sim_scenario_key) = (
        sim_args.chemistry_key.as_str(),
        sim_args.simulation_scenario_key.as_str(),
    );

    let mut builder = match (chemistry_key, sim_scenario_key) {
        ("cheese", "basic") => cheese::basic(sim_args),
        ("cheese", "with_genomes") => cheese::with_genomes(sim_args),
        ("cheese", "with_genome2") => cheese::with_genome2(sim_args),
        ("lever", "basic") => lever::basic(sim_args),
        ("lever", "with_genome") => lever::with_genome(sim_args),
        _ => panic!("Unsupported simulation scenario"),
    };

    if let Some(iterations) = sim_args.iterations {
        builder = builder.iterations(sim_args.iterations.unwrap_or(10000));
    }

    builder.to_simulation()
}

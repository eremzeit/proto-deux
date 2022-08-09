use crate::biology::phenotype::mouse::*;
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

pub fn get_simulation_scenario(
    sim_scenario_key: &String,
    unit_scenario_key: Option<&String>,
) -> Simulation {
    match sim_scenario_key.as_str() {
        "basic_cheese" => basic_cheese(unit_scenario_key.clone()).to_simulation(),
        _ => panic!("Unsupported simulation scenario"),
    }
}

pub fn basic_cheese(unit_scenario_key: Option<&String>) -> SimulationBuilder {
    let chemistry_key = "cheese".to_string();
    SimulationBuilder::default()
        .chemistry_key(chemistry_key.to_string())
        .unit_entries(get_unit_entries_for_cheese())
        .iterations(100000)
        .size((50, 50))
        .unit_placement(PlaceUnitsMethod::ManualSingleEntry {
            attributes: None,
            coords: vec![(1, 1)],
        })
}

pub fn get_unit_entries_for_cheese() -> Vec<UnitEntryBuilder> {
    vec![UnitEntryBuilder::default()
        .species_name("main".to_string())
        .phenotype(Rc::new(Box::new(Mouse::construct())))
        .default_resources(vec![("cheese", 100)])]
}

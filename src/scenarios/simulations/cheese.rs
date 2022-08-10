use crate::biology::phenotype::mouse::*;
use crate::runners::SimulationRunnerArgs;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use std::rc::Rc;

pub fn basic(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let chemistry_key = "cheese".to_string();
    SimulationBuilder::default()
        .chemistry_key(chemistry_key.to_string())
        .unit_entries(get_unit_entries_for_cheese())
        .size((50, 50))
        .unit_placement(PlaceUnitsMethod::ManualSingleEntry {
            attributes: None,
            coords: vec![(10, 10)],
        })
        .iterations(1000)
}

pub fn get_unit_entries_for_cheese() -> Vec<UnitEntryBuilder> {
    vec![UnitEntryBuilder::default()
        .species_name("main".to_string())
        .phenotype(Rc::new(Box::new(Mouse::construct())))
        .default_resources(vec![("cheese", 200)])]
}

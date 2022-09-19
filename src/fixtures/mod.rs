use crate::chemistry::actions::ActionParam;
use crate::chemistry::variants::CheeseChemistry;
use crate::simulation::common::*;
use crate::simulation::config::SimulationConfig;
use crate::simulation::config::*;
use crate::simulation::executors::simple::SimpleSimulationExecutor;
use crate::simulation::unit::{UnitAttributeValue, UnitAttributes};
use crate::simulation::world::*;
use std::rc::Rc;
use std::sync::Arc;

use crate::biology::unit_behavior::mouse::*;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use crate::simulation::simulation_data::{
    new_threaded_simulation_reference, ThreadedSimulationReference,
};

use std::time::Duration;

//pub use crate::biology::genome::framed::{FramedGenome};
use crate::biology::genetic_manifest::predicates::default_operators;
use crate::biology::genetic_manifest::GeneticManifestData;

use crate::simulation::common::{
    builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
};

use super::*;

pub fn default_base_with_unit_placement(place_units_method: PlaceUnitsMethod) -> Simulation {
    let chemistry = ChemistryBuilder::with_key("cheese").build();

    let mut sim = SimulationBuilder::default()
        .chemistry(chemistry)
        .place_units_method(place_units_method)
        .unit_manifest(UnitManifest::from(&vec![UnitEntry::new(
            "main",
            NullBehavior::construct(),
        )]))
        .size((5, 5))
        .to_simulation();

    sim.init();
    // cheese
    sim.world.set_unit_resource_at(&(1, 0), 0, 20);
    sim.world.set_unit_resource_at(&(1, 0), 0, 10);
    sim.world.set_unit_resource_at(&(3, 0), 0, 5);
    sim
}

pub fn default_base() -> Simulation {
    default_base_with_unit_placement(PlaceUnitsMethod::LinearBottomMiddle { attributes: None })
}

pub mod experiments;
pub mod fps;
pub mod perf;

use crate::chemistry::actions::{ActionDefinition, ActionParam};
use crate::chemistry::variants::{BaseChemistry, CheeseChemistry};
use crate::simulation::common::*;
use crate::simulation::config::SimulationConfig;
use crate::simulation::config::*;
use crate::simulation::executors::simple::SimpleSimulationExecutor;
use crate::simulation::unit::{UnitAttributeValue, UnitAttributes};
use crate::simulation::world::*;
use std::rc::Rc;
use std::sync::Arc;
use typemap::{CloneMap, Key};

use crate::biology::unit_behavior::mouse::*;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use crate::simulation::simulation_data::{
    new_threaded_simulation_reference, ThreadedSimulationReference,
};

use std::time::Duration;

//pub use crate::biology::genome::framed::{FramedGenome};
pub use crate::biology::genetic_manifest::predicates::default_operators;
pub use crate::biology::genetic_manifest::GeneticManifest;

pub fn test_framed_genome() {
    let gm = GeneticManifest::new();
}

pub fn test_framed_genome2() {}

pub mod fixtures {
    use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;

    use super::*;

    pub fn default_base_with_unit_placement(place_units_method: PlaceUnitsMethod) -> Simulation {
        let specs = SimulationSpecs {
            chemistry_key: "cheese".to_string(),
            place_units_method: place_units_method,
            ..Default::default()
        };

        let mut sim = SimulationBuilder::default()
            .specs(specs)
            .unit_manifest(UnitManifest::from(&vec![UnitEntry::new(
                "main",
                NullBehavior::construct(),
            )]))
            .headless(true)
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
}

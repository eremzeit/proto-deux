pub mod experiments;
pub mod fps;
pub mod perf;

use chemistry::actions::{ActionDefinition, ActionParam};
use chemistry::{BaseChemistry, CheeseChemistry, Chemistry};
use simulation::common::*;
use simulation::config::SimulationConfig;
use simulation::config::*;
use simulation::executors::simple::SimpleSimulationExecutor;
use simulation::specs::place_units::*;
use simulation::specs::SimulationSpec;
use simulation::unit::{UnitAttributeValue, UnitAttributes};
use simulation::world::*;
use std::rc::Rc;
use std::sync::Arc;
use typemap::{CloneMap, Key};

use biology::phenotype::mouse::*;
use simulation::common::*;
use simulation::config::*;
use simulation::executors::threaded::ThreadedSimulationExecutor;
use simulation::simulation_data::{new_threaded_simulation_reference, ThreadedSimulationReference};

use std::time::Duration;

//pub use biology::genome::framed::{FramedGenome};
pub use biology::genetic_manifest::predicates::default_operators;
pub use biology::genetic_manifest::GeneticManifest;

pub fn test_framed_genome() {
    let gm = GeneticManifest::new();
}

pub fn test_framed_genome2() {}

pub mod fixtures {
    use super::*;

    pub fn default_base(_specs: Option<Vec<Box<dyn SimulationSpec>>>) -> Simulation {
        let specs: Vec<Box<dyn SimulationSpec>> = match _specs {
            None => vec![Box::new(PlaceUnits {
                method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
            })],

            Some(s) => s,
        };

        let mut sim = SimulationBuilder::default()
            .unit_manifest(UnitManifest::from(&vec![UnitEntry::new(
                "main",
                EmptyPhenotype::construct(),
            )]))
            .headless(true)
            .size((5, 5))
            .chemistry_key("cheese".to_string())
            .specs(specs)
            .to_simulation();

        sim.init();
        // cheese
        sim.world.set_unit_resource_at(&(1, 0), 0, 20);
        sim.world.set_unit_resource_at(&(1, 0), 0, 10);
        sim.world.set_unit_resource_at(&(3, 0), 0, 5);
        sim
    }
}

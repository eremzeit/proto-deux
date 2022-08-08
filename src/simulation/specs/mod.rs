pub mod phenotype_execution;
pub mod place_units;
pub mod resource_allocation;
pub mod resource_transits;

use crate::simulation::common::{GridSize2D, Simulation, SimulationAttributes, UnitManifest};
use std::rc::Rc;
use std::sync::Arc;

use crate::chemistry::{Chemistry, ChemistryInstance};
use crate::simulation::common::SimCell;
use crate::simulation::config::SimulationConfig;
use crate::simulation::world::World;

use typemap::{CloneMap, Key};

pub use self::place_units::PlaceUnits;
pub use self::resource_allocation::ResourceAllocation;

pub type SimulationSpecs = Vec<Box<dyn SimulationSpec>>;

pub type SpecState = CloneMap;

// pub struct SimSpecs {
//     place_units: Option<Box<PlaceUnits>>,
//     //calculate_fitness: Option<Box<FitnessCalculation>>,
//     specs: Vec<Box<dyn SimulationSpec>>
// }

#[derive(Clone)]
struct ExampleSpec;
impl Key for ExampleSpec {
    type Value = ExampleSpec;
}

pub struct SpecContext {}

pub trait SimulationSpec {
    fn on_tick(&mut self, sim: &mut SimCell, context: &SpecContext) {
        //let my_spec_state = sim.world.spec_state.get_mut::<ExampleSpec>();
        // do stuff..
    }

    fn on_init(&mut self, sim: &mut SimCell, context: &SpecContext) {}

    fn on_end(&mut self, sim: &mut SimCell, context: &SpecContext) {}

    fn get_name(&self) -> String;
}

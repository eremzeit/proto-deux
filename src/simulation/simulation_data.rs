//use simulation::config::*;
// use simulation::position::*;
// use simulation::unit::*;
use simulation::world::*;
use simulation::config::*;
use simulation::unit::{UnitAttributes, UnitResources};

use util::Coord;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;

use std::sync::atomic::{*};

pub type ThreadedSimulationReference = Arc<Mutex<RefCell<Option<SimulationData>>>>;

pub fn new_threaded_simulation_reference() -> ThreadedSimulationReference {
    Arc::new(std::sync::Mutex::new(std::cell::RefCell::new(None)))
}

#[derive(Clone)]
pub struct SimulationData {
    pub grid: Grid,
    pub config: SimulationConfigData,
    pub tick: u64,
}


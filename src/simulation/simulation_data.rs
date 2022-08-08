//use crate::simulation::config::*;
// use crate::simulation::position::*;
// use crate::simulation::unit::*;
use crate::simulation::config::*;
use crate::simulation::unit::{UnitAttributes, UnitResources};
use crate::simulation::world::*;

use crate::util::Coord;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use std::sync::atomic::*;

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

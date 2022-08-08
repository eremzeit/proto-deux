use std::time::{Duration, Instant};
// use std::sync::atomic::{*};
// use std::sync::{Arc, Mutex};
// use std::cell::RefCell;
// use std::sync::mpsc::{channel, Receiver};
//
// use crate::chemistry::{BaseChemistry, CheeseChemistry};
// use crate::simulation::{*};
use crate::simulation::Simulation;
//use crate::simulation::simulation_data::{SimulationData, ThreadedSimulationReference};

pub struct SimpleSimulationExecutor {
    pub is_paused: bool,
    pub simulation: Simulation,
    pub sample_update_instant: Instant,
    pub sample_update_tick: u64,
}

//std::process::exit(0);
impl SimpleSimulationExecutor {
    pub fn new(simulation: Simulation) -> SimpleSimulationExecutor {
        SimpleSimulationExecutor {
            is_paused: true,
            simulation,
            sample_update_instant: Instant::now(),
            sample_update_tick: 0,
        }
    }

    pub fn start(&mut self) {
        self.is_paused = false;
        let mut target_delay = Duration::new(7, 0);
        while self.simulation.world.tick < self.simulation.iterations {
            self.simulation.tick();
            let sample_duration = Instant::now().duration_since(self.sample_update_instant);
            let should_update_console = sample_duration > target_delay;

            if should_update_console {
                let sample_ticks = self.simulation.world.tick - self.sample_update_tick;
                let rate = (sample_ticks as f64 / sample_duration.as_millis() as f64) * 1000.0f64;
                let per_unit = rate * (self.simulation.world._unit_count as f64);
                //println!("[Simulation] {:.2}tps", rate);
                println!("[Simulation] {:.2}tps -- {:.2}units/sec", rate, per_unit);

                self.sample_update_instant = Instant::now();
                self.sample_update_tick = self.simulation.world.tick;
            }
        }
    }
}

use std::cell::RefCell;
use std::sync::atomic::*;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::chemistry::{BaseChemistry, CheeseChemistry};
use crate::simulation::common::SimulationControlEventReceiver;
use crate::simulation::simulation_data::{SimulationData, ThreadedSimulationReference};
use crate::simulation::Simulation;
use crate::simulation::*;
use crate::util::RateCounter;

pub struct ThreadedSimulationExecutor {
    pub is_paused: bool,
    pub is_finished: bool,
    pub max_ticks_per_second: u32,
    pub max_view_updates_per_second: u32,
    pub simulation: Simulation,

    pub control_events_receiver: SimulationControlEventReceiver,
    pub last_view_update: Instant,
    pub last_tick: Instant,
    pub double: ThreadedSimulationReference,
}

impl ThreadedSimulationExecutor {
    pub fn new(
        mut simulation: Simulation,
        double: ThreadedSimulationReference,
        control_events: SimulationControlEventReceiver,
        max_ticks_per_second: u32,
        max_view_updates_per_second: u32,
    ) -> ThreadedSimulationExecutor {
        ThreadedSimulationExecutor {
            is_paused: true,
            is_finished: false,
            max_view_updates_per_second: max_ticks_per_second,
            max_ticks_per_second: max_view_updates_per_second,
            control_events_receiver: control_events,

            simulation,
            last_tick: Instant::now(),
            last_view_update: Instant::now(),
            double,
        }
    }
    pub fn start(&mut self) {
        self.is_paused = false;
        self.run();
    }
    pub fn handle_control_events(&mut self) {
        for event in self.control_events_receiver.try_iter() {
            println!("Received {:?}", event);
            match &event {
                SimulationControlEvent::Resume => {
                    self.is_paused = false;
                }
                _ => {}
            }
        }
    }

    pub fn run_loop(&mut self) {
        let mut should_break = false;

        while !(self.is_paused || self.is_finished || should_break) {
            let mut counter = RateCounter::new();

            std::thread::sleep(Duration::new(0, 200_000_000));

            let mut has_initialized = false;

            let mut target_tick_delay =
                Duration::new(0, (10u32).pow(9) / self.max_ticks_per_second);
            let mut target_view_delay =
                Duration::new(0, (10u32).pow(9) / self.max_view_updates_per_second);

            // TODO: this should be rewritten
            loop {
                let observed_delay = Instant::now().duration_since(self.last_tick);
                let should_tick = observed_delay > target_tick_delay;
                let mut divisor = if self.max_ticks_per_second < 5 { 2 } else { 3 };
                if self.max_ticks_per_second > 100 {
                    divisor = 10;
                }

                let should_update_view = Instant::now().duration_since(self.last_view_update)
                    > (target_view_delay / divisor);
                //println!("target: {:?}", &target_tick_delay);
                //let divisor = 2;

                // TODO: experiment with this
                std::thread::sleep(target_tick_delay / divisor);

                if should_tick {
                    counter.inc_and_update();
                    self.simulation.tick();
                    println!("tick");
                    self.last_tick = Instant::now();
                    has_initialized = true;

                    // // aoeu - might need to rig to update less often than the ticks at some point.  otherwise
                    // // if the tick rate is high then we'd be updating too often.
                    // let locked = self.double.lock().unwrap();
                    // locked.replace(Some(self.simulation.to_data()));
                    // self.last_view_update = Instant::now();
                }

                if should_update_view || should_tick {
                    let locked = self.double.lock().unwrap();
                    locked.replace(Some(self.simulation.to_data()));
                    self.last_view_update = Instant::now();
                }

                should_break = self.simulation.world.tick >= self.simulation.iterations
                    || self.is_finished
                    || self.is_paused;
                if should_break {
                    break;
                }

                self.handle_control_events();
            }
        }
    }

    pub fn wait_loop(&mut self) {
        while self.is_paused {
            self.handle_control_events();
            std::thread::sleep(Duration::new(0, 20_000_000));
        }
    }

    pub fn run(&mut self) {
        {
            let locked = self.double.lock().unwrap();
            locked.replace(Some(self.simulation.to_data()));
        }

        loop {
            self.wait_loop();
            self.run_loop();
            std::thread::sleep(Duration::new(0, 2_000_000));
        }
    }
}

pub fn test() {
    use std::thread;
    // use ui::widgets::simulation::{SimulationUi};

    // let (tx, rx) = channel::<String>();
    //
    // let cell = Arc::new(Mutex::new(RefCell::new(None)));
    // let cell2 = cell.clone();
    // let mut simulation = SimulationUi::new(cell);
    //
    // let handle = std::thread::spawn(move || {
    //     let _tx = tx; // force a move

    //     let config = SimulationConfig::new((5, 5), None);
    //     let sim = Simulation::new(config, vec![], BaseChemistry::default());
    //     let mut executor = ThreadedSimulationExecutor::new(sim, cell2);
    //     executor.start();
    // });

    // std::thread::sleep(Duration::new(5, 0));
    //
    // simulation.start_ui();
    //std::process::exit(0);
}

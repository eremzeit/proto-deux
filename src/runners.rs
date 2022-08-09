use crate::{
    scenarios::simulations::get_simulation_scenario,
    simulation::{
        executors::threaded::ThreadedSimulationExecutor,
        simulation_data::new_threaded_simulation_reference, SimulationControlEvent,
    },
};

use crate::ui;

pub struct SimulationRunnerArgs {
    pub simulation_scenario_key: String,
    pub unit_entry_scenario_key: Option<String>,
}
pub struct ExperimentRunnerArgs {
    pub experiment_scenario_key: String,
    pub experiment_name: String,
}

pub enum RunMode {
    HeadlessSimulation(SimulationRunnerArgs),
    GuiSimulation(SimulationRunnerArgs),
    HeadlessExperiment(ExperimentRunnerArgs),
    GuiExperiment(ExperimentRunnerArgs),
}

pub fn start_headless_sim(sim_scenario_key: String, unit_scenario_key: Option<String>) {}

pub fn start_sim_with_gui(sim_scenario_key: String, unit_scenario_key: Option<String>) {
    let (sender_from_ui, receiver_from_ui) = std::sync::mpsc::channel::<SimulationControlEvent>();
    let sim_ref = new_threaded_simulation_reference();
    let sim_ref2 = sim_ref.clone();
    let _sim_scenario_key = sim_scenario_key.to_string();
    let _unit_scenario_key = unit_scenario_key.clone();

    let handle = std::thread::spawn(move || {
        let mut sim = get_simulation_scenario(&_sim_scenario_key, _unit_scenario_key.as_ref());

        let mut executor = ThreadedSimulationExecutor::new(sim, sim_ref2, receiver_from_ui);
        executor.is_paused = true;
        executor.max_view_updates_per_second = 100;
        executor.max_ticks_per_second = 100;
        executor.run();
    });

    let s = sender_from_ui.clone();
    //sender_from_ui.send(SimulationControlEvent::Resume);
    ui::event_loop::start_ui_loop(sim_ref, sender_from_ui);
}

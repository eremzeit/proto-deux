pub mod exp_replay;

use crate::{
    scenarios::{experiments::get_experiment_scenario, simulations::get_simulation_scenario},
    simulation::{
        executors::{simple::SimpleSimulationExecutor, threaded::ThreadedSimulationExecutor},
        simulation_data::{new_threaded_simulation_reference, SimulationData},
        SimulationControlEvent,
    },
    ui::event_loop::{UiConfig, WorldRenderConfig},
};

use crate::ui;

use self::exp_replay::{construct_replay_sim, load_exp_settings};

#[derive(Clone)]
pub struct SimulationRunnerArgs {
    pub chemistry_key: String,
    pub simulation_scenario_key: String,
    pub unit_entry_scenario_key: Option<String>,
    pub iterations: Option<u64>,
}
#[derive(Clone)]
pub struct SimulationUiRunnerArgs {
    pub max_view_updates_per_second: Option<u32>,
    pub max_ticks_per_second: Option<u32>,
}

#[derive(Clone)]
pub struct ExperimentRunnerArgs {
    pub experiment_scenario_key: String,
    pub experiment_name_key: String,
}

#[derive(Clone)]
pub struct ExperimentSimReplayGuiArgs {
    pub experiment_name_key: String,
    pub genome_filename: String,
}

pub enum RunMode {
    HeadlessSimulation(SimulationRunnerArgs),
    GuiSimulation(SimulationRunnerArgs, SimulationUiRunnerArgs),
    ExperimentSimReplayGui(ExperimentSimReplayGuiArgs, SimulationUiRunnerArgs),
    HeadlessExperiment(ExperimentRunnerArgs),
    GuiExperiment(ExperimentRunnerArgs),
    OneOff(String),
}

// use crate::scenarios::experiments::get_experiment_scenario;

pub fn start_headless_experiment(exp_runner_args: ExperimentRunnerArgs) {
    let mut exp = get_experiment_scenario(exp_runner_args);

    println!("Starting headless experiment");
    exp.start();
}

pub fn start_headless_sim(sim_runner_args: SimulationRunnerArgs) {
    let mut sim = get_simulation_scenario(&sim_runner_args);

    println!("Starting headless simulation");
    let mut executor = SimpleSimulationExecutor::new(sim);
    executor.start();
}

pub fn start_sim_with_gui(
    sim_runner_args: SimulationRunnerArgs,
    sim_ui_runner_args: SimulationUiRunnerArgs,
) {
    let (sender_from_ui, receiver_from_ui) = std::sync::mpsc::channel::<SimulationControlEvent>();
    let sim_ref = new_threaded_simulation_reference();
    let sim_ref2 = sim_ref.clone();

    let view_updates_per_second = sim_ui_runner_args.max_view_updates_per_second.unwrap_or(1);

    let _sim_runner_args = sim_runner_args.clone();
    let handle = std::thread::spawn(move || {
        let mut sim = get_simulation_scenario(&_sim_runner_args.clone());
        let mut executor = ThreadedSimulationExecutor::new(
            sim,
            sim_ref2,
            receiver_from_ui,
            sim_ui_runner_args.max_ticks_per_second.unwrap_or(1),
            view_updates_per_second,
        );

        executor.is_paused = true;
        executor.run();
    });

    let s = sender_from_ui.clone();
    //sender_from_ui.send(SimulationControlEvent::Resume);
    ui::event_loop::start_sim_ui(
        UiConfig { window_size: None },
        sim_ref,
        sender_from_ui,
        WorldRenderConfig {
            chemistry_key: sim_runner_args.chemistry_key.clone(),
            renders_per_second: view_updates_per_second,
            cell_size: 20.0,
        },
    );
}

pub fn start_exp_replay_with_ui(
    exp_replay_args: ExperimentSimReplayGuiArgs,
    sim_ui_runner_args: SimulationUiRunnerArgs,
) {
    let settings = load_exp_settings(&exp_replay_args.experiment_name_key);

    let (sender_from_ui, receiver_from_ui) = std::sync::mpsc::channel::<SimulationControlEvent>();
    let sim_ref = new_threaded_simulation_reference();
    let sim_ref2 = sim_ref.clone();

    let view_updates_per_second = sim_ui_runner_args.max_view_updates_per_second.unwrap_or(1);

    let _sim_runner_args = sim_ui_runner_args.clone();
    let handle = std::thread::spawn(move || {
        let mut sim = construct_replay_sim(
            &exp_replay_args.experiment_name_key,
            &exp_replay_args.genome_filename,
        )
        .to_simulation();
        let mut executor = ThreadedSimulationExecutor::new(
            sim,
            sim_ref2,
            receiver_from_ui,
            sim_ui_runner_args.max_ticks_per_second.unwrap_or(1),
            view_updates_per_second,
        );

        executor.is_paused = true;
        executor.run();
    });

    let s = sender_from_ui.clone();
    //sender_from_ui.send(SimulationControlEvent::Resume);
    ui::event_loop::start_sim_ui(
        UiConfig { window_size: None },
        sim_ref,
        sender_from_ui,
        WorldRenderConfig {
            chemistry_key: settings.chemistry_options.chemistry_key.clone(),
            renders_per_second: view_updates_per_second,
            cell_size: 30.0,
        },
    );
}

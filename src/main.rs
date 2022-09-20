#![feature(let_chains)]
#![feature(trace_macros)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(where_clauses_object_safety)]
#![allow(unused_macros)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]

use crate::{runners::RunMode, scenarios::one_offs::run_one_off};

#[macro_use]
extern crate derive_builder;

// #[macro_use]
// extern crate serde_json;
extern crate chrono;
extern crate clap;
extern crate find_folder;
extern crate fps_counter;
extern crate image as im;
extern crate ndarray;
extern crate once_cell;
extern crate opengl_graphics;
extern crate pad;
extern crate palette;
extern crate piston_window;
extern crate rand;
extern crate ron;

#[macro_use]
pub mod perf;

#[macro_use]
pub mod util;
pub mod chemistry;
pub mod simulation;
#[macro_use]
pub mod biology;
pub mod fixtures;
pub mod runners;
pub mod scenarios;
pub mod tests;
pub mod ui;

use std::collections::HashMap;

fn main() {
    perf_timer_start!("main");
    let args = util::cli::parse_cli_args();
    match args {
        RunMode::ExperimentSimReplayGui(exp_args, sim_ui_args) => {
            runners::start_exp_replay_with_ui(exp_args, sim_ui_args);
        }
        RunMode::HeadlessExperiment(args) => {
            runners::start_headless_experiment(args);
        }
        RunMode::HeadlessSimulation(sim_args) => {
            runners::start_headless_sim(sim_args);
        }
        RunMode::GuiSimulation(sim_args, gui_args) => {
            runners::start_sim_with_gui(sim_args, gui_args);
        }
        RunMode::OneOff(scenario_key) => {
            run_one_off(&scenario_key);
        }
        RunMode::MultiPoolExperiment(args) => {
            runners::run_multi_pool_experiment(args);
        }
        _ => panic!("Run mode not implemented yet"),
    }

    perf_timer_stop!("main");
    perf_timers_print!();
}

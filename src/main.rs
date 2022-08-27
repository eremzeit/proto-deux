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
#![feature(trace_macros)]

#[macro_use]
extern crate derive_builder;

// #[macro_use]
// extern crate serde_json;

extern crate chrono;
extern crate clap;
extern crate find_folder;
extern crate ndarray;
extern crate pad;
extern crate palette;
extern crate petgraph;
extern crate rand;
extern crate roaring;
extern crate typemap;

extern crate fps_counter;
extern crate image as im;
extern crate once_cell;
extern crate opengl_graphics;
extern crate piston_window;

#[macro_use]
pub mod perf;

#[macro_use]
pub mod util;
pub mod chemistry;
pub mod simulation;
#[macro_use]
pub mod biology;
pub mod runners;
pub mod scenarios;
pub mod tests;
pub mod ui;

use common::ThreadedSimulationExecutor;
use ndarray::*;
use ndarray::{Array2, Dim, Shape};
use perf::{perf_timer_print, perf_timer_start, perf_timer_stop};
use runners::RunMode;
use scenarios::one_offs::run_one_off;
use simulation::simulation_data::new_threaded_simulation_reference;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::biology::*;
use crate::scenarios::simulations::get_simulation_scenario;
use crate::simulation::*;

fn main() {
    //tests::test_framed_genome();
    // ui::start_app_with_genome();
    // ui::execute::start_app();

    //tests::fps::test_with_genome();
    //tests::perf::test_multithreading2();
    perf_timer_start!("main");
    let args = util::cli::parse_cli_args();
    match args {
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
        _ => panic!("Run mode not implemented yet"),
    }

    perf_timer_stop!("main");
    perf_timers_print!();
}

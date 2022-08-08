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

//#![feature(trace_macros)]

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate serde_json;

extern crate chrono;
extern crate find_folder;
extern crate ndarray;
extern crate pad;
extern crate palette;
extern crate petgraph;
extern crate rand;
extern crate roaring;
extern crate typemap;

extern crate graphics as piston_graphics;
extern crate input as piston_input;

extern crate gfx_device_gl;

#[macro_use]
extern crate conrod_core;

extern crate piston_window;
#[macro_use]
extern crate conrod_derive;

#[macro_use]
pub mod util;

pub mod chemistry;

pub mod simulation;
#[macro_use]
pub mod biology;

pub mod launches;
pub mod tests;

use ndarray::*;
use ndarray::{Array2, Dim, Shape};
use serde_json::{Result, Value};
use std::collections::HashMap;

use biology::*;
use simulation::*;

fn main() {
    //ui::main();
    //ui::window::main();
    //ui::simulation::foo::main();

    // tests rendering the widget structure for simulations, but doesn't yet render real time simulations
    //ui::widgets::simulation::start_ui();

    //tests::test_framed_genome();
    //ui::start_app_with_genome();

    //tests::fps::test_with_genome();
    //tests::perf::test_multithreading2();
    //tests::experiments::evolve_lever();
}

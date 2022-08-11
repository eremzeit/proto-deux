use crate::chemistry::actions::{ActionDefinition, ActionParam};
use crate::chemistry::variants::{BaseChemistry, CheeseChemistry};
use crate::simulation::common::*;
use crate::simulation::config::SimulationConfig;
use crate::simulation::config::*;
use crate::simulation::executors::simple::SimpleSimulationExecutor;
use crate::simulation::unit::{UnitAttributeValue, UnitAttributes};
use crate::simulation::world::*;
use ndarray::parallel::prelude::IntoParallelIterator;
use ndarray::parallel::prelude::*;
use ndarray::*;
use ndarray::{Array, Array2, Dim, Ix, Shape};
use rand::Rng;
use std::rc::Rc;
use std::sync::Arc;

use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct FakeItem {
    i: u64,
    stuff: Vec<i64>,
}
impl FakeItem {
    pub fn new() -> Self {
        Self {
            i: 0,
            stuff: vec![-24; 1],
        }
    }
    pub fn do_foo(&mut self) {
        let mut rand = rand::thread_rng();

        for i in (0..1000) {
            self.i = rand.gen_range(0..100);
            self.i = self.i + 1;
        }
    }
}

pub type WithOption = Array2<Option<FakeItem>>;
pub type WithoutOption = Array2<FakeItem>;

pub fn test_grid_access_times() {
    let x_max = 100;
    let y_max = 100;
    let mut grid1: WithOption = Array2::from_elem((100, 100).f(), Some(FakeItem::new()));
    let mut grid2: WithoutOption = Array2::from_elem((100, 100).f(), FakeItem::new());
    let ITERATIONS: u64 = 10000000;
    use rand::Rng;
    let mut rand = rand::thread_rng();

    let test1_pre = Instant::now();
    for i in (0..ITERATIONS) {
        let mut item = grid2
            .get_mut([(i % x_max) as usize, (i % y_max) as usize])
            .unwrap();
        item.do_foo();

        //let aoeu = rand.gen_range(0..x_max);
        //let aoeu2 = rand.gen_range(0..x_max);
        //let aoeu3 = rand.gen_range(0..x_max);
    }
    let test1_post = Instant::now();
    for i in (0..ITERATIONS) {
        let mut maybe_item = grid1
            .get_mut([(i % x_max) as usize, (i % y_max) as usize])
            .unwrap();
        if let Some(item) = maybe_item {
            item.do_foo();
        }
    }

    let test2_post = Instant::now();
    unsafe {
        for i in (0..ITERATIONS) {
            let mut maybe_item = grid1.uget_mut([(i % x_max) as usize, (i % y_max) as usize]);
            if let Some(item) = maybe_item {
                item.do_foo();
            }
        }
    }

    let test3_post = Instant::now();
    unsafe {
        for i in (0..ITERATIONS) {
            let mut item = grid2.uget_mut([(i % x_max) as usize, (i % y_max) as usize]);
            item.do_foo();
        }
    }
    let test4_post = Instant::now();

    let time1 = test1_post.duration_since(test1_pre);
    let time2 = test2_post.duration_since(test1_post);
    let time3 = test3_post.duration_since(test2_post);
    let time4 = test4_post.duration_since(test3_post);
    println!(
        "time1: {} -- time2: {} -- time3: {} -- time4: {}",
        time1.as_millis(),
        time2.as_millis(),
        time3.as_millis(),
        time4.as_millis(),
    );
}

pub fn test_multithreading() {
    use ndarray::ShapeBuilder;

    let x_max = 1000;
    let y_max = 1000;
    let mut grid1: WithoutOption = Array2::from_elem((x_max, y_max).f(), FakeItem::new());

    let test1_pre = Instant::now();
    grid1.map_inplace(|x| x.do_foo());

    let test1_post = Instant::now();
    grid1.par_map_inplace(|x| x.do_foo());
    let test2_post = Instant::now();

    let time1 = test1_post.duration_since(test1_pre);
    let time2 = test2_post.duration_since(test1_post);
    println!(
        "time1: {} -- time2: {}",
        time1.as_millis(),
        time2.as_millis(),
    );
}

pub fn test_multithreading2() {
    use ndarray::ShapeBuilder;

    let x_max = 100;
    let y_max = 10;
    let mut grid: WithoutOption = Array2::from_elem((x_max, y_max).f(), FakeItem::new());

    let test1_pre = Instant::now();
    for y in (0..y_max) {
        for x in (0..x_max) {
            let mut item = grid.get_mut((x, y)).unwrap();
            item.do_foo();
        }
    }
    let test1_post = Instant::now();

    let chunk_size = 10;
    grid.axis_chunks_iter_mut(Axis(0), chunk_size)
        .into_par_iter()
        .for_each(|mut chunk| {
            for y in (0..y_max) {
                for x in (0..chunk_size) {
                    let mut item = chunk.get_mut((x, y)).unwrap();
                    item.do_foo();
                }
            }
        });

    let test2_post = Instant::now();
    grid.map_inplace(|x| x.do_foo());
    let test3_post = Instant::now();

    grid.par_map_inplace(|x| x.do_foo());
    let test4_post = Instant::now();

    let time1 = test1_post.duration_since(test1_pre);
    let time2 = test2_post.duration_since(test1_post);
    let time3 = test3_post.duration_since(test2_post);
    let time4 = test3_post.duration_since(test3_post);
    println!(
        "time1: {} -- time2: {} -- time3: {} -- time4: {}",
        time1.as_millis(),
        time2.as_millis(),
        time3.as_millis(),
        time4.as_millis(),
    );
}

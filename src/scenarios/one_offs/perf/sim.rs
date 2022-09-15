use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::biology::genome::framed::annotated::FramedGenomeExecutionStats;
use crate::biology::genome::framed::builders::{simple_convert_into_frames, FramedGenomeCompiler};
use crate::biology::genome::framed::render::with_stats::render_frames_with_stats;
use crate::biology::genome::framed::samples::cheese::get_genome2;
use crate::biology::unit_behavior::framed::FramedGenomeUnitBehavior;
use crate::runners::SimulationRunnerArgs;

use crate::chemistry::builder::*;
use crate::scenarios::simulations::lever::get_unit_entries_for_lever;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::{
    ChemistryInstance, CoordIterator, GeneticManifest, GeneticManifestData, UnitManifest,
};
use crate::simulation::config::SimulationBuilder;
use crate::simulation::executors::simple::SimpleSimulationExecutor;
use crate::simulation::Simulation;
use crate::unit_entry::builder::UnitEntryBuilder;

pub fn test_sim_perf() {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let chemistry = chemistry_builder.build();
    let gm = GeneticManifest::from_chemistry(&chemistry).wrap_rc();

    use crate::biology::genome::framed::samples::cheese::get_genome1;
    let genome1 = get_genome2(&gm);
    let genome2 = get_genome2(&gm);
    let genome3 = get_genome2(&gm);

    let genome1_stats = Rc::new(RefCell::new(genome1.new_stats()));
    let genome2_stats = Rc::new(RefCell::new(genome2.new_stats()));
    let genome3_stats = Rc::new(RefCell::new(genome3.new_stats()));

    let start_time = Instant::now();

    for i in 0..1000 {
        // for j in 0..3 {
        //     let g1_clone = genome1.clone();
        // }

        let entry1 = UnitEntryBuilder::default()
            .species_name("species1".to_string())
            .behavior(
                FramedGenomeUnitBehavior::new_with_stats(
                    genome1.clone(),
                    gm.clone(),
                    genome1_stats.clone(),
                )
                .construct(),
            )
            .default_resources(vec![("cheese".to_string(), 100)])
            .build(&chemistry_builder.manifest());

        let entry2 = UnitEntryBuilder::default()
            .species_name("species2".to_string())
            .behavior(
                FramedGenomeUnitBehavior::new_with_stats(
                    genome2.clone(),
                    gm.clone(),
                    genome2_stats.clone(),
                )
                .construct(),
            )
            .default_resources(vec![("cheese".to_owned(), 100)])
            .build(&chemistry_builder.manifest());

        let entry3 = UnitEntryBuilder::default()
            .species_name("species3".to_string())
            .behavior(
                FramedGenomeUnitBehavior::new_with_stats(
                    genome3.clone(),
                    gm.clone(),
                    genome3_stats.clone(),
                )
                .construct(),
            )
            .default_resources(vec![("cheese".to_owned(), 100)])
            .build(&chemistry_builder.manifest());

        let sim = SimulationBuilder::default()
            .chemistry(chemistry_builder.build())
            .size((20, 20))
            .iterations(1000)
            .unit_manifest(UnitManifest {
                units: vec![entry1, entry2, entry3],
            })
            .to_simulation();

        let mut executor = SimpleSimulationExecutor::new(sim);
        executor.start();

        if i % 10 == 0 {
            println!("iteration: {}", i);
        }

        if i == 999 {
            println!(
                "{}",
                render_frames_with_stats(&genome1.frames, &gm, Some(&genome1_stats.borrow()))
            );
        }
    }

    let duration = Instant::now().duration_since(start_time);
    println!("Time: {}ms", duration.as_millis());
}

pub fn test_resource_allocation_method_perf() {
    let iterations = 100;
    perf_timer_start!("allocation1");
    resource_allocation1(iterations);
    perf_timer_stop!("allocation1");
    perf_timer_start!("allocation2");
    resource_allocation2(iterations);
    perf_timer_stop!("allocation2");

    perf_timer_start!("allocation3");
    resource_allocation3(iterations);
    perf_timer_stop!("allocation3");
}

pub fn _sim(chemistry: ChemistryInstance) -> Simulation {
    let mut sim = SimulationBuilder::default()
        .chemistry(chemistry)
        .unit_entries(get_unit_entries_for_lever())
        .place_units_method(PlaceUnitsMethod::SimpleDropMultiple {
            attributes: None,
            units_per_entry: 10000,
        })
        .size((100, 100))
        .iterations(10)
        .to_simulation();
    sim.init();
    sim
}

pub fn resource_allocation1(iterations: usize) {
    let chemistry_builder = ChemistryBuilder::with_key("lever");
    let chemistry = chemistry_builder.build();
    let mut sim = _sim(chemistry);

    let chemistry = &chemistry_builder.build();
    let unit_manifest = &sim.unit_manifest;

    for i in 0..iterations {
        for coord in CoordIterator::new(sim.world.size) {
            let pos = sim.world.get_position_at(&coord).unwrap();
            //println!("iterating coord: {:?}", coord);

            match sim.world.get_unit_at(&coord) {
                Some(unit) => {
                    let entry_id = unit.entry_id;
                    let unit_entry = &unit_manifest.units[entry_id].info;

                    // let next_resources =
                    //     chemistry.get_next_unit_resources(unit_entry, pos, unit, &sim.world, 1);
                    // sim.world.set_unit_resources_at(&coord, next_resources);
                }
                _ => {}
            };
        }
    }
}

pub fn resource_allocation2(iterations: usize) {
    let chemistry_builder = ChemistryBuilder::with_key("lever");
    let chemistry = chemistry_builder.build();
    let mut sim = _sim(chemistry);

    let chemistry = &chemistry_builder.build();
    let unit_manifest = &sim.unit_manifest;

    for i in 0..iterations {
        for coord in CoordIterator::new(sim.world.size) {
            let pos = sim.world.get_position_at(&coord).unwrap();
            //println!("iterating coord: {:?}", coord);

            match sim.world.get_unit_at(&coord) {
                Some(unit) => {
                    let entry_id = unit.entry_id;
                    let unit_entry = &unit_manifest.units[entry_id].info;

                    // let next_resources =
                    //     chemistry.get_next_unit_resources(unit_entry, pos, unit, &sim.world, 1);
                    // sim.world.set_unit_resources_at(&coord, next_resources);
                }

                _ => {}
            };
        }
    }
}

pub fn resource_allocation3(iterations: usize) {
    let chemistry_builder = ChemistryBuilder::with_key("lever");
    let chemistry = chemistry_builder.build();
    let mut sim = _sim(chemistry);

    let chemistry = &chemistry_builder.build();
    let unit_manifest = &sim.unit_manifest;

    for i in 0..iterations {
        let (max_x, max_y) = sim.world.size;

        for x in 0..max_x {
            for y in 0..max_y {
                let coord = (x, y);
                let pos = sim.world.get_position_at(&coord).unwrap();
                //println!("iterating coord: {:?}", coord);

                match sim.world.get_unit_at(&coord) {
                    Some(unit) => {
                        let entry_id = unit.entry_id;
                        let unit_entry = &unit_manifest.units[entry_id].info;

                        // let next_resources =
                        //     chemistry.get_next_unit_resources(unit_entry, pos, unit, &sim.world, 1);
                        // sim.world.set_unit_resources_at(&coord, next_resources);
                    }
                    _ => {}
                };
            }
        }
    }
}

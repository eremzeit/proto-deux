use crate::biology::unit_behavior::framed::FramedGenomeUnitBehavior;
use crate::runners::SimulationRunnerArgs;

use crate::chemistry::builder::*;
use crate::scenarios::simulations::lever::get_unit_entries_for_lever;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::{
    ChemistryInstance, CoordIterator, GeneticManifestData, UnitManifest,
};
use crate::simulation::config::SimulationBuilder;
use crate::simulation::Simulation;
use crate::unit_entry::builder::UnitEntryBuilder;

pub fn test_sim_perf() {
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

// pub fn basic(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
//     let chemistry_builder = ChemistryBuilder::with_key("lever");
//     SimulationBuilder::default()
//         .unit_entries(get_unit_entries_for_lever())
//         .size((1, 1))
//         .iterations(10)
// }

// pub fn with_genome(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
//     let chemistry_builder = ChemistryBuilder::with_key("lever");
//     let gm = GeneticManifest::defaults(&chemistry_builder.manifest()).wrap_rc();

//     use crate::biology::genome::framed::samples::lever::genome1;
//     let _genome1 = genome1(&gm);

//     let entry1 = UnitEntryBuilder::default()
//         .species_name("species1".to_string())
//         .behavior(FramedGenomeUnitBehavior::new(_genome1, gm.clone()).construct())
//         .build(&gm.chemistry_manifest);

//     SimulationBuilder::default()
//         .size((10, 1))
//         .iterations(1000)
//         .unit_manifest(UnitManifest {
//             units: vec![entry1],
//         })
// }

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

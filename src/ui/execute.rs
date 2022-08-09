use crate::biology::phenotype::mouse::*;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use crate::simulation::simulation_data::{
    new_threaded_simulation_reference, ThreadedSimulationReference,
};

use crate::ui;
use std::rc::Rc;
use std::time::Duration;

use crate::biology::genome::framed::common::*;
use crate::biology::genome::framed::samples::legacy;
use crate::biology::phenotype::framed::*;
use crate::simulation::common::UnitEntryBuilder;
// #[macro_use]
// pub mod macros {
//     // convenience for making a conrod Rect
//     #[macro_export]
//     macro_rules! rect(
//         ($x:expr, $y:expr, $w:expr, $h:expr) => (
//             Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
//         )
//     );
// }

// pub fn start_app() {
//     let (sender_from_ui, receiver_from_ui) = std::sync::mpsc::channel::<SimulationControlEvent>();
//     let sim_ref = new_threaded_simulation_reference();
//     let sim_ref2 = sim_ref.clone();

//     let handle = std::thread::spawn(move || {
//         let sim = SimulationBuilder::default()
//             .chemistry_key("cheese".to_string())
//             .unit_entries(vec![UnitEntryBuilder::default()
//                 .species_name("main".to_string())
//                 .phenotype(Rc::new(Box::new(Mouse::construct())))
//                 .default_resources(vec![("cheese", 100)])])
//             //("default", Rc::new(Box::new(Mouse::construct())), vec![("cheese", 100)], vec![])
//             .iterations(100000)
//             .size((50, 50))
//             .unit_placement(PlaceUnitsMethod::ManualSingleEntry {
//                 attributes: None,
//                 coords: vec![(1, 1)],
//             })
//             .to_simulation();

//         let mut executor = ThreadedSimulationExecutor::new(sim, sim_ref2, receiver_from_ui);
//         executor.is_paused = true;
//         executor.max_view_updates_per_second = 100;
//         executor.max_ticks_per_second = 100;
//         executor.run();
//     });

//     let s = sender_from_ui.clone();
//     //sender_from_ui.send(SimulationControlEvent::Resume);
//     ui::event_loop::start_ui_loop(sim_ref, sender_from_ui);
// }

pub fn start_app_with_genome() {
    let (sender_from_ui, receiver_from_ui) = std::sync::mpsc::channel::<SimulationControlEvent>();
    let sim_ref = new_threaded_simulation_reference();
    let sim_ref2 = sim_ref.clone();

    // use crate::chemistry::cheese::constants::NEW_UNIT_COST;

    let handle = std::thread::spawn(move || {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);
        // how do i say, find an open square adjacent to me, and use that as a parameter?  is that what a register could be used for? ephemeral data?

        use crate::biology::genome::framed::samples::get_genome1;
        let frames1 = get_genome1(&cm, &sm, &gm);

        let genome_values2 = legacy::get_genome2().build(&sm, &cm, &gm);
        let frames2 = FramedGenomeParser::parse(
            simple_convert_into_frames(genome_values2),
            cm.clone(),
            sm.clone(),
            gm.clone(),
        );
        let genome_values3 = legacy::get_genome3().build(&sm, &cm, &gm);
        let frames3 = FramedGenomeParser::parse(
            simple_convert_into_frames(genome_values3),
            cm.clone(),
            sm.clone(),
            gm.clone(),
        );

        let mut sim = SimulationBuilder::default()
            //.size((50, 30))
            .size((20, 20))
            .iterations(10000)
            .chemistry(CheeseChemistry::construct())
            .unit_placement(PlaceUnitsMethod::SimpleDrop { attributes: None })
            .unit_manifest(UnitManifest {
                units: vec![
                    UnitEntryBuilder::default()
                        .species_name("species1".to_string())
                        .phenotype(
                            FramedGenomePhenotype::new(frames1, gm.clone(), cm.clone(), sm.clone())
                                .construct(),
                        )
                        .default_resources(vec![("cheese", 100)])
                        .build(&cm, None),
                    // UnitEntryBuilder::default()
                    //     .species_name("species2")
                    //     .phenotype(
                    //         FramedGenomePhenotype::new(
                    //             &frames2,
                    //             gm.clone(),
                    //             cm.clone(),
                    //             sm.clone(),
                    //         )
                    //         .construct(),
                    //     )
                    //     .default_resources(vec![("cheese", 100)])
                    //     .build(&cm),
                ],
            })
            .to_simulation();

        let mut executor = ThreadedSimulationExecutor::new(sim, sim_ref2, receiver_from_ui);
        executor.is_paused = true;
        executor.max_view_updates_per_second = 4;
        executor.max_ticks_per_second = 2000;
        executor.run();
    });

    let s = sender_from_ui.clone();
    //sender_from_ui.send(SimulationControlEvent::Resume);
    ui::event_loop::start_ui_loop(sim_ref, sender_from_ui);
}

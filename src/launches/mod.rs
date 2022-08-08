use simulation::common::{
    BaseChemistry, Chemistry, EmptyPhenotype, PlaceUnits, PlaceUnitsMethod, Simulation,
    SimulationBuilder, SimulationConfig, SimulationControlEvent, SimulationEvent, SimulationSpec,
    ThreadedSimulationReference, UnitEntry, UnitEntryBuilder, UnitEntryData, UnitManifest,
};
use simulation::executors::threaded::ThreadedSimulationExecutor;
use simulation::simulation_data::new_threaded_simulation_reference;
use std::time::Duration;

// pub fn start_ui_and_sim(sim_config: Option<SimulationConfig>) {
//     let (sender_from_sim, receiver_from_sim) = std::sync::mpsc::channel::<SimulationEvent>();
//     let (sender_from_ui, receiver_from_ui) = std::sync::mpsc::channel::<SimulationControlEvent>(); //
//     let sim_ref = new_threaded_simulation_reference();
//     let sim_ref2 = sim_ref.clone();

//     let handle = std::thread::spawn(move || {
//         let sim = SimulationBuilder::default()
//             .unit_entries(vec![UnitEntryBuilder::with_species_name("main")
//                 .phenotype(EmptyPhenotype::construct())
//                 .default_resources(vec![("cheese", 100)])])
//             .size((5, 5))
//             .chemistry_key("cheese".to_string())
//             .specs(vec![Box::new(PlaceUnits {
//                 method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
//             })])
//             .to_simulation();

//         let mut executor = ThreadedSimulationExecutor::new(sim, sim_ref2);
//     });

//     std::thread::sleep(Duration::new(1, 0));
//     sender_from_ui.send(SimulationControlEvent::Start);
//     ui::conrod::event_loop::start_ui_loop(sim_ref, receiver_from_sim, sender_from_ui);
// }

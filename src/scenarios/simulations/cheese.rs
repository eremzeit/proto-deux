use variants::CheeseChemistry;

use crate::biology::genome::framed::builders::simple_convert_into_frames;
use crate::biology::genome::framed::builders::FramedGenomeParser;
use crate::biology::genome::framed::samples::legacy;
use crate::biology::unit_behavior::framed::FramedGenomeUnitBehavior;
use crate::biology::unit_behavior::mouse::simple_mouse::SimpleMouse;
use crate::biology::unit_behavior::mouse::*;
use crate::runners::SimulationRunnerArgs;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use std::rc::Rc;

pub fn basic(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let specs = SimulationSpecs {
        chemistry_key: "cheese".to_string(),
        place_units_method: PlaceUnitsMethod::RandomRegionDrop {
            attributes: None,
            units_per_entry: 1,
            region_pct_rect: (0.40, 0.40, 0.60, 0.60),
        },
        ..Default::default()
    };

    SimulationBuilder::default()
        .specs(specs)
        .unit_entries(get_unit_entries_for_cheese())
        .size((50, 50))
        .iterations(1000)
}

pub fn with_genome(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let specs = SimulationSpecs {
        chemistry_key: "cheese".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };

    let (cm, sm, gm) = specs.context();

    // how do i say, find an open square adjacent to me, and use that as a parameter?  is that what a register could be used for? ephemeral data?

    use crate::biology::genome::framed::samples::cheese::get_genome1;
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

    let entry1 = UnitEntryBuilder::default()
        .species_name("species1".to_string())
        .behavior(
            FramedGenomeUnitBehavior::new(frames1, gm.clone(), cm.clone(), sm.clone()).construct(),
        )
        .default_resources(vec![("cheese", 100)])
        .build(&cm, None);

    let entry2 = UnitEntryBuilder::default()
        .species_name("species2".to_string())
        .behavior(
            FramedGenomeUnitBehavior::new(frames2, gm.clone(), cm.clone(), sm.clone()).construct(),
        )
        .default_resources(vec![("cheese", 100)])
        .build(&cm, None);

    SimulationBuilder::default()
        .specs(specs)
        .size((50, 30))
        .iterations(1000)
        .unit_manifest(UnitManifest {
            units: vec![entry1, entry2],
        })
}

pub fn get_unit_entries_for_cheese() -> Vec<UnitEntryBuilder> {
    vec![UnitEntryBuilder::default()
        .species_name("main".to_string())
        .behavior(Rc::new(Box::new(SimpleMouse::construct())))
        .default_resources(vec![("cheese", 200)])]
}

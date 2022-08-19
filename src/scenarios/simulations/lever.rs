use variants::CheeseChemistry;

use crate::biology::genome::framed::builders::simple_convert_into_frames;
use crate::biology::genome::framed::builders::FramedGenomeParser;
use crate::biology::genome::framed::samples::legacy;
use crate::biology::unit_behavior::framed::FramedGenomeUnitBehavior;
use crate::biology::unit_behavior::lever::SimpleLever;
use crate::biology::unit_behavior::mouse::simple_mouse::SimpleMouse;
use crate::biology::unit_behavior::mouse::*;
use crate::runners::SimulationRunnerArgs;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::variants::LeverChemistry;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use std::rc::Rc;

pub fn basic(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let specs = SimulationSpecs {
        chemistry_key: "lever".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };

    let (cm, sm, gm) = specs.context();

    SimulationBuilder::default()
        .specs(specs)
        .unit_entries(get_unit_entries_for_lever())
        .size((1, 1))
        .iterations(10)
}

pub fn with_genome(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let specs = SimulationSpecs {
        chemistry_key: "lever".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };

    let (cm, sm, gm) = specs.context();

    use crate::biology::genome::framed::samples::lever::genome1;
    let _genome1 = genome1(&cm, &sm, &gm);

    let entry1 = UnitEntryBuilder::default()
        .species_name("species1".to_string())
        .behavior(
            FramedGenomeUnitBehavior::new(_genome1, gm.clone(), cm.clone(), sm.clone()).construct(),
        )
        .build(&cm, None);

    SimulationBuilder::default()
        .size((10, 1))
        .specs(specs)
        .iterations(1000)
        .unit_manifest(UnitManifest {
            units: vec![entry1],
        })
}

pub fn get_unit_entries_for_lever() -> Vec<UnitEntryBuilder> {
    vec![UnitEntryBuilder::default()
        .species_name("main".to_string())
        .behavior(Rc::new(Box::new(SimpleLever::construct())))]
}

use variants::CheeseChemistry;

use crate::biology::genome::framed::builders::simple_convert_into_frames;
use crate::biology::genome::framed::builders::FramedGenomeCompiler;
use crate::biology::genome::framed::samples::legacy;
use crate::biology::unit_behavior::framed::FramedGenomeUnitBehavior;
use crate::biology::unit_behavior::mouse::simple_mouse::SimpleMouse;
use crate::biology::unit_behavior::mouse::*;
use crate::runners::SimulationRunnerArgs;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;
use crate::simulation::common::*;
use crate::simulation::config::*;
use crate::simulation::executors::threaded::ThreadedSimulationExecutor;
use std::rc::Rc;

pub fn basic(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");

    // let specs = SimulationSpecs {
    //     chemistry_key: "cheese".to_string(),
    //     place_units_method: ,
    //     ..Default::default()
    // };

    SimulationBuilder::default()
        .chemistry(chemistry_builder.build())
        .unit_entries(get_unit_entries_for_cheese())
        .size((50, 50))
        .iterations(1000)
}

pub fn with_genome(sim_args: &SimulationRunnerArgs) -> SimulationBuilder {
    let chemistry_builder = ChemistryBuilder::with_key("cheese");
    let gm = GeneticManifest::defaults(&chemistry_builder.manifest()).wrap_rc();

    // let specs = SimulationSpecs {
    //     chemistry_key: "cheese".to_string(),
    //     place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
    //     ..Default::default()
    // };

    // let (cm, sm, gm) = specs.context();

    // how do i say, find an open square adjacent to me, and use that as a parameter?  is that what a register could be used for? ephemeral data?

    use crate::biology::genome::framed::samples::cheese::get_genome1;
    let frames1 = get_genome1(&gm);

    let genome_values2 = legacy::get_genome2().build(&gm);
    let frames2 = FramedGenomeCompiler::compile(simple_convert_into_frames(genome_values2), &gm);
    let genome_values3 = legacy::get_genome3().build(&gm);
    let frames3 = FramedGenomeCompiler::compile(simple_convert_into_frames(genome_values3), &gm);

    let entry1 = UnitEntryBuilder::default()
        .species_name("species1".to_string())
        .behavior(FramedGenomeUnitBehavior::new(frames1, gm.clone()).construct())
        .default_resources(vec![("cheese", 100)])
        .build(&chemistry_builder.manifest());

    let entry2 = UnitEntryBuilder::default()
        .species_name("species2".to_string())
        .behavior(FramedGenomeUnitBehavior::new(frames2, gm.clone()).construct())
        .default_resources(vec![("cheese", 100)])
        .build(&chemistry_builder.manifest());

    SimulationBuilder::default()
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

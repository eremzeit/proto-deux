use super::*;
use crate::biology::genome::framed::common::*;
use crate::biology::genome::framed::samples::legacy::get_genome1;
use crate::simulation::common::*;
//use crate::biology::genome::framed::macros::util::GenomeBuilder;
use crate::biology::genome::framed::*;
use crate::biology::unit_behavior::framed::*;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;

pub fn test_with_genome() {
    let specs = SimulationSpecs {
        chemistry_key: "cheese".to_string(),
        place_units_method: PlaceUnitsMethod::SimpleDrop { attributes: None },
        ..Default::default()
    };
    let (cm, sm, gm) = specs.context();

    let genome_values1 = get_genome1().build(&sm, &cm, &gm);
    let frames1 =
        FramedGenomeCompiler::compile(simple_convert_into_frames(genome_values1), &cm, &sm, &gm);

    let mut sim = SimulationBuilder::default()
        .headless(true)
        .size((20, 20))
        .iterations(10000)
        .unit_manifest(UnitManifest {
            units: vec![UnitEntryBuilder::default()
                .species_name("species1".to_string())
                .behavior(
                    FramedGenomeUnitBehavior::new(frames1, gm.clone(), cm.clone(), sm.clone())
                        .construct(),
                )
                .default_resources(vec![("cheese", 100)])
                .build(&cm, None)],
        })
        .to_simulation();

    println!("starting");

    let mut executor = SimpleSimulationExecutor::new(sim);
    executor.start();
}

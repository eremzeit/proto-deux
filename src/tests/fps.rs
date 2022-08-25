use super::*;
use crate::biology::genome::framed::common::*;
use crate::biology::genome::framed::samples::legacy::get_genome1;
use crate::simulation::common::*;
//use crate::biology::genome::framed::macros::util::GenomeBuilder;
use crate::biology::genome::framed::*;
use crate::biology::unit_behavior::framed::*;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::helpers::place_units::PlaceUnitsMethod;

pub fn test_with_genome() {
    let chemistry = ChemistryBuilder::with_key("cheese").build();
    let gm = GeneticManifest::defaults(chemistry.get_manifest()).wrap_rc();

    let genome_values1 = get_genome1().build(&gm);
    let frames1 = FramedGenomeCompiler::compile(simple_convert_into_frames(genome_values1), &gm);

    let mut sim = SimulationBuilder::default()
        .place_units_method(PlaceUnitsMethod::SimpleDrop { attributes: None })
        .chemistry(chemistry)
        .size((20, 20))
        .iterations(10000)
        .unit_manifest(UnitManifest {
            units: vec![UnitEntryBuilder::default()
                .species_name("species1".to_string())
                .behavior(FramedGenomeUnitBehavior::new(frames1, gm.clone()).construct())
                .default_resources(vec![("cheese", 100)])
                .build(&gm.chemistry_manifest)],
        })
        .to_simulation();

    println!("starting");

    let mut executor = SimpleSimulationExecutor::new(sim);
    executor.start();
}

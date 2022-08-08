use super::*;
use crate::biology::genome::framed::common::*;
use crate::biology::genome::framed::samples::legacy::get_genome1;
use crate::simulation::common::*;
//use crate::biology::genome::framed::macros::util::GenomeBuilder;
use crate::biology::genome::framed::*;
use crate::biology::phenotype::framed::*;

pub fn test_with_genome() {
    let gm = GeneticManifest::new();
    let cm = CheeseChemistry::default_manifest();
    let sm = SensorManifest::with_default_sensors(&cm);

    let genome_values1 = get_genome1().build(&sm, &cm, &gm);
    let frames1 = FramedGenomeParser::parse(
        simple_convert_into_frames(genome_values1),
        cm.clone(),
        sm.clone(),
        gm.clone(),
    );

    let mut sim = SimulationBuilder::default()
        .headless(true)
        .size((20, 20))
        .iterations(10000)
        .chemistry(CheeseChemistry::construct())
        .unit_placement(PlaceUnitsMethod::SimpleDrop { attributes: None })
        .unit_manifest(UnitManifest {
            units: vec![UnitEntryBuilder::default()
                .species_name("species1".to_string())
                .phenotype(
                    FramedGenomePhenotype::new(frames1, gm.clone(), cm.clone(), sm.clone())
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
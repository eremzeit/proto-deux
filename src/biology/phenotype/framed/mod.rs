pub mod execution;
pub mod types;

use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::types::{
    BooleanVariable, DisjunctiveClause, Frame, FramedGenome, FramedGenomeWord,
};
pub use crate::biology::phenotype::framed::execution::GenomeExecutionContext;
pub use crate::biology::phenotype::framed::types::*;
use crate::biology::phenotype::Phenotype;
use crate::biology::sensor_manifest::SensorId;
use crate::chemistry;
use crate::chemistry::reactions::ReactionCallParam;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::rc::Rc;

pub mod common {
    pub use super::FramedGenomePhenotype;
    pub use crate::biology::phenotype::framed::types::*;
}

pub struct FramedGenomePhenotype {
    pub genome: FramedGenome,
    pub chemistry_manifest: ChemistryManifest,
    pub sensor_manifest: SensorManifest,
    pub genetic_manifest: GeneticManifest,
}

impl Phenotype for FramedGenomePhenotype {
    fn get_behavior(
        &self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> PhenotypeResult {
        let sensor_context = SensorContext::from(world, sim_attr, coord);

        //let computation_points = if world.tick > 5000 { 20 } else { 100 };
        let computation_points = 100;

        let new_registers = vec![0; 10];
        let mut execution_context = GenomeExecutionContext::new(
            &self.genome.frames,
            &sensor_context,
            new_registers,
            &self.chemistry_manifest,
            &self.sensor_manifest,
            &self.genetic_manifest,
            computation_points,
        );

        let reactions = execution_context.execute();
        //println!("REACTIONS AOEUAOEU: {:?}", &reactions);
        //println!("consumed_compute_points: {}", execution_context.consumed_compute_points);

        PhenotypeResult {
            reactions: reactions.clone(),
        }
    }
}

// pub enum ParsedGenomeParam {
//     Constant(OperatorParam),
//     SensorLookup(SensorId),
//     Register(RegisterId),
//     Random(usize)
// }

// pub fn eval_parsed_genome_param(param: &ParsedGenomeParam, coord: &Coord, world: &World, sim_attr: &SimulationAttributes, sm: &SensorManifest) -> ReactionCallParam {
//     match param {
//         ParsedGenomeParam::Constant(c) => {
//             return *c as ReactionCallParam;
//         },
//
//         ParsedGenomeParam::SensorLookup(sensor_id) => {
//             let sensor = sm.sensors[*sensor_id as usize];
//             sensor.calculate(&SensorContext {
//                 world,
//                 coord,
//                 sim_attr,
//             }).try_into().unwrap()
//         },
//
//         ParsedGenomeParam::Random(max_val) => {
//             let sensor = sm.sensors[*sensor_id as usize];
//             sensor.calculate(&SensorContext {
//                 world,
//                 coord,
//                 sim_attr,
//             }).try_into().unwrap()
//         },
//
//     }
// }

impl FramedGenomePhenotype {
    pub fn new(
        genome: FramedGenome,
        genetic_manifest: GeneticManifest,
        chemistry_manifest: ChemistryManifest,
        sensor_manifest: SensorManifest,
    ) -> Self {
        Self {
            genome,
            genetic_manifest,
            chemistry_manifest,
            sensor_manifest,
        }
    }

    pub fn construct(self) -> BoxedPhenotype {
        Rc::new(Box::new(self))
    }
}

pub mod test {
    use super::{FramedGenomePhenotype, GenomeExecutionContext};
    use crate::biology::genome::framed::common::*;
    use crate::biology::genome::framed::convert::simple_convert_into_frames;
    use crate::biology::genome::framed::parsing::FramedGenomeParser;
    use crate::biology::genome::framed::render::render_frames;
    use crate::biology::genome::framed::*;
    use crate::simulation::common::*;
    use std::rc::Rc;
    pub fn count_units(world: &World) -> u64 {
        let mut count: u64 = 0;
        for coord in CoordIterator::new(world.size) {
            if world.has_unit_at(&coord) {
                count += 1;
            }
        }

        return count;
    }

    #[test]
    fn genome_execution__execute() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();
        let sm = SensorManifest::with_default_sensors(&cm);

        let genome_values = genome!(gene(
            if_any(all((is_truthy, 1, 0, 0))),
            then_do(new_unit(0, 0, 0))
        ))
        .build(&sm, &cm, &gm);

        let framed_vals = simple_convert_into_frames(genome_values);
        let frames = FramedGenomeParser::parse(framed_vals, cm.clone(), sm.clone(), gm.clone());

        let (sender, receiver) = std::sync::mpsc::channel::<SimulationEvent>();
        let mut sim = SimulationBuilder::default()
            .sim_events(sender)
            .size((3, 3))
            .iterations(100)
            .chemistry(CheeseChemistry::construct())
            .headless(true)
            .unit_placement(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(1, 1)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntryBuilder::default()
                    .species_name("main".to_string())
                    .phenotype(
                        FramedGenomePhenotype::new(frames, gm.clone(), cm.clone(), sm.clone())
                            .construct(),
                    )
                    .default_resources(vec![("cheese", 1000)])
                    .build(&cm, None)],
            })
            .to_simulation();

        assert_eq!(count_units(&sim.world), 1);
        sim.tick();
        assert_eq!(count_units(&sim.world), 2);

        assert!(sim.world.has_unit_at(&(1, 2)));
        assert!(sim.world.has_unit_at(&(1, 1)));
    }
}

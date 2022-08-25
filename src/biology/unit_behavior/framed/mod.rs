pub mod types;

use rand::Rng;

use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
pub use crate::biology::genome::framed::execution::GenomeExecutionContext;
use crate::biology::genome::framed::types::{
    BooleanVariable, CompiledFramedGenome, DisjunctiveClause, Frame, FramedGenomeWord,
};
use crate::biology::sensor_manifest::SensorId;
pub use crate::biology::unit_behavior::framed::types::*;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
use crate::chemistry::reactions::ReactionCallParam;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::rc::Rc;

pub mod common {
    pub use super::FramedGenomeUnitBehavior;
    pub use crate::biology::unit_behavior::framed::types::*;
}

pub struct FramedGenomeUnitBehavior {
    pub genome: CompiledFramedGenome,
    pub genetic_manifest: Rc<GeneticManifest>,
}

impl UnitBehavior for FramedGenomeUnitBehavior {
    fn get_behavior(
        &self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> UnitBehaviorResult {
        let sensor_context = SensorContext::from(world, sim_attr, coord);

        //let computation_points = if world.tick > 5000 { 20 } else { 100 };
        let computation_points = 100;

        let new_registers = vec![0; 10];
        let mut execution_context = GenomeExecutionContext::new(
            &self.genome.frames,
            &sensor_context,
            new_registers,
            &self.genetic_manifest,
            computation_points,
        );

        // let mut rng = rand::thread_rng();
        // execution_context.override_channel = Some(rng.gen_range(0..4));

        let reactions = execution_context.execute();
        // println!("EXECUTING reactions: {:?}", &reactions);
        //println!("consumed_compute_points: {}", execution_context.consumed_compute_points);

        UnitBehaviorResult {
            reactions: reactions.clone(),
            consumed_execution_points: execution_context.consumed_compute_points,
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

impl FramedGenomeUnitBehavior {
    pub fn new(genome: CompiledFramedGenome, genetic_manifest: Rc<GeneticManifest>) -> Self {
        Self {
            genome,
            genetic_manifest,
        }
    }

    pub fn construct(self) -> BoxedUnitBehavior {
        Rc::new(Box::new(self))
    }
}

pub mod test {
    use super::{FramedGenomeUnitBehavior, GenomeExecutionContext};
    use crate::biology::genome::framed::common::*;
    use crate::biology::genome::framed::compile::FramedGenomeCompiler;
    use crate::biology::genome::framed::convert::simple_convert_into_frames;
    use crate::biology::genome::framed::render::render_frames;
    use crate::biology::genome::framed::*;
    use crate::chemistry::helpers::place_units::PlaceUnitsMethod;
    use crate::chemistry::variants::CheeseChemistry;
    use crate::simulation::common::builder::ChemistryBuilder;
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
        let chemistry = ChemistryBuilder::with_key("cheese").build();
        let gm = Rc::new(GeneticManifest::defaults(chemistry.get_manifest()));
        let cm = &gm.chemistry_manifest;
        let genome_values = genome!(gene(
            if_any(all((is_truthy, 1, 0, 0))),
            then_do(new_unit(0, 0, 0))
        ))
        .build(&gm);

        let framed_vals = simple_convert_into_frames(genome_values);
        let frames = FramedGenomeCompiler::compile(framed_vals, &gm);

        println!("genome: \n{}", frames.display(&gm));

        let mut sim = SimulationBuilder::default()
            .chemistry(chemistry)
            .size((3, 3))
            .iterations(100)
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(1, 1)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntryBuilder::default()
                    .species_name("main".to_string())
                    .behavior(FramedGenomeUnitBehavior::new(frames, gm.clone()).construct())
                    .default_resources(vec![("cheese", 1000)])
                    .build(&cm)],
            })
            .to_simulation();

        assert_eq!(count_units(&sim.world), 1);
        sim.tick();
        assert_eq!(count_units(&sim.world), 2);

        assert!(sim.world.has_unit_at(&(1, 2)));
        assert!(sim.world.has_unit_at(&(1, 1)));
    }
}

pub mod framed;
pub mod lever;
pub mod mouse;

use crate::biology::genetic_manifest::predicates::Operator;
pub use crate::biology::unit_behavior::framed::ParsedGenomeParam;
use crate::chemistry::reactions::ReactionCall;
use crate::simulation::common::*;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

pub trait UnitBehavior {
    fn get_behavior(
        &self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> UnitBehaviorResult {
        UnitBehaviorResult { reactions: vec![] }
    }
}

// #[derive(Clone)]

pub const NUM_REACTION_PARAMS: u32 = 3;

#[derive(PartialEq, Debug)]
pub struct UnitBehaviorResult {
    pub reactions: Vec<ReactionCall>,
    //pub register_changes: PhenotypeRegisterChanges,
}

impl UnitBehaviorResult {
    pub fn display(&self, chemistry_manifest: &ChemistryManifest) -> String {
        let mut s = "".to_string();
        for reaction_call in &self.reactions {
            let reaction_id = reaction_call.0;
            let key = chemistry_manifest.reactions[reaction_id as usize]
                .key
                .clone();
            s.push_str(&format!(
                "ParamedReactionCall({}, {:?}, {:?}, {:?}",
                key, reaction_call.1, reaction_call.2, reaction_call.3
            ));
        }

        s
    }
}

pub struct NullBehavior {}
impl UnitBehavior for NullBehavior {}

pub type BoxedUnitBehavior = Rc<Box<dyn UnitBehavior>>;

impl NullBehavior {
    pub fn construct() -> BoxedUnitBehavior {
        Rc::new(Box::new(Self {}))
    }
}

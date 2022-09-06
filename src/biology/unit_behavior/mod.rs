pub mod framed;
pub mod lever;
pub mod mouse;

use crate::biology::genetic_manifest::predicates::OperatorImplementation;
pub use crate::biology::unit_behavior::framed::ParsedGenomeParam;
use crate::chemistry::reactions::ReactionCall;
use crate::simulation::common::*;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

pub trait UnitBehavior {
    fn get_behavior(
        &mut self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> UnitBehaviorResult {
        UnitBehaviorResult::with_reactions(vec![])
    }
}

// #[derive(Clone)]

pub const NUM_REACTION_PARAMS: u32 = 3;

#[derive(PartialEq, Debug)]
pub struct UnitBehaviorResult {
    pub reactions: Vec<ReactionCall>,
    //pub register_changes: PhenotypeRegisterChanges,
    pub consumed_execution_points: u64,
}

impl UnitBehaviorResult {
    pub fn with_reactions(reactions: Vec<ReactionCall>) -> Self {
        UnitBehaviorResult {
            reactions,
            consumed_execution_points: 0,
        }
    }

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

// pub type BoxedUnitBehavior = Rc<Box<dyn UnitBehavior>>;

impl NullBehavior {
    pub fn construct() -> Rc<RefCell<dyn UnitBehavior>> {
        Rc::new(RefCell::new(Self {}))
    }
}

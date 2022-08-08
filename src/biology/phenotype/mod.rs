pub mod framed;
pub mod mouse;

use crate::biology::genetic_manifest::predicates::Operator;
pub use crate::biology::phenotype::framed::ParsedGenomeParam;
use crate::chemistry::reactions::ReactionCall;
use crate::simulation::common::*;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

pub trait Phenotype {
    fn get_behavior(
        &self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> PhenotypeResult {
        PhenotypeResult { reactions: vec![] }
    }
}

#[derive(Clone)]
pub struct EmptyPhenotype {}

pub const NUM_REACTION_PARAMS: u32 = 3;

#[derive(PartialEq, Debug)]
pub struct PhenotypeResult {
    pub reactions: Vec<ReactionCall>,
    //pub register_changes: PhenotypeRegisterChanges,
}

impl PhenotypeResult {
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

impl Phenotype for EmptyPhenotype {}

pub type BoxedPhenotype = Rc<Box<dyn Phenotype>>;

impl EmptyPhenotype {
    pub fn construct() -> BoxedPhenotype {
        Rc::new(Box::new(Self {}))
    }
}

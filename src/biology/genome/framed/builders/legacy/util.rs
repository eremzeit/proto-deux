use crate::biology::genetic_manifest::predicates::{
    OperatorId, OperatorImplementation, OperatorManifest, OperatorParam, OperatorParamDefinition,
    OperatorParamType,
};
use crate::biology::genome::framed::common::FramedGenomeValue;
use crate::biology::sensor_manifest::SensorManifest;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
use crate::chemistry::properties::AttributeIndex;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use crate::util::{grid_direction_from_string, grid_direction_to_num};
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

pub type GenomeBuildFunction = Rc<dyn Fn(&GeneticManifest) -> Vec<FramedGenomeValue>>;

pub struct GenomeBuilderLegacy {
    pub build_fn: GenomeBuildFunction,
}

impl GenomeBuilderLegacy {
    pub fn new(build_fn: GenomeBuildFunction) -> Self {
        Self { build_fn }
    }
    pub fn build(&self, genetic_manifest: &GeneticManifest) -> Vec<FramedGenomeValue> {
        (self.build_fn)(genetic_manifest)
    }
}

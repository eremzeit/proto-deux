use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorId, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
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

pub type GenomeBuildFunction =
    Rc<dyn Fn(&SensorManifest, &ChemistryManifest, &GeneticManifest) -> Vec<FramedGenomeValue>>;

pub struct GenomeBuilder {
    pub build_fn: GenomeBuildFunction,
}

impl GenomeBuilder {
    pub fn new(build_fn: GenomeBuildFunction) -> Self {
        Self { build_fn }
    }
    pub fn build(
        &self,
        sensor_manifest: &SensorManifest,
        chemistry_manifest: &ChemistryManifest,
        genetic_manifest: &GeneticManifest,
    ) -> Vec<FramedGenomeValue> {
        (self.build_fn)(sensor_manifest, chemistry_manifest, genetic_manifest)
    }
}

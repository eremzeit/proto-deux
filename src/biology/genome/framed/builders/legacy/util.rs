use std::rc::Rc;
use std::fmt::{Debug, Result, Formatter};
use biology::phenotype::{Phenotype};
use simulation::world::{World};
use util::{Coord};
use chemistry::{ChemistryInstance, ReactionId};
use biology::genetic_manifest::predicates::{Operator, OperatorId, OperatorParam, OperatorParamType, OperatorSet, OperatorParamDefinition };
use biology::genetic_manifest::{GeneticManifest};
use simulation::common::{*};
use chemistry::properties::{AttributeIndex};
use biology::sensor_manifest::SensorManifest;
use biology::genome::framed::common::{FramedGenomeValue};
use chemistry;
use util::{grid_direction_from_string, grid_direction_to_num};


pub type GenomeBuildFunction =
    Rc<dyn Fn(&SensorManifest, &ChemistryManifest, &GeneticManifest) -> Vec<FramedGenomeValue>>;

pub struct GenomeBuilder {
    pub build_fn: GenomeBuildFunction,
}

impl GenomeBuilder {
    pub fn new(build_fn: GenomeBuildFunction) -> Self {
        Self {
            build_fn
        }
    }
    pub fn build(&self, sensor_manifest: &SensorManifest, chemistry_manifest: &ChemistryManifest, genetic_manifest: &GeneticManifest) -> Vec<FramedGenomeValue> {
        (self.build_fn)(sensor_manifest, chemistry_manifest, genetic_manifest)
    }
}

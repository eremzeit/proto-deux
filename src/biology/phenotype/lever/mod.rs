use crate::simulation::{
    common::{ChemistryInstance, Coord},
    world::World,
    SimulationAttributes,
};

use super::{Phenotype, PhenotypeResult};

pub struct SimpleLever {}
impl Phenotype for SimpleLever {
    fn get_behavior(
        &self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> PhenotypeResult {
        let reactions = &chemistry.get_manifest().reactions;

        return PhenotypeResult {
            reactions: vec![(
                0, //pull lever
                1, // pull 1 time
                0, 0,
            )],
        };
    }
}

impl SimpleLever {
    pub fn construct() -> Self {
        SimpleLever {}
    }
}

use crate::simulation::{
    common::{ChemistryInstance, Coord},
    world::World,
    SimulationAttributes,
};

use super::{UnitBehavior, UnitBehaviorResult};

pub struct SimpleLever {}
impl UnitBehavior for SimpleLever {
    fn get_behavior(
        &mut self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> UnitBehaviorResult {
        let reactions = &chemistry.get_manifest().reactions;

        return UnitBehaviorResult::with_reactions(vec![(
            0, //pull lever
            1, // pull 1 time
            0, 0,
        )]);
    }
}

impl SimpleLever {
    pub fn construct() -> Self {
        SimpleLever {}
    }
}

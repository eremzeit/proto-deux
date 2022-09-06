use crate::biology::genetic_manifest::predicates::OperatorImplementation;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry::variants::cheese::defs;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::iterators::CoordOffsetIterator;
use crate::simulation::world::World;
use crate::util::{grid_direction_to_num, Coord, GridDirection};

use rand::Rng;

pub struct SimpleMouse {}
impl UnitBehavior for SimpleMouse {
    fn get_behavior(
        &mut self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> UnitBehaviorResult {
        let reactions = &chemistry.get_manifest().reactions;

        let pos_resources = defs::PositionResourcesLookup::new();

        if world.get_pos_resource_at(coord, pos_resources.cheese) > 10 {
            return UnitBehaviorResult::with_reactions(vec![(
                defs::REACTION_ID_GOBBLE_CHEESE,
                0,
                0,
                0,
            )]);
        }

        for (_coord, _dir) in CoordOffsetIterator::new(coord, &world.size) {
            if world.get_pos_resource_at(&_coord, pos_resources.cheese) > 10 {
                return UnitBehaviorResult::with_reactions(vec![(
                    defs::REACTION_ID_MOVE_UNIT,
                    grid_direction_to_num(_dir) as u16,
                    0,
                    0,
                )]);
            }
        }

        UnitBehaviorResult::with_reactions(vec![(
            defs::REACTION_ID_MOVE_UNIT,
            grid_direction_to_num(GridDirection::Up) as u16,
            0,
            0,
        )])
        // let mut rng = rand::thread_rng();
        // let mut reaction_id: ReactionId = rng.gen();
        // reaction_id = reaction_id % reactions.len() as ReactionId;
        //(reaction_id, rng.gen(), rng.gen(), rng.gen())
    }
    // fn get_behavior(&self, coord: &Coord, world: &World, chemistry: &ChemistryInstance, registers: &PhenotypeRegisters) -> PhenotypeResult {
    //
    // }
    // fn get_reaction(
    //     &self,
    //     coord: &Coord,
    //     world: &World,
    //     chemistry: &ChemistryInstance,
    // ) -> ReactionId {
    //     let lookup = PositionResourcesLookup::new();

    //     let pos = world.get_position_at(coord).unwrap();
    //     let amount = pos.get_resource(lookup.cheese);
    //
    //     if amount > 0 {
    //         return 0;
    //     }

    //     0
    // }
}

impl SimpleMouse {
    pub fn construct() -> Self {
        SimpleMouse {}
    }
}

pub mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::{
        chemistry::actions::*,
        simulation::common::{
            builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
            variants::CheeseChemistry,
        },
    };

    #[test]
    pub fn basic() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();
        let mut sim = SimulationBuilder::default()
            .chemistry(chemistry)
            .size((5, 5))
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(2, 0)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();

        sim.tick();
    }
}

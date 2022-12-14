use rand::Rng;

use crate::biology::genetic_manifest::predicates::OperatorImplementation;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry::variants::cheese::defs;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::iterators::CoordOffsetIterator;
use crate::simulation::world::World;
use crate::util::{grid_direction_to_num, Coord, GridDirection};

pub fn get_direction_where<F>(world: &World, coord: &Coord, f: F) -> Option<GridDirection>
where
    F: Fn(&World, &Coord) -> bool,
{
    for (_coord, _dir) in CoordOffsetIterator::new(coord, &world.size) {
        if f(world, &_coord) {
            return Some(_dir);
        }
    }

    None
}

pub struct SmartMouse {}
impl UnitBehavior for SmartMouse {
    fn get_behavior(
        &mut self,
        coord: &Coord,
        sim_attr: &SimulationAttributes,
        world: &World,
        chemistry: &ChemistryInstance,
    ) -> UnitBehaviorResult {
        let reactions = &chemistry.get_manifest().reactions;
        let pos_resources = defs::PositionResourcesLookup::new();
        let pos_attributes = defs::PositionAttributesLookup::new();

        if world.get_pos_resource_at(coord, pos_resources.milk) > 50 / 2 {
            return UnitBehaviorResult::with_reactions(vec![(
                defs::REACTION_ID_GOBBLE_CHEESE,
                0,
                0,
                0,
            )]);
        }

        for (_coord, _dir) in CoordOffsetIterator::new(coord, &world.size) {
            if world.get_pos_resource_at(&_coord, pos_resources.milk) > 10 {
                return UnitBehaviorResult::with_reactions(vec![(
                    defs::REACTION_ID_MOVE_UNIT,
                    grid_direction_to_num(_dir) as u16,
                    0,
                    0,
                )]);
            }
        }

        let dir_of_cheese_source = get_direction_where(world, coord, |w, _coord| {
            w.get_pos_attribute_at(_coord, pos_attributes.is_cheese_dispenser)
                .unwrap_bool()
        });

        if let Some(dir) = dir_of_cheese_source {
            return UnitBehaviorResult::with_reactions(vec![(
                defs::REACTION_ID_MOVE_UNIT,
                grid_direction_to_num(dir) as u16,
                0,
                0,
            )]);
        }

        let mut rnd = rand::thread_rng();
        let direction = rnd.gen_range(0..4);

        UnitBehaviorResult::with_reactions(vec![(
            defs::REACTION_ID_MOVE_UNIT,
            direction as u16,
            0,
            0,
        )])
    }
}

impl SmartMouse {
    pub fn construct() -> Self {
        SmartMouse {}
    }
}

use crate::chemistry::variants::CheeseChemistry;
use crate::simulation::common::CoordIterator;
use crate::simulation::common::*;
use crate::simulation::config::SimulationConfig;
use crate::simulation::iterators::*;
use crate::simulation::unit::{add_resources_to, UnitAttributes, UnitResources};
use crate::util::text_grid::TextGridOptions;
use crate::util::*;
use std::sync::Arc;
use typemap::{CloneMap, Key};

// function linearDiffTransitsForCoord(coord, simulation, config = {}) {
//   const world = simulation.world
//   const fromUnit = world.get(coord).unit//
//   return dirToOffset.map((offset) => {
//     const destCoords = calcCoordsByOffset(coord, offset, world.xSize, world.ySize)
//     if (!destCoords) {
//       return
//     }//
//     const destPos = world.get(destCoords)
//     if (destPos.unit) {
//       const amounts = _.mapValues(fromUnit.storedResources, (amount, resourceName) => {
//         const toAmount = destPos.unit.storedResources[resourceName] || 0
//         const diff = Math.max(amount - toAmount, 0)
//         return Math.round(diff / DIFFUSION_FACTOR)
//       })
//       return [ destCoords, amounts ]
//     } else {
//       return
//     }
//   }).filter(x => !!x)
// }
pub fn calculate_linear_diff_transits(
    coord: Coord,
    world: &World,
    chemistry: &Arc<Box<dyn Chemistry>>,
) -> Vec<(Coord, UnitResources)> {
    let diffusion_factor = 10;
    let known_resources = &chemistry.get_manifest().unit_resources;

    let mut our_delta = chemistry.get_manifest().empty_unit_resources();
    let mut their_deltas = vec![
        chemistry.get_manifest().empty_unit_resources(),
        chemistry.get_manifest().empty_unit_resources(),
        chemistry.get_manifest().empty_unit_resources(),
        chemistry.get_manifest().empty_unit_resources(),
    ];

    let mut result = vec![];
    let directions = vec![
        GridDirection::Up,
        GridDirection::Right,
        GridDirection::Down,
        GridDirection::Left,
    ];

    for i in 0..directions.len() {
        let dir = directions[i].clone();
        let __coord = coord_by_direction_offset(&coord, &dir, world.size);
        if __coord.is_none() {
            continue;
        }
        let _coord = __coord.unwrap();

        for resource in known_resources {
            if resource.is_streamed {
                continue;
            }
            let theirs = world.get_unit_resource_at(&_coord, resource.id as usize);
            let ours = world.get_unit_resource_at(&coord, resource.id as usize);
            let spread = ours - theirs;

            // resources only flow from the haves to the havenots
            if spread > 0 {
                let delta = spread / diffusion_factor;
                our_delta[resource.id as usize] = our_delta[resource.id as usize] - delta;
                their_deltas[i][resource.id as usize] =
                    their_deltas[i][resource.id as usize] + delta;
            };
        }
    }

    result
}

pub fn linear_diff_for_pos(coord: Coord, world: &World, chemistry: &Arc<Box<dyn Chemistry>>) {}

mod tests {
    use crate::simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
    };

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();

        let mut sim = SimulationBuilder::default()
            .size((5, 5))
            .chemistry(chemistry)
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())], // TODO: use UnitEntryBuilder
            })
            .to_simulation();

        sim.world.seed_unit_at(
            &(1, 0),
            &sim.unit_manifest.units[0].info,
            None,
            &sim.chemistry,
        );
        sim.world.seed_unit_at(
            &(3, 0),
            &sim.unit_manifest.units[0].info,
            None,
            &sim.chemistry,
        );

        // cheese
        sim.world.set_unit_resource_at(&(1, 0), 0, 20);
        sim.world.set_unit_resource_at(&(1, 0), 0, 10);
        sim.world.set_unit_resource_at(&(3, 0), 0, 5);

        //assert!(true == false);
        //calculate_linear_diff_transits(sim.world)
    }
}

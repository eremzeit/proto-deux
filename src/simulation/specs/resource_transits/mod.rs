use chemistry::{BaseChemistry, CheeseChemistry, Chemistry};
use simulation::common::*;
use simulation::config::SimulationConfig;
use simulation::iterators::*;
use simulation::specs::{SimulationSpec, SpecContext};
use simulation::unit::{add_resources_to, UnitAttributes, UnitResources};
use std::sync::Arc;
use typemap::{CloneMap, Key};
use util::text_grid::{TextGridOptions};
use util::{*};
use simulation::common::CoordIterator;


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
pub fn calculate_linear_diff_transits(coord: Coord, world: &World, chemistry: &Arc<Box<dyn Chemistry>>) -> Vec<(Coord, UnitResources)> {
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
  let directions = vec![GridDirection::Up, GridDirection::Right, GridDirection::Down, GridDirection::Left];

  for i in 0..directions.len()  {
    let dir = directions[i].clone();
    let __coord = coord_by_direction_offset(&coord, &dir, world.size);
    if __coord.is_none() { continue; }
    let _coord = __coord.unwrap();
    
    for resource in known_resources {
      if resource.is_streamed { continue; }
      let theirs = world.get_unit_resource_at(&_coord, resource.id as usize);
      let ours = world.get_unit_resource_at(&coord, resource.id as usize);
      let spread = ours - theirs;
      
      // resources only flow from the haves to the havenots
      if spread > 0 {
        let delta = spread / diffusion_factor;
        our_delta[resource.id as usize] = our_delta[resource.id as usize] - delta;
        their_deltas[i][resource.id as usize] = their_deltas[i][resource.id as usize] + delta;
      };
    }
  }


  result
}

pub fn linear_diff_for_pos(coord: Coord, world: &World, chemistry: &Arc<Box<dyn Chemistry>>) {

}

pub struct ResourceTransits  {

}

impl SimulationSpec for ResourceTransits {
  fn on_tick(&mut self, sim: &mut SimCell, context: &SpecContext) {

  }

  fn get_name(&self) -> String {
    "ResourceTransits".to_string()
  }
}

mod tests {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  fn test() {
        let mut sim = SimulationBuilder::default()
            .size((5, 5))
            .chemistry(CheeseChemistry::construct())
            .headless(true)
            .specs(vec![Box::new(PlaceUnits {
                method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
            })])
            .unit_manifest(UnitManifest { units: vec![UnitEntry::new("main", EmptyPhenotype::construct())] })
            .to_simulation();

        sim.world.seed_unit_at(&(1,0), &sim.unit_manifest.units[0].info, None, &sim.chemistry);
        sim.world.seed_unit_at(&(3,0), &sim.unit_manifest.units[0].info, None, &sim.chemistry);
          
        // cheese
        sim.world.set_unit_resource_at(&(1,0), 0, 20);
        sim.world.set_unit_resource_at(&(1,0), 0, 10);
        sim.world.set_unit_resource_at(&(3,0), 0, 5);

        //assert!(true == false);
        //calculate_linear_diff_transits(sim.world)
  }
}


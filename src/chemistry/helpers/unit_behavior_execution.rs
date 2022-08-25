use crate::chemistry::reactions::execute_reaction;
use crate::chemistry::variants::{CheeseChemistry, NanobotsChemistry};
use crate::chemistry::Chemistry;
use crate::simulation::common::CoordIterator;
use crate::simulation::common::*;
use crate::simulation::config::SimulationConfig;
use crate::simulation::iterators::*;
use crate::simulation::unit::{add_resources_to, UnitAttributes, UnitResources};
use crate::simulation::unit_entry::UnitEntryId;
use crate::util::text_grid::TextGridOptions;
use crate::util::*;
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use typemap::{CloneMap, Key};

pub fn behavior_execution(sim: &mut SimCell) {
    //let mut rng = rand::thread_rng();

    for coord in CoordIterator::new(sim.world.size) {
        // let center_distance_x = ((sim.world.size.0 / 2) as i64 - coord.0 as i64).abs();
        // let center_distance_y = ((sim.world.size.1 / 2) as i64 - coord.1 as i64).abs();
        // let location_odds = (center_distance_x as f64 / sim.world.size.0 as f64
        //     + center_distance_y as f64 / center_distance_y as f64 / sim.world.size.1 as f64)
        //     / 2.0;

        // if rng.gen_range(0.0..1.0) > location_odds {
        //     continue;
        // }

        let maybe_unit = sim.world.get_unit_at(&coord);
        if maybe_unit.is_none() {
            continue;
        }

        let unit = maybe_unit.unwrap();
        if unit.last_update_tick >= sim.world.tick {
            continue;
        }
        //println!("coord: {:?}", coord);

        let mut entry_id: UnitEntryId = unit.entry_id;

        // write this before the unit could potentially change coordinates
        // use tick as the update tick, with 0 implying "no tick yet"
        sim.world.set_unit_last_update_tick(&coord, sim.world.tick);

        let entry = &sim.unit_manifest.units[entry_id];

        let result =
            entry
                .behavior
                .get_behavior(&coord, &sim.attributes, &sim.world, sim.chemistry);

        // chemistry.consume_execution_points(result.consumed_execution_points);

        sim.chemistry.execute_unit_reaction(sim, &coord, &result);

        // for i in 0..result.reactions.len().min(1) {
        //     let reaction_call = result.reactions[i];
        //     let reaction_def = &sim.chemistry.get_manifest().reactions[reaction_call.0 as usize];

        //     // println!("EXECUTING REACTION: {}", reaction_def.key);
        //     execute_reaction(
        //         sim,
        //         &coord,
        //         &reaction_def,
        //         sim.chemistry,
        //         sim.unit_manifest,
        //         reaction_call,
        //     );
        // }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test() {}
}

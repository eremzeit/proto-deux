use crate::chemistry::variants::CheeseChemistry;
use crate::simulation::common::*;
use crate::simulation::config::SimulationConfig;
use crate::simulation::iterators::*;
use crate::simulation::unit::{add_resources_to, UnitAttributes, UnitResources};
use crate::util::text_grid::TextGridOptions;
use crate::{fixtures, simulation::common::helpers::place_units::PlaceUnitsMethod};
use std::sync::Arc;

use crate::simulation::common::CoordIterator;

#[derive(Clone)]
pub enum StoredResourceAllocationMethod {
    Every,
    //Interval(u32)
}

#[derive(Clone)]
pub struct ResourceAllocation {
    pub stored_method: StoredResourceAllocationMethod,
}

pub fn allocate_stored_resources<'a>(
    sim: &'a mut SimCell,
    unit_manifest: &UnitManifest,
    stored_method: &StoredResourceAllocationMethod,
) {
    match stored_method {
        StoredResourceAllocationMethod::Every => {
            allocation_method_every(sim, unit_manifest);
        }
    }
}

// pub fn allocate_streamed_resources(
//     world: &mut World,
//     sim_config: &SimulationConfig,
// ) {
//     for coord in CoordIterator::new(sim_config.size) {
//         let resources = chemistry.get_base_streamed_resource_allocation(world, &coord);

//         if world.has_unit_at(&coord) {
//             world.set_some_unit_resources_at(&coord, &resources, chemistry);
//         }
//     }
// }

pub fn allocation_method_every<'a>(sim: &'a mut SimCell, unit_manifest: &UnitManifest) {
    let chemistry = sim.chemistry;
    for coord in CoordIterator::new(sim.world.size) {
        if !sim.world.has_unit_at(&coord) {
            continue;
        }
        chemistry.allocate_unit_resources(&coord, sim);

        // continue;
        // match sim.world.get_unit_at(&coord) {
        //     Some(unit) => {
        //         let entry_id = unit.entry_id;
        //         let unit_entry = &unit_manifest.units[entry_id].info;

        //         //AOEU
        //         let next_resources =
        //             chemistry.get_next_unit_resources(unit_entry, pos, unit, sim.world, 1);
        //         sim.world.set_unit_resources_at(&coord, next_resources);
        //     }

        //     _ => {}
        // };
    }
}

mod tests {
    use crate::simulation::common::builder::ChemistryBuilder;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_stored_resource_allocation() {
        let chemistry = ChemistryBuilder::with_key("foo").build();
        let manifest = chemistry.get_manifest().clone();

        let mut sim = SimulationBuilder::default()
            .chemistry(chemistry)
            .size((5, 5))
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(2, 0)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntryBuilder::with_species_name("main")
                    .behavior(NullBehavior::construct())
                    .build(&manifest)],
            })
            .to_simulation();

        let is_foo_position = sim
            .chemistry
            .get_manifest()
            .position_attribute_by_key("is_foo_position")
            .id as usize;

        sim.world.set_pos_attribute_at(
            &(2, 0),
            is_foo_position,
            PositionAttributeValue::Bool(true),
        );

        assert_eq!(sim.world.has_unit_at(&(2, 0)), true);

        let id = sim
            .chemistry
            .get_manifest()
            .unit_resource_by_key(&"foo_stored_resource")
            .id;
        assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 0);

        sim.tick();
        assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 20);

        sim.tick();
        assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 40);
    }
}

use chemistry::{BaseChemistry, CheeseChemistry, Chemistry};
use simulation::common::*;
use simulation::config::SimulationConfig;
use simulation::iterators::*;
use simulation::specs::{SimulationSpec, SpecContext};
use simulation::unit::{add_resources_to, UnitAttributes, UnitResources};
use std::sync::Arc;
use typemap::{CloneMap, Key};
use util::text_grid::TextGridOptions;

use simulation::common::CoordIterator;

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
  chemistry: &ChemistryInstance,
  stored_method: &StoredResourceAllocationMethod,
) {
  match stored_method {
    StoredResourceAllocationMethod::Every => {
      allocation_method_every(sim, unit_manifest, chemistry);
    }
  }
}

pub fn allocate_streamed_resources(
  world: &mut World,
  sim_config: &SimulationConfig,
  chemistry: &ChemistryInstance,
) {
  for coord in CoordIterator::new(sim_config.size) {
    let resources = chemistry.get_base_streamed_resource_allocation(world, &coord);

    if world.has_unit_at(&coord) {
      world.set_some_unit_resources_at(&coord, &resources, chemistry);
    }
  }
}

pub fn allocation_method_every<'a>(
  sim: &'a mut SimCell,
  unit_manifest: &UnitManifest,
  chemistry: &ChemistryInstance,
) {
  for coord in CoordIterator::new(sim.world.size) {
    let pos = sim.world.get_position_at(&coord).unwrap();
    //println!("iterating coord: {:?}", coord);

    match sim.world.get_unit_at(&coord) {
      Some(unit) => {
        let entry_id = unit.entry_id;
        let unit_entry = &unit_manifest.units[entry_id].info;

        let next_resources = chemistry.get_next_unit_resources(unit_entry, pos, unit, sim.world, 1);
        sim.world.set_unit_resources_at(&coord, next_resources);
      }

      _ => {}
    };
  }
}

impl SimulationSpec for ResourceAllocation {
  fn on_tick(&mut self, sim: &mut SimCell, context: &SpecContext) {
    allocate_stored_resources(sim, sim.unit_manifest, sim.chemistry, &self.stored_method);

    // streamed is handled during stored.  TODO rename
    //allocate_streamed_resources(&mut world, sim_config, &chemistry);
  }

  fn get_name(&self) -> String {
    "ResourceAllocation".to_string()
  }
}

mod tests {
  #[allow(unused_imports)]
  use super::*;

  #[test]
  fn test_stored_resource_allocation() {
    let mut sim = SimulationBuilder::default()
      .size((5, 5))
      .chemistry(CheeseChemistry::construct())
      .headless(true)
      .specs(vec![
        Box::new(PlaceUnits {
          method: PlaceUnitsMethod::ManualSingleEntry {
            attributes: None,
            coords: vec![(2, 0)],
          },
        }),
        Box::new(ResourceAllocation {
          stored_method: StoredResourceAllocationMethod::Every,
        }),
      ])
      .unit_manifest(UnitManifest {
        units: vec![UnitEntry::new("main", EmptyPhenotype::construct())],
      })
      .to_simulation();

    let id_is_cheese_source = sim
      .chemistry
      .get_manifest()
      .position_attribute_by_key("is_cheese_source")
      .id as usize;
    let id_is_air_source = sim
      .chemistry
      .get_manifest()
      .position_attribute_by_key("is_air_source")
      .id as usize;
    sim.world.set_pos_attribute_at(
      &(2, 0),
      id_is_cheese_source,
      PositionAttributeValue::Bool(true),
    );
    sim.world.set_pos_attribute_at(
      &(2, 0),
      id_is_air_source,
      PositionAttributeValue::Bool(true),
    );
    assert_eq!(sim.world.has_unit_at(&(2, 0)), true);

    let id = sim
      .chemistry
      .get_manifest()
      .unit_resource_by_key(&"cheese")
      .id;
    assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 0);

    sim.tick();
    assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 20);

    sim.tick();
    assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 40);
  }
  use tests::fixtures;

  #[test]
  fn test_streamed_resource_allocation() {
    let mut sim = fixtures::default_base(Some(vec![
      Box::new(PlaceUnits {
        method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
      }),
      Box::new(ResourceAllocation {
        stored_method: StoredResourceAllocationMethod::Every,
      }),
    ]));

    sim.init();

    let id = sim
      .chemistry
      .get_manifest()
      .position_attribute_by_key(&"is_air_source")
      .id;
    sim
      .world
      .set_pos_attribute_at(&(2, 0), id, PositionAttributeValue::Bool(true));

    assert_eq!(sim.world.has_unit_at(&(2, 0)), true);
    let id = sim.chemistry.get_manifest().unit_resource_by_key(&"air").id;
    assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 0);

    sim.tick();
    assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 10);

    sim.tick();
    assert_eq!(sim.world.get_unit_resource_at(&(2, 0), id as usize), 10);
  }
}

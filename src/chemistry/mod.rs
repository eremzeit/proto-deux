pub mod actions;
pub mod helpers;
pub mod manifest;
pub mod properties;
pub mod reactions;
pub mod variants;

use self::helpers::place_units::place_units;
use self::helpers::place_units::PlaceUnitsMethod;
use self::properties::*;
use self::reactions::*;
use self::variants::LeverChemistry;
use crate::chemistry::actions::{
    default_actions, ActionDefinition, ActionParam, ActionParamType, ActionSet,
};
use crate::simulation::common::*;
use crate::simulation::SimulationAttributes;
use crate::util::{grid_direction_from_num, Coord};
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Instant;

pub use crate::chemistry::manifest::*;

pub type ReactionId = u8;
pub type ChemistryInstance = Box<dyn Chemistry>;

/* used to pass values from the phenotype to the action execution
 * to replace placeholders */
pub type ActionArgValue = u32;
pub fn get_chemistry_by_key(key: &str, place_units_method: PlaceUnitsMethod) -> Box<dyn Chemistry> {
    match key {
        "cheese" => CheeseChemistry::construct(place_units_method),
        "lever" => LeverChemistry::construct(place_units_method),
        _ => CheeseChemistry::construct(place_units_method),
    }
}

pub trait Chemistry {
    fn init_manifest(&mut self) {
        let mut manifest = self.get_manifest_mut();
        manifest.normalize_manifest();
    }

    fn get_unit_placement(&self) -> PlaceUnitsMethod;

    fn get_manifest(&self) -> &ChemistryManifest;
    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest;

    fn get_key(&self) -> String;
    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue>;
    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue>;
    fn get_next_unit_resources(
        &self,
        entry: &UnitEntryData,
        pos: &Position,
        unit: &Unit,
        world: &World,
        tick_multiplier: u32,
    ) -> UnitResources; /* {
                            self.get_manifest().empty_unit_resources()
                        }*/

    fn get_default_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        self.get_manifest().empty_unit_attributes()
    }
    fn get_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        if entry.default_attributes.is_some() {
            entry.default_attributes.as_ref().unwrap().clone()
        } else {
            self.get_default_unit_seed_attributes(world, coord, entry)
        }
    }

    fn get_unit_seed_stored_resource_amounts(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitResources {
        if entry.default_resources.is_some() {
            entry.default_resources.as_ref().unwrap().clone()
        } else {
            self.get_manifest().empty_unit_resources()
        }
    }

    fn get_base_stored_resource_allocation(
        &self,
        world: &mut World,
        coord: &Coord,
    ) -> SomeUnitResources {
        vec![]
    }

    fn get_base_streamed_resource_allocation(
        &self,
        world: &mut World,
        coord: &Coord,
    ) -> SomeUnitResources {
        vec![]
    }

    fn on_simulation_init(&self, sim: &mut SimCell) {
        self.init_pos_properties(&mut sim.world);
        self.init_world_custom(&mut sim.world);
        self.init_units(sim);
    }
    fn on_simulation_tick(&self, sim: &mut SimCell);
    fn on_simulation_finish(&self, sim: &mut SimCell);

    fn init_world_custom(&self, world: &mut World) {}
    fn init_units(&self, sim: &mut SimCell) {
        place_units(sim, &self.get_unit_placement());
    }

    fn init_pos_properties(&self, world: &mut World) {
        for coord in CoordIterator::new(world.size.clone()) {
            let pos_attributes = self.get_manifest().empty_position_attributes();
            world.set_pos_attributes_at(&coord, pos_attributes);
        }
    }

    // fn init_unit_properties(&self, world: &mut World) {
    //     for coord in CoordIterator::new(world.size.clone()) {
    //         if world.has_unit_at(&coord) {
    //             println!("updating properties at: {:?}", coord);
    //             let unit_resources = self.get_manifest().empty_unit_resources();
    //             world.set_unit_resources_at(&coord, unit_resources);
    //             //println!("resources: {:?}, {:?}", coord, unit_resources);
    //             world.set_unit_attributes_at(&coord, self.get_manifest().empty_unit_attributes());
    //         }
    //     }
    // }

    /* TODO: remove the concept of specs, which were meant to be a dynamic list of behaviors that could
    be configured on the fly independent of the chemistry.
        We already have the concept of chemistry.on_init().  Lets just add the concept of chemistry.on_tick().

        We can still have behaviors called specs defined outside a chemistry that can be shared across chemistries.
        The order of specs will be hard-coded on a per-chemistry basis.
        If we want to configure the behavior of those behaviors on a per-chemistry basis we can pass
        in a configuration object into the chemistry itself.  The configuration object itself can be clonable.
        The chemistry won't be cloneable because it is used as a trait object by the simulation struct.
    */
    // fn construct_specs(
    //     &self,
    //     unit_placement: &PlaceUnitsMethod,
    // ) -> Vec<std::boxed::Box<dyn SimulationSpec>>;
}

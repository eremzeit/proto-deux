pub mod actions;
pub mod cheese;
pub mod lever;
pub mod manifest;
pub mod nanobots;
pub mod properties;
pub mod reactions;
pub mod simple;

use self::properties::*;
use self::reactions::*;
use chemistry::actions::{
    default_actions, ActionDefinition, ActionParam, ActionParamType, ActionSet,
};
use simulation::common::*;
use simulation::specs::place_units::{PlaceUnits, PlaceUnitsMethod};
use simulation::specs::SimulationSpec;
use simulation::SimulationAttributes;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Instant;
use util::{grid_direction_from_num, Coord};

pub use chemistry::cheese::CheeseChemistry;
pub use chemistry::lever::LeverChemistry;
pub use chemistry::manifest::*;
pub use chemistry::nanobots::NanobotsChemistry;

pub type ReactionId = u8;
pub type ChemistryInstance = Box<dyn Chemistry>;

/* used to pass values from the phenotype to the action execution
 * to replace placeholders */
pub type ActionArgValue = u32;
pub fn get_chemistry_by_key(key: &str) -> Box<dyn Chemistry> {
    match key {
        "cheese" => CheeseChemistry::construct(),
        "lever" => LeverChemistry::construct(),
        _ => CheeseChemistry::construct(),
    }
}

pub trait Chemistry {
    fn init_manifest(&mut self) {
        let mut manifest = self.get_manifest_mut();
        manifest.normalize_manifest();
    }

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

    fn init_pos_properties(&self, world: &mut World) {
        for coord in CoordIterator::new(world.size.clone()) {
            let pos_attributes = self.get_manifest().empty_position_attributes();
            world.set_pos_attributes_at(&coord, pos_attributes);
        }
    }

    fn init_unit_properties(&self, world: &mut World) {
        for coord in CoordIterator::new(world.size.clone()) {
            if world.has_unit_at(&coord) {
                let unit_resources = self.get_manifest().empty_unit_resources();
                world.set_unit_resources_at(&coord, unit_resources);
                //println!("resources: {:?}, {:?}", coord, unit_resources);
                world.set_unit_attributes_at(&coord, self.get_manifest().empty_unit_attributes());
            }
        }
    }

    // fn init_world(&self, world: &mut World, sim_config: &SimulationConfig) {
    //   for coord in CoordIterator::new(sim_config.size) {
    //     let pos_attributes = self.get_manifest().empty_position_attributes();
    //     world.set_pos_attributes_at(&coord, pos_attributes);

    //     if world.has_unit_at(&coord) {
    //       let unit_resources = self.get_manifest().empty_unit_resources();
    //       world.set_unit_resources_at(&coord, unit_resources);
    //       //println!("resources: {:?}, {:?}", coord, unit_resources);
    //       world.set_unit_attributes_at(&coord, self.get_manifest().empty_unit_attributes());
    //     }
    //   }
    // }

    fn init_world_custom(&self, world: &mut World) {}

    // used for extreme convenience and backwards compatability
    // only.  doesn't allow for any configuration.
    //fn default_specs(&self) -> Vec<std::boxed::Box<dyn SimulationSpec>>;

    // if we ever need custom init data per each chemistry type then
    // we can convert this to accepting an enum of where each variant
    // is a named dictionary.
    fn construct_specs(
        &self,
        unit_placement: &PlaceUnitsMethod,
    ) -> Vec<std::boxed::Box<dyn SimulationSpec>>;
}

pub struct BaseChemistry {
    pub manifest: ChemistryManifest,
}

impl BaseChemistry {
    pub fn construct() -> ChemistryInstance {
        wrap_chemistry!(BaseChemistry {
            manifest: BaseChemistry::default_manifest(),
        })
    }

    pub fn default_manifest() -> ChemistryManifest {
        let reactions = vec![];

        let mut manifest = ChemistryManifest {
            simulation_attributes: vec![],
            reactions,
            action_set: default_actions(),

            position_resources: vec![],

            unit_resources: vec![UnitResourceDefinition::new("energy", false, 0)],
            unit_attributes: vec![UnitAttributeDefinition::new(
                "is_foo",
                AttributeDefinitionType::Boolean,
                0,
            )],
            position_attributes: vec![PositionAttributeDefinition::new(
                "is_rooted",
                AttributeDefinitionType::Boolean,
                0,
            )],

            unit_entry_attributes: vec![],

            all_properties: vec![],
        };
        manifest.normalize_manifest();

        manifest
    }
}

impl Chemistry for BaseChemistry {
    fn get_key(&self) -> String {
        "base".to_string()
    }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }

    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    // fn default_specs(&self) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
    //     vec![Box::new(PlaceUnits {
    //         method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
    //     })]
    // }

    fn construct_specs(
        &self,
        unit_placement: &PlaceUnitsMethod,
    ) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
        vec![
            Box::new(PlaceUnits {
                method: unit_placement.clone(),
            }),
            Box::new(PhenotypeExecution {}),
        ]
    }

    // fn default_specs(&self) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
    //     vec![Box::new(PlaceUnits {
    //         method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
    //     })]
    // }

    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue> {
        self.get_manifest().empty_simulation_attributes()
    }
    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue> {
        self.get_manifest().empty_unit_entry_attributes()
    }
    fn get_next_unit_resources(
        &self,
        entry: &UnitEntryData,
        pos: &Position,
        unit: &Unit,
        world: &World,
        tick_multiplier: u32,
    ) -> UnitResources {
        self.get_manifest().empty_unit_resources()
    }
}

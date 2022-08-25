pub mod actions;
pub mod builder;

#[macro_use]
pub mod config;
pub mod helpers;
pub mod manifest;
pub mod properties;
pub mod reactions;
pub mod variants;

use self::helpers::place_units::place_units;
use self::helpers::place_units::PlaceUnitsMethod;
use self::properties::*;
use self::reactions::*;
use self::variants::BaseChemistry;
use self::variants::LeverChemistry;
use crate::chemistry::actions::{
    default_actions, ActionDefinition, ActionParam, ActionParamType, ActionSet,
};
use crate::simulation::common::*;
use crate::simulation::SimulationAttributes;

use crate::chemistry::config::convert_configurable_to_action_param;
use crate::chemistry::config::ChemistryConfigValue;
use crate::util::{grid_direction_from_num, Coord};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Instant;

pub use crate::chemistry::manifest::*;

pub type ReactionId = u8;
pub type ChemistryInstance = Box<dyn Chemistry>;
pub type ChemistryConfiguration = HashMap<String, ChemistryConfigValue>;

/* used to pass values from the unit_behavior to the action execution
 * to replace placeholders */
pub type ActionArgValue = u32;
// pub fn get_chemistry_by_key(key: &str, config: ChemistryConfiguration) -> Rc<dyn Chemistry> {
pub fn construct_chemistry(
    key: &str,
    config: Option<ChemistryConfiguration>,
) -> Box<dyn Chemistry> {
    let _config = config.unwrap_or_default();
    match key {
        "cheese" => CheeseChemistry::construct(_config),
        "lever" => LeverChemistry::construct(_config),
        "nanobots" => NanobotsChemistry::construct(_config),
        "base" => BaseChemistry::construct(),
        _ => panic!("chemistry key not found: {}", key),
    }
}

pub trait Chemistry {
    fn init_manifest(&mut self) {
        let reactions = self.init_chemistry_action_params();
        let mut manifest = self.get_manifest_mut();
        manifest.reactions = reactions;
        manifest.normalize_manifest();
    }

    fn get_configuration(&self) -> ChemistryConfiguration;

    fn init_chemistry_action_params(&self) -> Vec<ReactionDefinition> {
        let mut manifest = self.get_manifest();
        let mut reactions = manifest.reactions.clone();
        let config = &self.get_configuration();
        for i in 0..reactions.len() {
            let mut reaction = &mut reactions[i];
            for j in 0..reaction.reagents.len() {
                let mut reagent = &mut reaction.reagents[j];

                reagent.params = reagent
                    .params
                    .clone()
                    .iter()
                    .map(|param| {
                        if let ActionParam::ChemistryArgument(key, param_type) = param.clone() {
                            let value = config.get(&key).unwrap();
                            let action_param_value =
                                convert_configurable_to_action_param(value.clone(), param_type);
                            action_param_value
                        } else {
                            param.clone()
                        }
                    })
                    .collect::<Vec<_>>();
            }
        }
        reactions
    }

    fn get_manifest(&self) -> &ChemistryManifest;
    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest;

    fn get_key(&self) -> String;
    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue>;
    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue>;
    // fn get_next_unit_resources(
    //     &self,
    //     entry: &UnitEntryData,
    //     pos: &Position,
    //     unit: &Unit,
    //     world: &World,
    //     tick_multiplier: u32,
    // ) -> UnitResources {
    //     self.get_manifest().empty_unit_resources()
    // }

    fn allocate_unit_resources(&self, coord: &Coord, sim: &mut SimCell) {}

    /**
     * Get the seed attributes used when a unit entry doesn't have seed attributes specified
     */
    fn get_default_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        self.get_manifest().empty_unit_attributes()
    }

    /**
     * Get the seed attributes used for a specifc unit entry
     */
    fn get_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        if entry.default_unit_attributes.is_some() {
            entry.default_unit_attributes.as_ref().unwrap().clone()
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

    // fn get_base_stored_resource_allocation(
    //     &self,
    //     world: &mut World,
    //     coord: &Coord,
    // ) -> SomeUnitResources {
    //     vec![]
    // }

    // fn get_base_streamed_resource_allocation(
    //     &self,
    //     world: &mut World,
    //     coord: &Coord,
    // ) -> SomeUnitResources {
    //     vec![]
    // }

    fn get_default_place_units_method(&self) -> PlaceUnitsMethod {
        PlaceUnitsMethod::SimpleDrop { attributes: None }
    }

    fn on_simulation_init(&self, sim: &mut SimCell) {
        self.init_pos_properties(&mut sim.world);
        self.init_world_custom(&mut sim.world);
        // self.init_units(sim);
    }

    fn on_simulation_tick(&self, sim: &mut SimCell);
    fn on_simulation_finish(&self, sim: &mut SimCell);

    fn init_world_custom(&self, world: &mut World) {}

    // fn init_units(&self, sim: &mut SimCell) {}

    fn init_pos_properties(&self, world: &mut World) {
        for coord in CoordIterator::new(world.size.clone()) {
            let pos_attributes = self.get_manifest().empty_position_attributes();
            world.set_pos_attributes_at(&coord, pos_attributes);
        }
    }

    fn deduct_unit_execution_points(&self, sim: &mut SimCell, unit_entry_id: usize, points: u64) {}

    fn execute_unit_reaction(&self, sim: &mut SimCell, coord: &Coord, result: &UnitBehaviorResult) {
        self.deduct_unit_execution_points(sim, 0, result.consumed_execution_points);

        // println!("behavior result: {:?}", result);
        for i in 0..result.reactions.len().min(1) {
            let reaction_call = result.reactions[i];
            let reaction_def = &sim.chemistry.get_manifest().reactions[reaction_call.0 as usize];

            execute_reaction(
                sim,
                &coord,
                &reaction_def,
                sim.chemistry,
                sim.unit_manifest,
                reaction_call,
            );
        }
    }
}

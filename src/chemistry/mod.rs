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
use self::variants::foo::FooChemistry;
use self::variants::LeverChemistry;
use crate::biology::genetic_manifest::predicates::default_operators;
use crate::biology::genetic_manifest::predicates::OperatorLibrary;
use crate::biology::sensor_manifest::CustomSensorImplementation;
use crate::biology::sensor_manifest::CustomSensorLibrary;
use crate::biology::sensor_manifest::LocalPropertySensorManifest;
use crate::chemistry::actions::{default_actions, ActionParam, ActionParamType};
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

pub struct ChemistryConfigBuilder {
    config: ChemistryConfiguration,
}

impl ChemistryConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: ChemistryConfiguration::new(),
        }
    }
    pub fn set_bool(mut self, key: &str, val: bool) -> Self {
        self.config
            .insert(key.to_string(), ChemistryConfigValue::Boolean(val));
        self
    }
    pub fn set_integer(mut self, key: &str, val: u64) -> Self {
        self.config
            .insert(key.to_string(), ChemistryConfigValue::Integer(val));
        self
    }
    pub fn set_resource_amount(mut self, key: &str, val: i32) -> Self {
        self.config
            .insert(key.to_string(), ChemistryConfigValue::ResourceAmount(val));
        self
    }
    pub fn build(self) -> ChemistryConfiguration {
        self.config
    }
}

/* used to pass values from the unit_behavior to the action execution
 * to replace placeholders */
pub type ActionArgValue = u32;
// pub fn get_chemistry_by_key(key: &str, config: ChemistryConfiguration) -> Rc<dyn Chemistry> {
pub fn construct_chemistry(
    key: &str,
    config: Option<ChemistryConfiguration>,
) -> Box<dyn Chemistry> {
    match key {
        "cheese" => CheeseChemistry::construct(config.unwrap_or(CheeseChemistry::default_config())),
        "lever" => LeverChemistry::construct(config.unwrap_or(LeverChemistry::default_config())),
        "nanobots" => {
            NanobotsChemistry::construct(config.unwrap_or(NanobotsChemistry::default_config()))
        }
        "foo" => FooChemistry::construct(config.unwrap_or(FooChemistry::default_config())),
        _ => panic!("chemistry key not found: {}", key),
    }
}

pub fn construct_chemistry_libraries(key: &str) -> ChemistryLibraries {
    match key {
        "cheese" => CheeseChemistry::get_libraries(),
        "lever" => LeverChemistry::get_libraries(),
        "nanobots" => NanobotsChemistry::get_libraries(),
        "foo" => FooChemistry::get_libraries(),
        _ => panic!("chemistry key not found: {}", key),
    }
}

/**
 * Represents the "universe" of possible implementations of actions, custom sensors
 * and operators.  Is essentially on a one to one basis with chemistry types.
 */
pub struct ChemistryLibraries {
    pub action_library: ActionLibrary,
    pub custom_sensor_library: CustomSensorLibrary,
    pub operator_library: OperatorLibrary,
}

pub trait Chemistry {
    fn construct(config: ChemistryConfiguration) -> Box<Self>
    where
        Self: Sized;

    fn fill_with_defaults(config: ChemistryConfiguration) -> ChemistryConfiguration
    where
        Self: Sized,
    {
        let mut _config = config;

        let defaults = Self::default_config();
        for key in defaults.keys() {
            _config
                .entry(key.clone())
                .or_insert(defaults.get(key).unwrap().clone());
        }

        _config
    }

    fn construct_with_default_config() -> Box<Self>
    where
        Self: Sized,
    {
        Self::construct(Self::default_config())
    }

    fn construct_manifest(config: &ChemistryConfiguration) -> ChemistryManifest
    where
        Self: Sized;

    fn default_manifest() -> ChemistryManifest
    where
        Self: Sized,
    {
        Self::construct_manifest(&Self::default_config())
    }

    fn default_config() -> ChemistryConfiguration
    where
        Self: Sized,
    {
        ChemistryConfiguration::new()
    }

    // fn get_action_definitions(&self) -> ActionDefinitionSet;

    fn custom_action_definitions() -> Vec<ActionDefinition>
    where
        Self: Sized,
    {
        vec![]
    }

    fn get_libraries() -> ChemistryLibraries
    where
        Self: Sized,
    {
        ChemistryLibraries {
            action_library: Self::construct_action_library(),
            custom_sensor_library: Self::custom_sensor_library(),
            operator_library: default_operators().operators,
        }
    }

    fn construct_action_library() -> ActionLibrary
    where
        Self: Sized,
    {
        let mut actions = default_actions();
        actions.append(&mut Self::custom_action_definitions());
        actions
    }

    fn custom_sensor_library() -> Vec<CustomSensorImplementation>
    where
        Self: Sized,
    {
        vec![]
    }

    /**
     * Specifies which local properties are to be mapped into sensors
     */
    fn default_local_property_sensor_manifest(&self) -> LocalPropertySensorManifest {
        LocalPropertySensorManifest::from_all_props(self.get_manifest().all_properties.as_slice())
    }

    fn get_default_local_property_sensor_manifest(
        all_properties: &[Property],
    ) -> LocalPropertySensorManifest
    where
        Self: Sized,
    {
        LocalPropertySensorManifest::from_all_props(all_properties)
    }

    // fn init_manifest(&mut self) {
    //     let config = &self.get_configuration();
    //     let mut manifest = self.get_manifest_mut();
    //     init_manifest(manifest, config);
    // }

    fn get_configuration(&self) -> ChemistryConfiguration;

    // fn init_chemistry_action_params__(&self) -> Vec<ReactionDefinition> {
    //     let mut manifest = self.get_manifest();
    //     let mut reactions = manifest.reactions.clone();
    //     let config = &self.get_configuration();
    //     for i in 0..reactions.len() {
    //         let mut reaction = &mut reactions[i];
    //         for j in 0..reaction.reagents.len() {
    //             let mut reagent = &mut reaction.reagents[j];

    //             reagent.params = reagent
    //                 .params
    //                 .clone()
    //                 .iter()
    //                 .map(|param| {
    //                     if let ActionParam::ChemistryArgument(key, param_type) = param.clone() {
    //                         let value = config.get(&key).unwrap();
    //                         let action_param_value =
    //                             convert_configurable_to_action_param(value.clone(), param_type);
    //                         action_param_value
    //                     } else {
    //                         param.clone()
    //                     }
    //                 })
    //                 .collect::<Vec<_>>();
    //         }
    //     }
    //     reactions
    // }

    // should this return an Rc?
    fn get_manifest(&self) -> &ChemistryManifest;
    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest;

    // fn get_key(&self) -> String;
    fn get_key() -> String
    where
        Self: Sized;

    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue> {
        self.get_manifest().empty_simulation_attributes()
    }

    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue> {
        self.get_manifest().empty_unit_entry_attributes()
    }

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

    fn custom_place_units(&self, sim: &mut SimCell) {
        panic!("Not implemented for chemistry");
    }

    fn get_default_place_units_method(&self) -> PlaceUnitsMethod {
        PlaceUnitsMethod::SimpleDrop { attributes: None }
    }

    /**
     * This runs before units are placed
     */
    fn on_simulation_init(&self, sim: &mut SimCell) {
        self.init_pos_properties(&mut sim.world);
        self.init_world_custom(&mut sim.world);
        // self.init_units(sim);
    }

    fn on_simulation_tick(&self, sim: &mut SimCell) -> bool;
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

// fn init_manifest(cm: &mut ChemistryManifest, config: &ChemistryConfiguration) {
//     let reactions = init_chemistry_action_params(cm, config);
//     cm.normalize_manifest();
// }

fn init_chemistry_action_params(cm: &mut ChemistryManifest, config: &ChemistryConfiguration) {
    let mut manifest = cm;
    let mut reactions = &mut manifest.reactions;

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
                        let value = config.get(&key).expect(&format!(
                            "Chemistry configuration variable {} is undefined",
                            key
                        ));
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
}

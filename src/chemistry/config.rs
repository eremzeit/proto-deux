use std::collections::HashMap;

use crate::chemistry::actions::ActionSet;
use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::chemistry::reactions::*;
use crate::simulation::common::helpers::place_units::place_units;
use crate::simulation::common::helpers::resource_allocation::allocate_stored_resources;
use crate::simulation::common::helpers::resource_allocation::StoredResourceAllocationMethod;
use crate::simulation::common::helpers::unit_behavior_execution::behavior_execution;
use crate::simulation::common::*;
use crate::simulation::common::*;
use crate::simulation::unit::*;
use crate::simulation::Simulation;
use crate::simulation::{
    common::{
        default_actions, helpers::place_units::PlaceUnitsMethod, ActionParam, ActionParamType,
        Chemistry, ChemistryInstance, ChemistryManifest, UnitEntryAttributeValue,
    },
    world::World,
    SimCell, SimulationAttributeValue,
};
use crate::util::Coord;

// use self::defs::ConfigurableParam;

// pub enum CheeseChemistryParamName {
//     MaxGobbleAmount,
// }

// pub enum CheeseChemistryParam {
//     MaxGobbleAmount(u32),
// }

#[macro_export]
macro_rules! def_configurable_parameters {
    ($all:tt) => {
        _def_enum_for_chemistry_param_values! {ConfigurableParam, $all}
        // _def_enum_for_chemistry_param_names! {ConfigurableParamName, $all}
    };
}

#[macro_export]
macro_rules! _def_enum_for_chemistry_param_values {
    ($enum_name:ident, [$([$param_name:ident,  $param_name2: ident, $param_type: ty]),*]) => {

		#[derive(Clone)]
        pub enum $enum_name {
            $(
                $param_name($param_type),
            )*
        }
    };
}

#[macro_export]
macro_rules! _def_enum_for_chemistry_param_names {
    ($enum_name:ident, [$([$param_name:ident,  $param_name2: ident, $param_type: ty]),*]) => {
        pub enum $enum_name {
            $(
                $param_name,
            )*
        }
    };
}

#[derive(Clone)]
pub enum ChemistryConfigValueType {
    Constant,
    Boolean,
    Direction,
}

#[derive(Clone, Debug)]
pub enum ChemistryConfigValue {
    Integer(u64),
    Boolean(bool),
    Direction(GridDirection),
}

impl ChemistryConfigValue {
    pub fn unwrap_bool(&self) -> bool {
        match self {
            Self::Boolean(x) => *x,
            _ => {
                panic!("Expected a bool but found a {:?}", self);
            }
        }
    }

    pub fn unwrap_integer(&self) -> u64 {
        match self {
            Self::Integer(x) => *x,
            _ => {
                panic!("Expected an integer but found a {:?}", self);
            }
        }
    }

    pub fn unwrap_direction(&self) -> GridDirection {
        match self {
            Self::Direction(x) => x.clone(),
            _ => {
                panic!("Expected an integer but found a {:?}", self);
            }
        }
    }
}

pub fn convert_configurable_to_action_param(
    value: ChemistryConfigValue,
    param_type: ActionParamType,
) -> ActionParam {
    match &param_type {
        ActionParamType::Boolean => ActionParam::Boolean(value.unwrap_bool()),
        ActionParamType::ConstantNum => {
            ActionParam::Constant(value.unwrap_integer().try_into().unwrap())
        }
        ActionParamType::Direction => ActionParam::Direction(value.unwrap_direction()),
        ActionParamType::UnitResourceAmount => {
            ActionParam::UnitResourceAmount(value.unwrap_integer().try_into().unwrap())
        }

        _ => {
            panic!("unsupported")
        }
    }
}

pub struct FooChemistry {
    manifest: ChemistryManifest,
    configuration: HashMap<String, ChemistryConfigValue>,
}

pub mod defs {
    use super::*;

    // def_configurable_parameters! {
    //     [
    //     [MoveCost, move_cost, u32]
    //     // ["new_unit_cost", Number]
    //     // ["max_gobble_amount", Number]
    //     // ["max_cheese_unit_storage", Number]
    //     ]
    // }

    def_unit_entry_attributes! {[ ]}

    def_simulation_attributes! {[
    ]}

    def_unit_attributes! {[
    ]}

    def_position_attributes! {[
    ]}

    def_position_resources! {[
    ]}

    def_unit_resources! {[
       [foo_resource, false]
    ]}

    def_reactions! {
        reaction!("move_unit",
            reagent!("offset_unit_resource",
                param_value!(UnitResourceKey, "foo_resource"),
                chemistry_arg!(move_unit_cost, UnitResourceAmount),
                param_value!(Boolean, false),
            ),
            reagent!("move_unit",
                unit_behavior_arg!(Direction)
            ),
        ),
    }
}

impl FooChemistry {
    pub fn construct(mut config: ChemistryConfiguration) -> ChemistryInstance {
        if !config.contains_key("move_unit_cost") {
            config.insert(
                "move_unit_cost".to_string(),
                ChemistryConfigValue::Integer(100),
            );
        }

        let mut chemistry = FooChemistry {
            manifest: FooChemistry::default_manifest(),
            configuration: config,
        };

        chemistry.init_manifest();

        wrap_chemistry!(chemistry)
    }

    pub fn default_manifest() -> ChemistryManifest {
        let mut manifest = ChemistryManifest {
            all_properties: vec![],
            simulation_attributes: defs::SimulationAttributesLookup::make_defs(),
            unit_entry_attributes: defs::UnitEntryAttributesLookup::make_defs(),
            action_set: default_actions(),
            unit_resources: defs::UnitResourcesLookup::make_defs(),
            unit_attributes: defs::UnitAttributesLookup::make_defs(),
            position_attributes: defs::PositionAttributesLookup::make_defs(),
            position_resources: defs::PositionResourcesLookup::make_defs(),
            reactions: defs::get_reactions(),
        };

        manifest.normalize_manifest();

        manifest
    }
}

impl Chemistry for FooChemistry {
    fn get_key(&self) -> String {
        "foo".to_string()
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    // fn get_unit_placement(&self) -> PlaceUnitsMethod {
    //     PlaceUnitsMethod::Default
    // }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }
    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    fn on_simulation_init(&self, sim: &mut SimCell) {}

    fn on_simulation_tick(&self, sim: &mut SimCell) {}

    fn on_simulation_finish(&self, sim: &mut SimCell) {}

    fn init_world_custom(&self, world: &mut World) {}

    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue> {
        self.get_manifest().empty_simulation_attributes()
    }

    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue> {
        self.get_manifest().empty_unit_entry_attributes()
    }

    fn get_default_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        self.get_manifest().unit_attributes_of(vec![(
            "rolling_consumption",
            UnitAttributeValue::Integer(0),
        )])
    }
}

pub mod tests {
    use crate::simulation::common::config::ChemistryConfigValue;

    use super::{ChemistryConfiguration, FooChemistry};

    #[test]
    pub fn test_compilation() {
        let mut config = ChemistryConfiguration::new();
        config.insert(
            "move_unit_cost".to_string(),
            ChemistryConfigValue::Integer(1337),
        );

        let chemistry = FooChemistry::construct(config);
        let manifest = chemistry.get_manifest();
        let move_unit_reaction = manifest
            .reactions
            .iter()
            .find(|r| r.key == "move_unit")
            .unwrap()
            .clone();

        let offset_resource_reagent = move_unit_reaction
            .reagents
            .iter()
            .find(|r| r.action_key == "offset_unit_resource")
            .unwrap()
            .clone();

        println!("param: {:?}", offset_resource_reagent.params[1]);
        assert_eq!(
            offset_resource_reagent.params[1].to_unit_resource_amount(),
            1337
        );
        // assert_eq!(12, 1337);
    }
}

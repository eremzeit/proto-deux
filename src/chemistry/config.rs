use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// use crate::chemistry::actions::CompiledActionManifest;
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChemistryConfigValue {
    ResourceAmount(ResourceAmount),
    Integer(u64),
    Boolean(bool),
    Direction(GridDirection),
}

impl ChemistryConfigValue {
    pub fn unwrap_resource_amount(&self) -> ResourceAmount {
        match self {
            Self::ResourceAmount(x) => *x,
            _ => {
                panic!("Expected a resource amount but found a {:?}", self);
            }
        }
    }
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
            ActionParam::UnitResourceAmount(value.unwrap_resource_amount().try_into().unwrap())
        }

        _ => {
            panic!("unsupported")
        }
    }
}

pub mod tests {
    use super::variants::foo::defs;
    use super::{ChemistryConfiguration, FooChemistry};
    use crate::simulation::common::{config::ChemistryConfigValue, Chemistry};

    #[test]
    pub fn test_compilation() {
        let mut config = ChemistryConfiguration::new();
        config.insert(
            "new_unit_cost".to_string(),
            ChemistryConfigValue::Integer(1337),
        );

        println!("UNPROCESSED raw reactions: {:?}\n\n", defs::get_reactions());

        let chemistry = FooChemistry::construct(config);
        let manifest = chemistry.get_manifest();
        let new_unit_reaction = manifest
            .reactions
            .iter()
            .find(|r| r.key == "new_unit")
            .unwrap()
            .clone();

        let offset_resource_reagent = new_unit_reaction
            .reagents
            .iter()
            .find(|r| r.action_key == "offset_unit_resource")
            .unwrap()
            .clone();

        println!("reagent: {:?}", offset_resource_reagent);
        println!("param: {:?}", offset_resource_reagent.params[1]);
        assert_eq!(
            offset_resource_reagent.params[1].to_unit_resource_amount(),
            1337
        );
        // assert_eq!(12, 1337);
    }
}

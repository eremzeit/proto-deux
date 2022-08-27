use serde::{Deserialize, Serialize};

use crate::biology::genetic_manifest::predicates::OperatorParam;
use crate::simulation::common::{
    ChemistryManifest, PositionAttributeIndex, PositionAttributeValue, PositionResourceAmount,
    PositionResourceIndex, SimulationAttributeIndex, SimulationAttributeValue, UnitAttributeIndex,
    UnitAttributeValue, UnitResourceAmount, UnitResourceIndex,
};

pub use crate::chemistry::variants::cheese::CheeseChemistry;
pub use crate::chemistry::variants::nanobots::NanobotsChemistry;

#[derive(Clone)]
pub enum PropertyValue {
    UnitAttribute(UnitAttributeValue),
    PositionAttribute(PositionAttributeValue),
    UnitResource(UnitResourceAmount),
    PositionResource(PositionResourceAmount),
    SimulationAttribute(SimulationAttributeValue),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PropertyId {
    UnitAttributeId(UnitAttributeIndex),
    PositionAttributeId(PositionAttributeIndex),
    UnitResourceId(UnitResourceIndex),
    PositionResourceId(PositionResourceIndex),
    SimulationAttributeId(SimulationAttributeIndex),
}

impl PropertyId {
    pub fn as_operator_param(&self) -> OperatorParam {
        match &self {
            PropertyId::PositionAttributeId(id) => *id as OperatorParam,
            PropertyId::PositionResourceId(id) => *id as OperatorParam,
            PropertyId::UnitAttributeId(id) => *id as OperatorParam,
            PropertyId::UnitResourceId(id) => *id as OperatorParam,
            PropertyId::SimulationAttributeId(id) => *id as OperatorParam,
        }
    }

    pub fn as_key(&self, chemistry_manifest: &ChemistryManifest) -> String {
        match &self {
            PropertyId::PositionAttributeId(id) => {
                chemistry_manifest.position_attributes[*id].key.to_string()
            }
            PropertyId::PositionResourceId(id) => {
                chemistry_manifest.position_resources[*id].key.to_string()
            }
            PropertyId::UnitAttributeId(id) => {
                chemistry_manifest.unit_attributes[*id].key.to_string()
            }
            PropertyId::UnitResourceId(id) => {
                chemistry_manifest.unit_resources[*id].key.to_string()
            }
            PropertyId::SimulationAttributeId(id) => chemistry_manifest.simulation_attributes[*id]
                .key
                .to_string(),
        }
    }

    pub fn coerce_to_sim_attribute_id(&self) -> SimulationAttributeIndex {
        match &self {
            PropertyId::SimulationAttributeId(id) => *id,
            _ => {
                panic!("aoeu")
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Property {
    pub key: String,
    pub long_key: String,
    pub property_id: PropertyId,
    pub id: usize,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub key: String,
    pub id: ResourceIndex,
    pub is_streamed: bool,
}

pub type PositionResourceDefinition = ResourceDefinition;
pub type UnitResourceDefinition = ResourceDefinition;

impl ResourceDefinition {
    pub fn new(key: &str, is_streamed: bool, id: ResourceIndex) -> ResourceDefinition {
        ResourceDefinition {
            id,
            key: key.to_string(),
            is_streamed,
        }
    }
}

pub type ResourceIndex = usize;
pub type ResourceAmount = i32;
pub type AttributeIndex = usize;
pub type RawPropertyId = usize;

#[derive(Clone, Serialize, Deserialize)]
pub enum AttributeDefinitionType {
    Number,
    Str,
    Boolean,
}

pub type AttributeInteger = i32;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    Bool(bool),
    Integer(AttributeInteger),
    String(String),
    Nil,
}

use std::fmt::{Debug, Formatter, Result};
use std::ops::AddAssign;

impl AttributeValue {
    pub fn unwrap_bool(&self) -> bool {
        match self {
            Self::Bool(x) => *x,
            Self::Nil => false,
            _ => {
                panic!("Expected a bool but found a {:?}", self);
            }
        }
    }

    pub fn unwrap_integer(&self) -> AttributeInteger {
        match self {
            Self::Integer(x) => *x,
            Self::Nil => 0,
            _ => {
                panic!("Expected an integer but found a {:?}", self);
            }
        }
    }

    pub fn coerce_unwrap_to_integer(&self) -> AttributeInteger {
        match self {
            Self::Bool(x) => {
                if *x {
                    1
                } else {
                    0
                }
            }
            Self::Integer(x) => *x,
            Self::Nil => 0,
            _ => 0,
        }
    }
}

impl AddAssign for AttributeValue {
    fn add_assign(&mut self, other: Self) {
        AttributeValue::Integer(self.coerce_unwrap_to_integer() + other.coerce_unwrap_to_integer());
    }
}

impl Debug for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AttributeValue::Bool(boolean) => write!(f, "{}", boolean),
            AttributeValue::Integer(int) => write!(f, "{}", int),
            AttributeValue::String(string) => write!(f, "{}", string),
            AttributeValue::Nil => write!(f, "Nil"),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub key: String,
    pub value_type: AttributeDefinitionType,
    pub id: AttributeIndex,
}

pub type UnitAttributeDefinition = AttributeDefinition;
pub type PositionAttributeDefinition = AttributeDefinition;
pub type SimulationAttributeDefinition = AttributeDefinition;
pub type UnitEntryAttributeDefinition = AttributeDefinition;

impl AttributeDefinition {
    pub fn new(
        key: &str,
        value_type: AttributeDefinitionType,
        id: AttributeIndex,
    ) -> AttributeDefinition {
        AttributeDefinition {
            id: 0,
            key: key.to_string(),
            value_type,
        }
    }
}

impl Debug for AttributeDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "AttributeDefinition {:?}", (self.id, &self.key))
    }
}

#[derive(Clone, Debug)]
pub struct ResourceTabulation {
    pub last_update_tick: u64,
    pub offset_per_tick: i32,
    pub last_amount: ResourceAmount,
    pub max_amount: Option<ResourceAmount>,
}

impl ResourceTabulation {
    pub fn new() -> Self {
        ResourceTabulation {
            last_update_tick: 0,
            offset_per_tick: 0,
            last_amount: 0,
            max_amount: None,
        }
    }

    pub fn get_current_amount(&self, current_tick: u64) -> ResourceAmount {
        let tick_diff = (current_tick - self.last_update_tick) as i32;
        let amount = (tick_diff * self.offset_per_tick + self.last_amount) as ResourceAmount;

        if let Some(max) = self.max_amount {
            if amount > max {
                max
            } else {
                amount
            }
        } else {
            amount
        }
    }

    pub fn update(&mut self, current_tick: u64, amount: ResourceAmount) {
        self.last_amount = amount;
        self.last_update_tick = current_tick;
    }
}

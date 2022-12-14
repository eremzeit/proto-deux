use crate::chemistry::properties::{AttributeIndex, AttributeValue, ResourceAmount, ResourceIndex};
use crate::chemistry::{Chemistry, ChemistryManifest};
use crate::simulation::common::*;
use crate::simulation::unit::*;
use crate::util::Coord;
use std::collections::HashMap;

pub type PositionAttributeIndex = AttributeIndex;
pub type PositionResourceIndex = ResourceIndex;
pub type PositionResourceAmount = ResourceAmount;
pub type PositionAttributeValue = AttributeValue;
pub type PositionAttributes = Vec<PositionAttributeValue>;

pub type PositionResourceTabulation = ResourceTabulation;
pub type PositionResourceTabulations = Vec<PositionResourceTabulation>;
pub type PositionResources = Vec<PositionResourceAmount>;

#[derive(Clone)]
pub struct Position {
    pub attributes: PositionAttributes,
    pub resources: PositionResourceTabulations,
    pub unit: Option<Unit>,
    pub coord: Coord,
}

impl Position {
    pub fn set_unit(&mut self, unit: Option<Unit>) {
        self.unit = unit;
    }

    pub fn get_attribute(&self, attr_idx: PositionAttributeIndex) -> PositionAttributeValue {
        if attr_idx < self.attributes.len() {
            self.attributes[attr_idx].clone()
        } else {
            PositionAttributeValue::Nil
        }
    }

    pub fn get_resource(
        &self,
        resource_idx: PositionResourceIndex,
        current_tick: u64,
    ) -> PositionResourceAmount {
        if resource_idx < self.resources.len() {
            return self.resources[resource_idx]
                .get_current_amount(current_tick)
                .clone() as PositionResourceAmount;
        }

        panic!("invalid resource_idx: {}", resource_idx);
    }

    pub fn set_resource(
        &mut self,
        resource_idx: PositionResourceIndex,
        val: PositionResourceAmount,
        tick: u64,
    ) {
        self.resources[resource_idx].update(tick, val);
    }

    pub fn set_attribute(&mut self, attr_idx: PositionAttributeIndex, val: PositionAttributeValue) {
        self.attributes[attr_idx] = val;
    }

    pub fn set_attributes(&mut self, attributes: PositionAttributes) {
        self.attributes = attributes;
    }

    pub fn has_unit(&self) -> bool {
        return self.unit.is_some();
    }

    pub fn get_unit_attribute(&self, attr_idx: UnitAttributeIndex) -> UnitAttributeValue {
        if let Some(u) = &self.unit {
            return u.get_attribute(attr_idx).clone();
        }

        panic!["Unit does not exist for position at {:?}", self.coord];
    }
    pub fn get_unit_resource(&self, resource_idx: UnitResourceIndex) -> UnitResourceAmount {
        if let Some(u) = &self.unit {
            return u.get_resource(resource_idx);
        }

        panic!["Unit does not exist for position at {:?}", self.coord];
    }

    pub fn set_unit_attribute(&mut self, attr_idx: UnitAttributeIndex, value: UnitAttributeValue) {
        let mut unit = self.unit.as_mut();
        if let Some(u) = unit {
            u.set_attribute(attr_idx, value);
        }
    }
    pub fn set_unit_attributes(&mut self, attributes: UnitAttributes) {
        let mut unit = self.unit.as_mut();
        if let Some(u) = unit {
            u.set_attributes(attributes);
        }
    }
    pub fn set_unit_resource(&mut self, resource: UnitResourceIndex, amount: UnitResourceAmount) {
        let mut unit = self.unit.as_mut();
        if let Some(u) = unit {
            u.set_resource(resource, amount);
        }
    }
    pub fn set_some_unit_resources(&mut self, resources: &Vec<Option<UnitResourceAmount>>) {
        let mut unit = self.unit.as_mut();
        //println!("setting some unit resources: {:?}", unit);

        if let Some(u) = unit {
            u.set_some_resources(resources);
        } else {
            panic!("Unit doesnt exist: {:?}", self.coord);
        }
    }

    pub fn set_unit_resources(&mut self, resources: UnitResources) {
        let mut unit = self.unit.as_mut().unwrap();
        unit.set_resources(resources);

        // if let Some(u) = unit {
        //     u.set_resources(resources);
        // } else {
        //     panic!("Unit doesnt exist: {:?}", self.coord);
        // }
    }

    pub fn add_unit_resources(&mut self, resources: &SomeUnitResources) {
        let mut unit = self.unit.as_mut();
        if let Some(u) = unit {
            u.add_resources(resources);
        }
    }

    pub fn add_unit_resource(
        &mut self,
        resource_idx: UnitResourceIndex,
        amount: UnitResourceAmount,
    ) {
        let mut unit = self.unit.as_mut();
        if let Some(u) = unit {
            u.add_resource(resource_idx, amount);
        }
    }
}

pub fn empty_position(coord: Coord, manifest: &ChemistryManifest) -> Position {
    let attributes = manifest.empty_position_attributes();
    let resources = manifest.empty_position_resources();

    Position {
        coord: coord,
        unit: None,
        attributes,
        resources,
    }
}

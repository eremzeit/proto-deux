use chemistry::properties::{AttributeIndex, AttributeValue, ResourceAmount, ResourceIndex};
use chemistry::ChemistryInstance;
use simulation::common::{SimulationEventSender, SimulationEvent, Coord, send_event};
use super::unit_entry::{UnitEntryId};
use chemistry::{Chemistry, ChemistryManifest};
use HashMap;

pub type UnitResourceIndex = ResourceIndex;
pub type UnitResourceAmount = ResourceAmount;
pub type UnitAttributeIndex = AttributeIndex;
pub type UnitAttributeValue = AttributeValue;

pub type UnitResources = Vec<UnitResourceAmount>;
pub type SomeUnitResources = Vec<Option<UnitResourceAmount>>;
pub type UnitAttributes = Vec<UnitAttributeValue>;
pub type UnitId = u64;

#[derive(Clone)]
pub struct Unit {
    pub resources: UnitResources,
    pub attributes: Vec<UnitAttributeValue>,
    pub entry_id: UnitEntryId,
    pub id: UnitId,
    pub coord: Coord,
    pub last_update_tick: u64,
}

use std::fmt::{Debug, Formatter, Result};
impl Debug for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "<unit>")
    }
}

impl Unit {
    pub fn set_last_update_tick(&mut self, tick: u64) {
        self.last_update_tick = tick;
    }

    pub fn get_resource(&self, resource_idx: UnitResourceIndex) -> UnitResourceAmount {
        *self.resources.get(resource_idx).expect(&format!("Invalid unit resource set"))
    }

    pub fn get_attribute(&self, attr_idx: UnitAttributeIndex) -> &UnitAttributeValue {
        &self.attributes.get(attr_idx).expect(&format!("Invalid unit attribute set"))
    }

    pub fn add_resources(&mut self, resources: &SomeUnitResources, events: &mut SimulationEventSender) {
        add_resources_to(&mut self.resources, resources);
        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }

    pub fn set_resources(&mut self, resources: UnitResources, events: &mut SimulationEventSender) {
        //println!("unit.set_resources: {:?}", resources);
        self.resources = resources;
        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }
    
    pub fn add_resource(&mut self, idx: UnitResourceIndex, amount: i32, events: &mut SimulationEventSender) {
        self.resources[idx] += amount;
        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }

    pub fn set_resource(&mut self, resource_idx: UnitResourceIndex, amount: UnitResourceAmount, events: &mut SimulationEventSender) {
        self.resources[resource_idx] = amount;
        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }

    pub fn set_some_resources(&mut self, resources: &Vec<Option<UnitResourceAmount>>, events: &mut SimulationEventSender) {
        for i in (0..resources.len()) {
            if resources[i].is_some() {
                self.resources[i] = resources[i].unwrap();
            }
        }

        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }

    pub fn set_attribute(&mut self, attr_idx: UnitAttributeIndex, value: UnitAttributeValue, events: &mut SimulationEventSender) {
        self.attributes[attr_idx] = value;
        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }

    pub fn set_attributes(&mut self, attributes: UnitAttributes, events: &mut SimulationEventSender) {
        self.attributes = attributes;
        send_event(events, SimulationEvent::PositionUpdated(self.coord));
    }

    pub fn format_resources_short(&self, chemistry: ChemistryInstance) -> String {
        chemistry.get_manifest().format_unit_resources(&self.resources)
    }
}

pub fn merge_unit_attributes(attr1: &mut UnitAttributes, attr2: &UnitAttributes) {
    for (i, val) in attr2.iter().enumerate() {
        match val {
            AttributeValue::Nil => {}
            _ => {
                attr1[i] = val.clone();
            }
        }
    }
}

pub fn add_resources_to(resources: &mut UnitResources, to_add: &SomeUnitResources) {
    let common_len = std::cmp::min(resources.len(), to_add.len());
    for i in 0..common_len {
        
        match to_add[i] {
            Some(value) => { resources[i] += value; },
            None => {}
            
        }
        if (to_add[i].is_some()) {
        }
    }
}

pub fn empty_unit() -> Unit {
    Unit {
        // If this isn't populated with a non-empty vector before the simulation starts then the simulation will error
        resources: vec![],
        attributes: vec![],
        entry_id: 0, 
        id: 0,
        coord: (0,0),
        last_update_tick: 0,
    }
}

pub mod util {
    use super::{*};
    use simulation::common::{SomeUnitResources, UnitResourceAmount};
    
    pub fn convert_maybe_resources_to_resources(maybe_resources: SomeUnitResources) -> UnitResources {
        maybe_resources.iter().map(|maybe_amount: &Option<i32>| -> UnitResourceAmount {
            maybe_amount.unwrap_or(0) as UnitResourceAmount
        }).collect::<Vec<UnitResourceAmount>>()
    }
}

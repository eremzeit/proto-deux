use crate::simulation::common::{
    AttributeIndex, Chemistry, ChemistryManifest, PhenotypeId, UnitAttributeValue, UnitAttributes,
    UnitBehavior, UnitResourceAmount, UnitResources,
};
use crate::simulation::unit::util::convert_maybe_resources_to_resources;
use std::boxed::Box;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use crate::chemistry::properties::AttributeValue;

pub type UnitEntryId = usize;

pub type UnitEntryAttributeValue = AttributeValue;
pub type UnitEntryAttributeIndex = AttributeIndex;
pub type UnitEntryAttributes = Vec<UnitEntryAttributeValue>;

use crate::chemistry::properties::UnitEntryAttributeDefinition;

use super::common::helpers::place_units::PlaceUnitsMethod;

#[derive(Clone)]
pub struct UnitManifest {
    pub units: Vec<UnitEntry>,
}

impl UnitManifest {
    pub fn from(entries: &Vec<UnitEntry>) -> Self {
        let mut units: Vec<UnitEntry> = entries.clone();

        for i in (0..entries.len()) {
            units[i].info.unit_entry_id = i as usize;
        }

        UnitManifest { units }
    }

    pub fn init_manifest(&mut self) {
        for i in (0..self.units.len()) {
            self.units[i].info.unit_entry_id = i;
        }
    }
}

#[derive(Clone)]
pub struct UnitEntry {
    pub info: UnitEntryData,
    pub behavior: Rc<RefCell<dyn UnitBehavior>>,
    /*
        technically, the unit_behavior accesses the world via sensors, which might differ by unit_entry.
        so the sensor manifest should be included in each unit_entry.
    */
    // sensor_manifest
}

impl UnitEntry {
    pub fn new(species_name: &'static str, unit_behavior: Rc<RefCell<dyn UnitBehavior>>) -> Self {
        Self {
            info: UnitEntryData {
                species_name: species_name.to_string(),
                default_unit_attributes: None,
                default_resources: None,
                default_entry_attributes: None,
                unit_entry_id: 0,
                external_id: 0,
            },

            behavior: unit_behavior,
        }
    }

    pub fn with_default_attributes(mut self, default_attr: UnitAttributes) -> Self {
        self.info.default_unit_attributes = Some(default_attr);
        self
    }
    pub fn with_default_resources(mut self, default_res: UnitResources) -> Self {
        self.info.default_resources = Some(default_res);
        self
    }

    // //pub attributes: Option<GenomeAttributes>, //TODO
    // pub default_attributes: Option<UnitAttributes>,
    // pub default_resources: Option<UnitResources>,

    // Vec<(&'static str, UnitResourceAmount)>,
    // Vec<(&'static str, UnitAttributeValue)>,

    // pub fn with_default_attributes(mut self, hr_attr: Vec<(&'static str, UnitAttributeValue)>, cm: &ChemistryManifest) -> Self {
    //     let compiled = &cm.unit_attributes_of(hr_attr.clone());
    //     self.info.default_attributes = Some(compiled.clone());
    //     return self;
    // }

    // pub fn with_default_resources(mut self, hr_res: Vec<(&'static str, UnitResourceAmount)>, cm: &ChemistryManifest) -> Self {
    //     let compiled = convert_maybe_resources_to_resources(
    //         cm.unit_resources_of(hr_res.clone()),
    //     );
    //     self.info.default_resources = Some(compiled.clone());

    //     return self;
    // }
    pub fn set_default_attributes(mut self, attributes: &UnitAttributes) -> Self {
        self.info.default_unit_attributes = Some(attributes.clone());
        return self;
    }

    pub fn set_default_resources(mut self, resources: &UnitResources) -> Self {
        self.info.default_resources = Some(resources.clone());
        return self;
    }
}

#[derive(Clone, Debug)]
pub struct UnitEntryData {
    pub species_name: String,
    pub default_entry_attributes: Option<UnitEntryAttributes>,
    pub default_unit_attributes: Option<UnitAttributes>,
    pub default_resources: Option<UnitResources>,
    pub unit_entry_id: UnitEntryId,
    pub external_id: usize,
}

impl UnitEntryData {
    pub fn new(species_name: &'static str, place_units_method: Option<PlaceUnitsMethod>) -> Self {
        Self {
            species_name: species_name.to_string(),
            default_unit_attributes: None,
            default_resources: None,
            unit_entry_id: 0,
            default_entry_attributes: None,
            external_id: 0,
        }
    }
}

pub mod builder {
    use super::*;
    use crate::simulation::unit_entry;
    #[derive(Builder)]
    #[builder(pattern = "owned", setter(strip_option))]
    #[builder(build_fn(skip))]
    pub struct UnitEntry {
        pub behavior: Rc<RefCell<dyn UnitBehavior>>,
        pub species_name: String,
        pub default_attributes: Vec<(String, UnitAttributeValue)>,
        pub default_resources: Vec<(String, UnitResourceAmount)>,
        pub default_entry_attributes: Vec<(String, UnitEntryAttributeValue)>,
        pub place_units_method: Option<PlaceUnitsMethod>,
        pub id: UnitEntryId,
        pub external_id: usize,
    }

    impl UnitEntryBuilder {
        pub fn with_species_name(name: &'static str) -> Self {
            Self::default().species_name(name.to_string())
        }

        pub fn build(self, cm: &ChemistryManifest) -> unit_entry::UnitEntry {
            let compiled_attr = match &self.default_attributes {
                Some(attr) => Some(cm.unit_attributes_of(attr.clone()).clone()),
                None => None,
            };

            let compiled_res = match &self.default_resources {
                Some(res) => Some(convert_maybe_resources_to_resources(
                    cm.unit_resources_of(res.clone()),
                )),
                None => None,
            };

            let compiled_entry_attr = match &self.default_entry_attributes {
                Some(attr) => Some(cm.unit_entry_attributes_of(attr.clone()).clone()),
                None => None,
            };
            //let compiled_res = convert_maybe_resources_to_resources(maybe_res.clone());

            unit_entry::UnitEntry {
                info: UnitEntryData {
                    unit_entry_id: 0,
                    species_name: self.species_name.unwrap().clone(),
                    default_unit_attributes: compiled_attr,
                    default_resources: compiled_res,
                    default_entry_attributes: compiled_entry_attr,
                    external_id: self.external_id.unwrap_or(0),
                },

                behavior: self.behavior.unwrap(),
            }
        }
    }
}

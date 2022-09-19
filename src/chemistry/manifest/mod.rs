pub mod serialize;

use self::properties::*;
use self::reactions::*;
use crate::biology::genetic_manifest::predicates::OperatorParam;
use crate::chemistry::actions::{default_actions, ActionParam};
use crate::simulation::common::*;
use crate::simulation::unit_entry::UnitEntryAttributeIndex;
use crate::util::Coord;

use super::init_chemistry_action_params;

/**
 *
 *  question: would it be possible for a simulation to have more than one distinct chemistry manifests?
 *      -   yes, eg, if one unit had different reactions available to it or if theey required different configurations.
 *      -   so maybe we should split off the reactions into something like UnitReactionsManifest, which would be unique per
 *          unit entry
 *
 */
#[derive(Clone)]
pub struct ChemistryManifest {
    pub chemistry_key: String,
    pub reactions: Vec<CompiledReactionDefinition>,
    pub action_manifest: ActionManifest,
    pub all_properties: Vec<Property>,

    pub unit_resources: Vec<UnitResourceDefinition>,
    pub unit_attributes: Vec<UnitAttributeDefinition>,
    pub position_attributes: Vec<PositionAttributeDefinition>,
    pub position_resources: Vec<PositionResourceDefinition>,
    pub simulation_attributes: Vec<SimulationAttributeDefinition>,
    pub unit_entry_attributes: Vec<UnitEntryAttributeDefinition>,
}

impl ChemistryManifest {
    pub fn unit_resource_by_key(&self, key: &str) -> UnitResourceDefinition {
        self.unit_resources
            .iter()
            .find(|&x| -> bool { key == x.key })
            .expect(&format!("could not find unit resource with key: {}", key))
            .clone()
    }

    pub fn unit_attribute_by_key(&self, key: &str) -> UnitAttributeDefinition {
        self.unit_attributes
            .iter()
            .find(|&x| -> bool { key == x.key })
            .expect(&format!("could not find unit attribute with key: {}", key))
            .clone()
    }

    pub fn unit_entry_attribute_by_key(&self, key: &str) -> UnitEntryAttributeDefinition {
        self.unit_entry_attributes
            .iter()
            .find(|&x| -> bool { key == x.key })
            .expect(&format!(
                "could not find unit entry attribute with key: {}",
                key
            ))
            .clone()
    }

    pub fn position_attribute_by_key(&self, key: &str) -> PositionAttributeDefinition {
        self.position_attributes
            .iter()
            .find(|&x| -> bool { key == x.key })
            .unwrap()
            .clone()
    }
    pub fn position_resource_by_key(&self, key: &str) -> PositionResourceDefinition {
        self.position_resources
            .iter()
            .find(|&x| -> bool { key == x.key })
            .unwrap()
            .clone()
    }

    pub fn simulation_attribute_by_key(&self, key: &str) -> SimulationAttributeDefinition {
        self.simulation_attributes
            .iter()
            .find(|&x| -> bool { key == x.key })
            .unwrap()
            .clone()
    }
    pub fn empty_simulation_attributes(&self) -> SimulationAttributes {
        let length = self.unit_attributes.len();
        let mut attr: UnitAttributes = Vec::with_capacity(length);
        attr.resize(length, AttributeValue::Nil);
        return attr;
    }

    pub fn empty_unit_resources(&self) -> UnitResources {
        let length = self.unit_resources.len();
        let mut resources: UnitResources = Vec::with_capacity(length);
        resources.resize(length, 0);
        return resources;
    }
    pub fn empty_unit_attributes(&self) -> UnitAttributes {
        let length = self.unit_attributes.len();
        let mut attr: UnitAttributes = Vec::with_capacity(length);
        attr.resize(length, AttributeValue::Nil);
        return attr;
    }
    pub fn empty_position_resources(&self) -> PositionResourceTabulations {
        let length = self.position_resources.len();
        let mut resources: PositionResourceTabulations = Vec::with_capacity(length);
        resources.resize(length, PositionResourceTabulation::new());
        return resources;
    }

    pub fn empty_position_attributes(&self) -> PositionAttributes {
        let length = self.position_attributes.len();
        let mut resources: PositionAttributes = Vec::with_capacity(length);
        resources.resize(length, AttributeValue::Nil);
        return resources;
    }
    pub fn empty_unit_entry_attributes(&self) -> UnitEntryAttributes {
        let length = self.unit_entry_attributes.len();
        let mut attr: UnitEntryAttributes = Vec::with_capacity(length);
        attr.resize(length, AttributeValue::Nil);
        return attr;
    }

    pub fn unit_resources_of(
        &self,
        resource_list: Vec<(String, UnitResourceAmount)>,
    ) -> SomeUnitResources {
        let length = self.unit_resources.len();
        let mut resources: SomeUnitResources = Vec::with_capacity(length);
        resources.resize(length, None);

        for pair in resource_list.iter() {
            let idx = self.unit_resource_by_key(&pair.0).id as UnitResourceIndex;
            resources[idx] = Some(pair.1);
        }

        resources
    }

    pub fn unit_attributes_of(
        &self,
        attribute_list: Vec<(String, UnitAttributeValue)>,
    ) -> UnitAttributes {
        let length = self.unit_attributes.len();
        let mut attributes: UnitAttributes = Vec::with_capacity(length);
        attributes.resize(length, UnitAttributeValue::Nil);

        for pair in attribute_list.iter() {
            let idx = self.unit_attribute_by_key(&pair.0).id as UnitAttributeIndex;
            attributes[idx] = pair.1.clone();
        }

        attributes
    }
    pub fn unit_entry_attributes_of(
        &self,
        attribute_list: Vec<(String, UnitEntryAttributeValue)>,
    ) -> UnitEntryAttributes {
        let length = self.unit_entry_attributes.len();

        let mut attributes: UnitEntryAttributes = Vec::with_capacity(length);
        attributes.resize(length, UnitEntryAttributeValue::Nil);

        for pair in attribute_list.iter() {
            let idx = self.unit_attribute_by_key(&pair.0).id as UnitAttributeIndex;
            attributes[idx] = pair.1.clone();
        }

        attributes
    }
    pub fn pos_attributes_of(
        &self,
        attribute_list: Vec<(&'static str, PositionAttributeValue)>,
    ) -> PositionAttributes {
        let length = self.position_attributes.len();
        let mut attributes: UnitAttributes = Vec::with_capacity(length);
        attributes.resize(length, UnitAttributeValue::Nil);

        for pair in attribute_list.iter() {
            let idx = self.position_attribute_by_key(pair.0).id as UnitAttributeIndex;
            attributes[idx] = pair.1.clone();
        }

        attributes
    }

    pub fn format_unit_resources(&self, resources: &UnitResources) -> String {
        let mut s = String::new();

        for i in (0..self.unit_resources.len()) {
            let def = &self.unit_resources[i];

            s = format!("{}\n[{}, {}]", &s, &def.key, resources[i]);
        }

        s
    }

    pub fn raw_property_id_to_key(&self, raw_prop_id: RawPropertyId) -> Option<String> {
        let prop_id = self.normalize_raw_property_id(raw_prop_id);
        if prop_id.is_none() {
            return None;
        }

        Some(match &prop_id.unwrap() {
            PropertyId::PositionAttributeId(id) => {
                self.position_attributes[*id as usize].key.clone()
            }
            PropertyId::PositionResourceId(id) => {
                self.position_attributes[*id as usize].key.clone()
            }
            PropertyId::UnitAttributeId(id) => self.position_attributes[*id as usize].key.clone(),
            PropertyId::UnitResourceId(id) => self.position_attributes[*id as usize].key.clone(),
            PropertyId::SimulationAttributeId(id) => {
                self.simulation_attributes[*id as usize].key.clone()
            }
        })
    }
    pub fn gather_properties(&self) -> Vec<Property> {
        let mut all_props: Vec<Property> = vec![];

        all_props.append(
            &mut self
                .simulation_attributes
                .iter()
                .map(|x| Property {
                    key: x.key.to_string(),
                    long_key: format!("sim_attr::{}", x.key.to_string()),
                    property_id: PropertyId::SimulationAttributeId(x.id),
                    id: 0,
                })
                .collect::<Vec<_>>(),
        );

        all_props.append(
            &mut self
                .unit_resources
                .iter()
                .map(|x| Property {
                    key: x.key.to_string(),
                    long_key: format!("unit_res::{}", x.key.to_string()),
                    property_id: PropertyId::UnitResourceId(x.id),
                    id: 0,
                })
                .collect::<Vec<_>>(),
        );

        all_props.append(
            &mut self
                .unit_attributes
                .iter()
                .map(|x| Property {
                    key: x.key.to_string(),
                    long_key: format!("unit_attr::{}", x.key.to_string()),
                    property_id: PropertyId::UnitAttributeId(x.id),
                    id: 0,
                })
                .collect::<Vec<_>>(),
        );

        all_props.append(
            &mut self
                .position_resources
                .iter()
                .map(|x| Property {
                    key: x.key.to_string(),
                    long_key: format!("pos_res::{}", x.key.to_string()),
                    property_id: PropertyId::PositionResourceId(x.id),
                    id: 0,
                })
                .collect::<Vec<_>>(),
        );

        all_props.append(
            &mut self
                .position_attributes
                .iter()
                .map(|x| Property {
                    key: x.key.to_string(),
                    long_key: format!("pos_attr::{}", x.key.to_string()),
                    property_id: PropertyId::PositionAttributeId(x.id),
                    id: 0,
                })
                .collect::<Vec<_>>(),
        );
        let mut i = 0;
        while i < all_props.len() {
            let mut prop = &mut all_props[i];
            prop.id = i;
            i += 1;
        }

        all_props
    }

    pub fn normalize_raw_property_id(&self, prop_id: RawPropertyId) -> Option<PropertyId> {
        let mut id = prop_id;
        let sa_length = self.simulation_attributes.len();
        if id >= sa_length {
            id -= sa_length;
        } else {
            return Some(PropertyId::SimulationAttributeId(id));
        }

        let ur_length = self.unit_resources.len();
        if id >= ur_length {
            id -= ur_length;
        } else {
            return Some(PropertyId::UnitResourceId(id));
        }
        let ua_length = self.unit_attributes.len();
        if id >= ua_length {
            id -= ua_length;
        } else {
            return Some(PropertyId::UnitAttributeId(id));
        }

        let pr_length = self.position_resources.len();
        if id >= pr_length {
            id -= pr_length;
        } else {
            return Some(PropertyId::PositionResourceId(id));
        }

        let pa_length = self.position_attributes.len();
        if id >= pa_length {
            id -= pa_length;
        } else {
            return Some(PropertyId::PositionAttributeId(id));
        }

        None
    }
    pub fn identify_property_key(&self, key: &String) -> Option<PropertyId> {
        let starts_with_unit = key.starts_with("unit::");
        let starts_with_pos = key.starts_with("position::") || key.starts_with("pos::");
        let mut result: Option<PropertyId> = None;

        if starts_with_unit {
            for i in 0..self.unit_resources.len() {
                let prop = &self.unit_resources[i];
                if key.ends_with(&self.unit_resources[i].key) {
                    if result.is_some() {
                        panic!("Ambiguous property key: {}", key);
                    }
                    result = Some(PropertyId::UnitResourceId(prop.id));
                }
            }
            for i in 0..self.unit_attributes.len() {
                let prop = &self.unit_attributes[i];
                if key.ends_with(&self.unit_attributes[i].key) {
                    if result.is_some() {
                        panic!("Ambiguous property key: {}", key);
                    }
                    result = Some(PropertyId::UnitAttributeId(prop.id));
                }
            }
        }
        if starts_with_pos {
            for i in 0..self.position_resources.len() {
                let prop = &self.position_resources[i];
                if key.ends_with(&self.position_resources[i].key) {
                    result = Some(PropertyId::PositionResourceId(prop.id));
                }
            }

            for i in 0..self.position_attributes.len() {
                let prop = &self.position_attributes[i];
                if key.ends_with(&self.position_attributes[i].key) {
                    return Some(PropertyId::PositionAttributeId(prop.id));
                } else {
                    return None;
                }
            }
        }
        return None;
    }

    /**
     * Returns the number of parameters that the unit_behavior is supposed to supply for this execution
     */
    pub fn get_required_params_for_reaction(&self, key: &String) -> usize {
        let reaction = self.identify_reaction(key).unwrap();
        let mut count = 0;
        for reagent in &reaction.reagents {
            for param in &reagent.params {
                match &param {
                    ActionParam::UnitBehaviorArgument(t) => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }

        count
    }

    pub fn identify_reaction(&self, key: &String) -> Option<ReactionDefinition> {
        for (i, reaction) in self.reactions.iter().enumerate() {
            if &reaction.key == key {
                return Some(reaction.clone());
            }
        }

        return None;
    }

    pub fn normalize_manifest(&mut self, config: &ChemistryConfiguration) {
        self.normalize_properties(config);

        init_chemistry_action_params(self, config);
        let mut reactions: Vec<ReactionDefinition> = self.reactions.clone();
        // normalize reaction definitions
        //
        // Lookup string keys and replace them with the numerical id
        for (i, reaction) in self.reactions.iter().enumerate() {
            reactions[i].id = i;
            // println!("processing reaction key: {}", &reaction.key);
            for (j, reagent) in reaction.reagents.iter().enumerate() {
                let action_key = &reaction.reagents[j].action_key;

                reactions[i].reagents[j].action_index =
                    self.action_manifest.by_key(action_key).index;

                // println!("\tfor reagent: {}", &reagent.action_key);
                for (param_idx, param_value) in reagent.params.iter().enumerate() {
                    // println!("\t\tinitial param value: {:?}", param_value);
                    match param_value {
                        ActionParam::UnitAttributeKey(key) => {
                            let idx = self.unit_attribute_by_key(key).id as usize;
                            reactions[i].reagents[j].params[param_idx] =
                                ActionParam::UnitAttributeIndex(idx);
                        }
                        ActionParam::UnitResourceKey(key) => {
                            let idx = self.unit_resource_by_key(key).id as usize;
                            reactions[i].reagents[j].params[param_idx] =
                                ActionParam::UnitResourceIndex(idx);
                        }
                        ActionParam::PositionAttributeKey(key) => {
                            let idx = self.position_attribute_by_key(key).id as usize;
                            reactions[i].reagents[j].params[param_idx] =
                                ActionParam::PositionAttributeIndex(idx)
                        }

                        ActionParam::PositionResourceKey(key) => {
                            let idx = self.position_resource_by_key(key).id as usize;
                            reactions[i].reagents[j].params[param_idx] =
                                ActionParam::PositionResourceIndex(idx)
                        }

                        ActionParam::SimulationAttributeKey(key) => {
                            let idx = self.simulation_attribute_by_key(key).id as usize;
                            reactions[i].reagents[j].params[param_idx] =
                                ActionParam::SimulationAttributeIndex(idx)
                        }

                        _ => {}
                    };
                }
            }
        }

        /*
         * Normalize reagent definitions
         */

        // println!("NORMALIZING REACTIONS");
        // println!("--- pre: {:?}\n", &self.reactions);
        // println!("--- post: {:?}\n", &reactions);
        // println!("END NORMALIZING REACTIONS\n");

        self.reactions = reactions;
    }

    pub fn normalize_properties(&mut self, config: &ChemistryConfiguration) {
        // let mut unit_resources: Vec<ResourceDefinition> = self.unit_resources.clone();
        // let mut unit_attributes: Vec<UnitAttributeDefinition> = self.unit_attributes.clone();
        // let mut pos_attributes: Vec<PositionAttributeDefinition> = self.position_attributes.clone();
        // let mut pos_resources: Vec<PositionResourceDefinition> = self.position_resources.clone();
        // let mut sim_attributes: Vec<SimulationAttributeDefinition> =
        //     self.simulation_attributes.clone();

        // normalize property definitions
        for (i, prop_def) in self.unit_resources.iter_mut().enumerate() {
            prop_def.id = i;
        }
        for (i, prop_def) in self.unit_attributes.iter_mut().enumerate() {
            prop_def.id = i as UnitAttributeIndex;
        }

        for (i, prop_def) in self.position_attributes.iter_mut().enumerate() {
            prop_def.id = i as PositionAttributeIndex;
        }

        for (i, prop_def) in self.position_resources.iter_mut().enumerate() {
            prop_def.id = i as PositionAttributeIndex;
        }

        for (i, prop_def) in self.simulation_attributes.iter_mut().enumerate() {
            prop_def.id = i as SimulationAttributeIndex;
        }

        for (i, prop_def) in self.unit_entry_attributes.iter_mut().enumerate() {
            prop_def.id = i as UnitEntryAttributeIndex;
        }

        self.all_properties = self.gather_properties();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn manifest() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let manifest = &gm.chemistry_manifest;

        assert_eq!(manifest.unit_resources[0].id, 0);
        assert_eq!(manifest.unit_resources[1].id, 1);
    }
}

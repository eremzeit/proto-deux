use serde::Deserialize;
use serde::Serialize;

use self::properties::*;
use self::reactions::*;
use crate::biology::genetic_manifest::predicates::OperatorParam;
use crate::chemistry::actions::{default_actions, ActionParam};
use crate::simulation::common::*;
use crate::simulation::unit_entry::UnitEntryAttributeIndex;
use crate::util::Coord;

#[derive(Clone)]
pub struct ChemistryManifestData {
    pub chemistry_key: String,
    pub reactions: Vec<ReactionDefinition>,
    pub action_manifest: ActionManifestData,
    pub all_properties: Vec<Property>,

    pub unit_resources: Vec<UnitResourceDefinition>,
    pub unit_attributes: Vec<UnitAttributeDefinition>,
    pub position_attributes: Vec<PositionAttributeDefinition>,
    pub position_resources: Vec<PositionResourceDefinition>,
    pub simulation_attributes: Vec<SimulationAttributeDefinition>,
    pub unit_entry_attributes: Vec<UnitEntryAttributeDefinition>,
}

impl ChemistryManifestData {
    pub fn to_manifest(&self, action_library: Vec<ActionDefinition>) -> ChemistryManifest {
        ChemistryManifest {
            chemistry_key: self.chemistry_key.clone(),
            reactions: self.reactions.clone(),
            action_manifest: self.action_manifest.to_manifest(&action_library),
            all_properties: self.all_properties.clone(),
            unit_resources: self.unit_resources.clone(),
            unit_attributes: self.unit_attributes.clone(),
            position_attributes: self.position_attributes.clone(),
            position_resources: self.position_resources.clone(),
            simulation_attributes: self.simulation_attributes.clone(),
            unit_entry_attributes: self.unit_entry_attributes.clone(),
        }
    }

    pub fn from_manifest(manifest: &ChemistryManifest) -> Self {
        let manifest = manifest.clone();
        Self {
            chemistry_key: manifest.chemistry_key,
            reactions: manifest.reactions,
            action_manifest: ActionManifestData::from_manifest(&manifest.action_manifest),
            all_properties: manifest.all_properties,
            unit_resources: manifest.unit_resources,
            unit_attributes: manifest.unit_attributes,
            position_attributes: manifest.position_attributes,
            position_resources: manifest.position_resources,
            simulation_attributes: manifest.simulation_attributes,
            unit_entry_attributes: manifest.unit_entry_attributes,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::chemistry::{Chemistry, FooChemistry};
    use crate::simulation::common::*;

    use super::ChemistryManifestData;

    #[test]
    fn test_serialize() {
        let chemistry = FooChemistry::construct(ChemistryConfiguration::new());
        let manifest = chemistry.get_manifest();

        let manifest_data = ChemistryManifestData::from_manifest(manifest);

        assert_eq!(
            manifest.action_manifest.actions.len(),
            manifest_data.action_manifest.actions.len()
        );
    }
}

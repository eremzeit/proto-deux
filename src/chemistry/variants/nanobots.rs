use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::simulation::common::helpers::resource_allocation::allocate_stored_resources;
use crate::simulation::common::helpers::resource_allocation::StoredResourceAllocationMethod;
use crate::simulation::common::helpers::unit_behavior_execution::behavior_execution;
use crate::simulation::unit::*;
use crate::simulation::world::World;
use crate::simulation::Simulation;
use crate::util::Coord;

pub struct NanobotsChemistry {
    manifest: ChemistryManifest,
    configuration: ChemistryConfiguration,
}

impl NanobotsChemistry {}

impl Chemistry for NanobotsChemistry {
    fn construct(config: ChemistryConfiguration) -> Box<NanobotsChemistry> {
        let mut chemistry = NanobotsChemistry {
            manifest: NanobotsChemistry::construct_manifest(&config),
            configuration: config,
        };
        wrap_chemistry!(chemistry)
    }

    fn get_key() -> String {
        "nanobots".to_string()
    }

    fn construct_manifest(config: &ChemistryConfiguration) -> ChemistryManifest {
        let mut manifest = ChemistryManifest {
            chemistry_key: Self::get_key(),
            action_manifest: ActionManifest::new(Self::construct_action_library()),
            all_properties: vec![],
            simulation_attributes: vec![],
            unit_resources: vec![UnitResourceDefinition::new("energy", false, 0)],
            unit_attributes: vec![],    //todo
            position_resources: vec![], //todo
            position_attributes: vec![PositionAttributeDefinition::new(
                "is_rooted",
                AttributeDefinitionType::Boolean,
                0,
            )],
            unit_entry_attributes: vec![],
            reactions: vec![],
        };

        let config = Self::fill_with_defaults(config.clone());
        manifest.normalize_manifest(&config);

        manifest
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }

    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue> {
        self.get_manifest().empty_simulation_attributes()
    }

    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue> {
        self.get_manifest().empty_unit_entry_attributes()
    }

    fn on_simulation_tick(&self, sim: &mut SimCell) -> bool {
        allocate_stored_resources(
            sim,
            sim.unit_manifest,
            &StoredResourceAllocationMethod::Every,
        );
        behavior_execution(sim);

        true
    }

    fn on_simulation_finish(&self, sim: &mut SimCell) {}
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::chemistry::actions::*;

    #[test]
    fn make_nanobots_manifest() {}
}

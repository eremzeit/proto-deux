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
    place_units_method: PlaceUnitsMethod,
    configuration: ChemistryConfiguration,
}

impl NanobotsChemistry {
    pub fn construct(
        place_units_method: PlaceUnitsMethod,
        config: ChemistryConfiguration,
    ) -> Box<NanobotsChemistry> {
        let mut chemistry = NanobotsChemistry {
            manifest: NanobotsChemistry::default_manifest(),
            place_units_method: place_units_method,
            configuration: config,
        };
        chemistry.init_manifest();
        Box::new(chemistry)
    }
    pub fn default_manifest() -> ChemistryManifest {
        let reactions = vec![ReactionDefinition::new(
            &"new_unit_right",
            vec![
                reagent!(
                    "offset_unit_resource",
                    param_value!(UnitResourceKey, "energy"),
                    param_value!(UnitResourceAmount, -10),
                ),
                reagent!("new_unit", unit_behavior_arg!(Direction),),
            ],
        )];

        let mut manifest = ChemistryManifest {
            action_set: default_actions(),
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

            reactions,
        };

        manifest.normalize_manifest();

        manifest
    }
}

impl Chemistry for NanobotsChemistry {
    fn get_key(&self) -> String {
        "nanobots".to_string()
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn get_unit_placement(&self) -> PlaceUnitsMethod {
        self.place_units_method.clone()
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

    fn get_next_unit_resources(
        &self,
        entry: &UnitEntryData,
        pos: &Position,
        unit: &Unit,
        world: &World,
        tick_multiplier: u32,
    ) -> UnitResources {
        self.get_manifest().empty_unit_resources()
    }

    fn on_simulation_tick(&self, sim: &mut SimCell) {
        allocate_stored_resources(
            sim,
            sim.unit_manifest,
            &StoredResourceAllocationMethod::Every,
        );
        behavior_execution(sim);
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

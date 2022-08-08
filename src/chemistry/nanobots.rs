use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::simulation::specs::place_units::PlaceUnits;
use crate::simulation::unit::*;
use crate::simulation::world::World;
use crate::simulation::Simulation;
use crate::util::Coord;

pub struct NanobotsChemistry {
    manifest: ChemistryManifest,
}

impl NanobotsChemistry {
    pub fn construct() -> Box<NanobotsChemistry> {
        Box::new(NanobotsChemistry {
            manifest: NanobotsChemistry::default_manifest(),
        })
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
                reagent!("new_unit", param_arg!(Direction),),
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

    fn construct_specs(
        &self,
        unit_placement: &PlaceUnitsMethod,
    ) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
        vec![Box::new(PlaceUnits {
            method: unit_placement.clone(),
        })]
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
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::chemistry::actions::*;

    #[test]
    fn make_nanobots_manifest() {}
}

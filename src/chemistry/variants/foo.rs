use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;
use crate::simulation::common::reactions::{ReactionDefinition, ReagentDefinition};
use crate::simulation::{
    common::{
        default_actions,
        helpers::{
            place_units::PlaceUnitsMethod,
            resource_allocation::{allocate_stored_resources, StoredResourceAllocationMethod},
            unit_behavior_execution::behavior_execution,
        },
        properties::{
            AttributeDefinitionType, PositionAttributeDefinition, UnitResourceDefinition,
        },
        ActionSet, Chemistry, ChemistryInstance, ChemistryManifest, UnitAttributeDefinition,
        UnitEntryAttributeValue, UnitEntryData,
    },
    position::Position,
    unit::{Unit, UnitResources},
    world::World,
    SimCell, SimulationAttributeValue,
};

pub mod constants {
    pub const NEW_UNIT_COST: i32 = 100;
}
pub mod defs {
    use crate::simulation::common::ReactionId;

    use super::*;

    def_unit_entry_attributes! {[
        [foo_entry_attribute, Number]
    ]}

    def_simulation_attributes! {[
        [is_sim_foo, Boolean]
    ]}

    def_unit_attributes! {[
        [is_foo_unit, Boolean]
    ]}

    def_position_attributes! {[
        [is_foo_position, Boolean]
    ]}

    def_position_resources! {[
        [foo_position_resource, false]
    ]}

    def_unit_resources! {[
       [foo_streamed_resource, false],
       [foo_stored_resource, false]
    ]}

    pub const REACTION_ID_NEW_UNIT: ReactionId = 0;

    def_reactions! {
        reaction!("new_unit",
            reagent!("offset_unit_resource",
                param_value!(UnitResourceKey, "foo_stored_resource"),
                chemistry_arg!(UnitResourceAmount, new_unit_cost),
                param_value!(Boolean, false),
            ),
            reagent!("new_unit",
                unit_behavior_arg!(Direction),
            ),
        ),
    }
}

/**
 * TODO: rename to FooChemistry.
 */
pub struct FooChemistry {
    pub manifest: ChemistryManifest,
    pub configuration: ChemistryConfiguration,
}

impl FooChemistry {
    pub fn custom_actions() -> ActionSet {
        ActionSet::from(vec![])
    }
}

impl Chemistry for FooChemistry {
    fn construct(config: ChemistryConfiguration) -> Box<FooChemistry> {
        let mut chemistry = FooChemistry {
            manifest: FooChemistry::construct_manifest(&config),
            configuration: config,
        };

        wrap_chemistry!(chemistry)
    }

    fn construct_manifest(config: &ChemistryConfiguration) -> ChemistryManifest {
        let mut manifest = ChemistryManifest {
            all_properties: vec![],
            simulation_attributes: defs::SimulationAttributesLookup::make_defs(),
            unit_entry_attributes: defs::UnitEntryAttributesLookup::make_defs(),
            action_set: default_actions().add(Self::custom_actions().actions.clone()),
            unit_resources: defs::UnitResourcesLookup::make_defs(),
            unit_attributes: defs::UnitAttributesLookup::make_defs(),
            position_attributes: defs::PositionAttributesLookup::make_defs(),
            position_resources: defs::PositionResourcesLookup::make_defs(),
            reactions: defs::get_reactions(),
        };

        manifest.normalize_manifest(config);

        manifest
    }

    fn default_config() -> ChemistryConfiguration
    where
        Self: Sized,
    {
        let mut config = ChemistryConfiguration::new();
        config.insert(
            "new_unit_cost".to_owned(),
            ChemistryConfigValue::Integer(10),
        );
        config
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn get_key(&self) -> String {
        "foo".to_string()
    }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }

    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    fn allocate_unit_resources(&self, coord: &Coord, sim: &mut SimCell) {
        let position_attributes = defs::PositionAttributesLookup::new();
        let unit_resources = defs::UnitResourcesLookup::new();
        let unit_attributes = defs::UnitAttributesLookup::new();
        let sim_attributes = defs::SimulationAttributesLookup::new();
        let position_resources = defs::PositionResourcesLookup::new();
        let unit_entry_attributes = defs::UnitEntryAttributesLookup::new();

        let mut pos = sim
            .world
            .grid
            .get_mut([coord.0, coord.1])
            .unwrap()
            .as_mut()
            .unwrap();

        // let is_air_source = pos
        //     .get_attribute(position_attributes.is_air_source)
        //     .unwrap_bool();

        // let is_cheese_source = pos
        //     .get_attribute(position_attributes.is_cheese_source)
        //     .unwrap_bool();

        let is_foo_position_attr = pos
            .get_attribute(position_attributes.is_foo_position)
            .unwrap_bool();
        let has_unit = pos.has_unit();
        let unit = pos.unit.as_mut().unwrap();

        let resources = &mut unit.resources;

        if is_foo_position_attr {
            resources[unit_resources.foo_streamed_resource] = 10;
            resources[unit_resources.foo_stored_resource] += 20;
        } else {
            resources[unit_resources.foo_streamed_resource] =
                std::cmp::max(resources[unit_resources.foo_streamed_resource] - 1, 0);
        }
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

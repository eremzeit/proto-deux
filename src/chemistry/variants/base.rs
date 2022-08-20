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

    pub const REACTION_ID_GOBBLE_CHEESE: ReactionId = 0;

    def_reactions! {
        reaction!("new_unit",
            reagent!("offset_unit_resource",
                param_value!(UnitResourceKey, "foo_stored_resource"),
                param_value!(UnitResourceAmount, -constants::NEW_UNIT_COST),
                param_value!(Boolean, false),
            ),
            reagent!("new_unit",
                unit_behavior_arg!(Direction),
            ),
        ),
    }
}

pub struct BaseChemistry {
    pub manifest: ChemistryManifest,
    pub configuration: ChemistryConfiguration,
}

impl BaseChemistry {
    pub fn construct() -> ChemistryInstance {
        let mut chemistry = BaseChemistry {
            manifest: BaseChemistry::default_manifest(),
            configuration: ChemistryConfiguration::new(),
        };

        chemistry.init_manifest();
        wrap_chemistry!(chemistry)
    }

    pub fn default_manifest() -> ChemistryManifest {
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

        manifest.normalize_manifest();

        manifest
    }

    pub fn custom_actions() -> ActionSet {
        ActionSet::from(vec![])
    }
}

impl Chemistry for BaseChemistry {
    // fn get_unit_placement(&self) -> PlaceUnitsMethod {
    //     self.place_units_method.clone()
    // }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn get_key(&self) -> String {
        "base".to_string()
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
        let mut resources = unit.resources.clone();

        let position_attributes = defs::PositionAttributesLookup::new();
        let unit_resources = defs::UnitResourcesLookup::new();
        let unit_attributes = defs::UnitAttributesLookup::new();
        let sim_attributes = defs::SimulationAttributesLookup::new();
        let position_resources = defs::PositionResourcesLookup::new();

        let is_foo_position_attr = pos
            .get_attribute(position_attributes.is_foo_position)
            .unwrap_bool();

        if is_foo_position_attr {
            resources[unit_resources.foo_streamed_resource] = 10;
        } else {
            resources[unit_resources.foo_streamed_resource] =
                std::cmp::max(resources[unit_resources.foo_streamed_resource] - 1, 0);
        }

        let is_foo_position_attribute = pos
            .get_attribute(position_attributes.is_foo_position)
            .unwrap_bool();
        //println!("is_cheese_source: {}", is_cheese_source);
        //let id_cheese: PositionAttributeIndex = self.get_manifest().unit_resource_by_key("cheese").id as usize;
        //println!("id_air: {}", id_air);
        //println!("id_cheese: {}", id_cheese);

        if is_foo_position_attribute {
            resources[unit_resources.foo_stored_resource] += 20;
        }

        // println!("resources: {:?}", resources);
        resources
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

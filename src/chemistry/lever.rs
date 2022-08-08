use crate::chemistry::actions::ActionSet;
use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::simulation::common::*;
use crate::simulation::specs::place_units::PlaceUnits;
use crate::simulation::specs::SimulationSpec;
use crate::simulation::unit::*;
use crate::simulation::world::World;
use crate::simulation::Simulation;
use crate::util::Coord;

use std::rc::Rc;

use crate::simulation::position::{
    PositionAttributeIndex, PositionAttributeValue, PositionResourceAmount, PositionResourceIndex,
};

use crate::chemistry::properties::PositionAttributeDefinition;

use crate::simulation::unit::{
    UnitAttributeIndex, UnitAttributeValue, UnitResourceAmount, UnitResourceIndex,
};
use crate::util::*;
use std::collections::HashMap;

#[macro_use]
pub mod constants {}

pub struct LeverChemistry {
    manifest: ChemistryManifest,
}

pub mod defs {
    use super::*;

    def_unit_entry_attributes! {[
        [lever_pulls, Number]
    ]}

    def_simulation_attributes! {[ ]}

    def_unit_attributes! {[]}

    def_position_attributes! {[ ]}

    def_position_resources! {[ ]}

    def_unit_resources! {[ ]}

    pub const REACTION_ID_PULL_LEVER: ReactionId = 0;

    def_reactions! {
        reaction!("pull_lever",
            reagent!("pull_lever",
                param_arg!(ConstantNum),
            ),
        ),
    }
}
impl LeverChemistry {
    pub fn construct() -> ChemistryInstance {
        wrap_chemistry!(LeverChemistry {
            manifest: LeverChemistry::default_manifest(),
        })
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
        ActionSet::from(vec![ActionDefinition::new(
            &"pull_lever",
            vec![],
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let unit = sim_cell.world.get_unit_at(context.coord).unwrap();
                    let entry_id = unit.entry_id;

                    let uea_lookup = defs::UnitEntryAttributesLookup::new();
                    let to_add = context.params[0].to_constant();
                    println!("pulling lever");
                    let existing = sim_cell.unit_entry_attributes[entry_id as usize]
                        [uea_lookup.lever_pulls]
                        .unwrap_integer();
                    sim_cell.unit_entry_attributes[entry_id][uea_lookup.lever_pulls] =
                        AttributeValue::Integer(existing + to_add);
                    true
                },
            ),
        )])
    }
}

impl Chemistry for LeverChemistry {
    fn get_key(&self) -> String {
        "lever".to_string()
    }

    fn construct_specs(
        &self,
        unit_placement: &PlaceUnitsMethod,
    ) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
        vec![
            Box::new(PlaceUnits {
                method: unit_placement.clone(),
            }),
            Box::new(PhenotypeExecution {}),
            Box::new(specs::PostTick {}),
        ]
    }

    fn get_next_unit_resources(
        &self,
        entry: &UnitEntryData,
        pos: &Position,
        unit: &Unit,
        world: &World,
        tick_multiplier: u32,
    ) -> UnitResources {
        unit.resources.clone()
    }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }

    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    fn get_base_streamed_resource_allocation(
        &self,
        world: &mut World,
        coord: &Coord,
    ) -> SomeUnitResources {
        return self.manifest.unit_resources_of(vec![]);
    }

    fn get_base_stored_resource_allocation(
        &self,
        world: &mut World,
        coord: &Coord,
    ) -> SomeUnitResources {
        return self.manifest.unit_resources_of(vec![]);
    }

    fn init_world_custom(&self, world: &mut World) {}

    fn get_default_simulation_attributes(&self) -> Vec<SimulationAttributeValue> {
        self.get_manifest().empty_simulation_attributes()
    }

    fn get_default_unit_entry_attributes(&self) -> Vec<UnitEntryAttributeValue> {
        self.get_manifest().empty_unit_entry_attributes()
    }

    fn get_default_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        self.get_manifest().unit_attributes_of(vec![])
    }
}

mod specs {
    use super::*;

    pub struct PostTick {}

    impl SimulationSpec for PostTick {
        fn on_tick(&mut self, sim: &mut SimCell, context: &SpecContext) {}

        fn get_name(&self) -> String {
            "Lever::PostTick".to_string()
        }
    }

    mod tests {
        #[allow(unused_imports)]
        use super::*;
        use crate::chemistry::actions::*;
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::chemistry::actions::*;

    #[test]
    fn make_cheese_manifest() {
        let cheese = LeverChemistry::construct();
    }

    #[test]
    fn macros() {
        let unit_resources = defs::UnitResourcesLookup::make_defs();
        let unit_attributes = defs::UnitAttributesLookup::make_defs();
        let position_attributes = defs::PositionAttributesLookup::make_defs();
        let position_resources = defs::PositionResourcesLookup::make_defs();
    }

    mod gobble_cheese {
        use super::*;
        use crate::chemistry::actions::tests::execute_action;
        use crate::tests::fixtures;

        #[test]
        fn do_action() {
            let unit_attributes = defs::UnitAttributesLookup::new();
            let position_attributes = defs::PositionAttributesLookup::new();
            let sim_attributes = defs::SimulationAttributesLookup::new();
            let position_resources = defs::PositionResourcesLookup::new();
            let unit_resources = defs::UnitResourcesLookup::new();

            let actions = LeverChemistry::custom_actions();
            let action = actions.by_key("pull_lever");

            let src_coord = (2, 0);
            let mut sim = fixtures::default_base(Some(vec![Box::new(PlaceUnits {
                method: PlaceUnitsMethod::ManualSingleEntry {
                    attributes: None,
                    coords: vec![src_coord],
                },
            })]));

            assert_eq!(sim.unit_entry_attributes[0][0].unwrap_integer(), 0);
            let params = vec![ActionParam::Constant(1)];
            assert!(execute_action(
                &action,
                &src_coord,
                &mut sim,
                params.as_slice()
            ));

            assert_eq!(sim.unit_entry_attributes[0][0].unwrap_integer(), 1);
        }
    }
}

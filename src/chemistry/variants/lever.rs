use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::perf::perf_timer_start;
use crate::perf::perf_timer_stop;
use crate::perf::PERF_TIMER;
use crate::simulation::common::helpers::resource_allocation::allocate_stored_resources;
use crate::simulation::common::helpers::resource_allocation::StoredResourceAllocationMethod;
use crate::simulation::common::helpers::unit_behavior_execution::behavior_execution;
use crate::simulation::common::*;
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
    pub manifest: ChemistryManifest,
    pub configuration: ChemistryConfiguration,
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
                unit_behavior_arg!(ConstantNum),
            ),
        ),
    }
}
impl LeverChemistry {}

impl Chemistry for LeverChemistry {
    fn construct(config: ChemistryConfiguration) -> Box<LeverChemistry> {
        let mut chemistry = LeverChemistry {
            manifest: LeverChemistry::construct_manifest(&config),
            configuration: config,
        };
        wrap_chemistry!(chemistry)
    }

    fn get_key() -> String {
        "lever".to_string()
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn construct_manifest(config: &ChemistryConfiguration) -> ChemistryManifest
    where
        Self: Sized,
    {
        let mut manifest = ChemistryManifest {
            chemistry_key: Self::get_key(),
            all_properties: vec![],
            simulation_attributes: defs::SimulationAttributesLookup::make_defs(),
            unit_entry_attributes: defs::UnitEntryAttributesLookup::make_defs(),
            action_manifest: ActionManifest::new(Self::construct_action_library()),
            unit_resources: defs::UnitResourcesLookup::make_defs(),
            unit_attributes: defs::UnitAttributesLookup::make_defs(),
            position_attributes: defs::PositionAttributesLookup::make_defs(),
            position_resources: defs::PositionResourcesLookup::make_defs(),
            reactions: defs::get_reactions(),
        };
        manifest.normalize_manifest(config);
        manifest
    }

    // fn get_next_unit_resources(
    //     &self,
    //     entry: &UnitEntryData,
    //     pos: &Position,
    //     unit: &Unit,
    //     world: &World,
    //     tick_multiplier: u32,
    // ) -> UnitResources {
    //     unit.resources.clone()
    // }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }

    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    fn init_world_custom(&self, world: &mut World) {}

    fn get_default_unit_seed_attributes(
        &self,
        world: &mut World,
        coord: &Coord,
        entry: &UnitEntryData,
    ) -> UnitAttributes {
        self.get_manifest().unit_attributes_of(vec![])
    }

    fn on_simulation_tick(&self, sim: &mut SimCell) -> bool {
        perf_timer_start!("allocate_stored_resources");
        allocate_stored_resources(
            sim,
            sim.unit_manifest,
            &StoredResourceAllocationMethod::Every,
        );
        perf_timer_stop!("allocate_stored_resources");

        perf_timer_start!("unit_behavior_execution");
        behavior_execution(sim);
        perf_timer_stop!("unit_behavior_execution");

        true
    }

    fn on_simulation_finish(&self, sim: &mut SimCell) {}

    fn custom_action_definitions() -> Vec<ActionDefinition>
    where
        Self: Sized,
    {
        vec![ActionDefinition::new(
            &"pull_lever",
            vec![],
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let unit = sim_cell.world.get_unit_at(context.coord).unwrap();
                    let entry_id = unit.entry_id;

                    let uea_lookup = defs::UnitEntryAttributesLookup::new();
                    let to_add = context.params[0].to_constant();
                    // println!("pulling lever: {}", to_add);
                    // panic!("pulled a lever");
                    let existing = sim_cell.unit_entry_attributes[entry_id as usize]
                        [uea_lookup.lever_pulls]
                        .unwrap_integer();
                    sim_cell.unit_entry_attributes[entry_id][uea_lookup.lever_pulls] =
                        AttributeValue::Integer(existing + to_add);
                    true
                },
            ),
        )]
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::biology::unit_behavior::lever::SimpleLever;
    use crate::chemistry::actions::tests::execute_action;
    use crate::chemistry::actions::*;
    use crate::fixtures;
    use crate::simulation::common::builder::ChemistryBuilder;

    #[test]
    fn do_action() {
        let mut sim = SimulationBuilder::default()
            .chemistry(ChemistryBuilder::with_key("lever").build())
            .unit_entries(vec![UnitEntryBuilder::default()
                .species_name("main".to_string())
                .behavior(Rc::new(Box::new(SimpleLever::construct())))])
            .size((1, 1))
            .iterations(10)
            .to_simulation();

        assert!(sim.world.has_unit_at(&(0, 0)));

        let actions = LeverChemistry::custom_action_definitions();
        let action = actions
            .iter()
            .find(|a| a.key == "pull_lever".to_string())
            .unwrap();

        assert!(execute_action(
            &action,
            &(0, 0),
            &mut sim,
            vec![ActionParam::Constant(1)].as_slice()
        ));
        assert_eq!(sim.unit_entry_attributes[0][0].unwrap_integer(), 1);

        assert!(execute_action(
            &action,
            &(0, 0),
            &mut sim,
            vec![ActionParam::Constant(10)].as_slice()
        ));
        assert_eq!(sim.unit_entry_attributes[0][0].unwrap_integer(), 11);
    }
}

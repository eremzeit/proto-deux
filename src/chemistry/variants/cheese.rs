use crate::chemistry::actions::ActionSet;
use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::simulation::common::helpers::phenotype_execution::phenotype_execution;
use crate::simulation::common::helpers::place_units::place_units;
use crate::simulation::common::helpers::resource_allocation::allocate_stored_resources;
use crate::simulation::common::helpers::resource_allocation::StoredResourceAllocationMethod;
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
pub mod constants {

    #[macro_export]
    macro_rules! MAX_GOBBLE_AMOUNT_ {
        () => {
            5
        };
    }
    #[macro_export]
    macro_rules! MOVE_COST_ {
        () => {
            5
        };
    }
    #[macro_export]
    //macro_rules! NEW_UNIT_COST { () => {200} }
    macro_rules! NEW_UNIT_COST_ {
        () => {
            500
        };
    }
    pub const MOVE_COST: i32 = MOVE_COST_!();
    pub const NEW_UNIT_COST: i32 = NEW_UNIT_COST_!();
    pub const MAX_GOBBLE_AMOUNT: i32 = 30;
}

pub struct CheeseChemistry {
    manifest: ChemistryManifest,
    place_units_method: PlaceUnitsMethod,
    configuration: ChemistryConfiguration,
}

pub mod defs {
    use super::*;

    const CHEMISTRY_KEY: &str = "cheese";

    def_unit_entry_attributes! {[
        [total_cheese_consumed, Number]
    ]}

    def_simulation_attributes! {[
        [total_cheese_consumed, Number]
    ]}

    def_unit_attributes! {[
        [rolling_consumption, Number]
    ]}

    def_position_attributes! {[
        [is_cheese_source, Boolean],
        [is_air_source, Boolean]
    ]}

    def_position_resources! {[
        [cheese, false],
        [water, false]
    ]}

    def_unit_resources! {[
       [cheese, false],
       [air, true]
    ]}
    pub const REACTION_ID_GOBBLE_CHEESE: ReactionId = 0;
    pub const REACTION_ID_MOVE_UNIT: ReactionId = 1;
    pub const REACTION_ID_NEW_UNIT: ReactionId = 2;
    //trace_macros!(true);
    def_reactions! {
        reaction!("gobble_cheese",
            reagent!("gobble_cheese"),
        ),

        reaction!("move_unit",
            reagent!("offset_unit_resource",
                param_value!(UnitResourceKey, "cheese"),
                param_value!(UnitResourceAmount, -constants::MOVE_COST),
                param_value!(Boolean, false),
            ),
            reagent!("move_unit",
                phenotype_arg!(Direction)
            ),
        ),

        reaction!("new_unit",
            reagent!("offset_unit_resource",
                param_value!(UnitResourceKey, "cheese"),
                param_value!(UnitResourceAmount, -constants::NEW_UNIT_COST),
                param_value!(Boolean, false),
            ),
            reagent!("new_unit",
                phenotype_arg!(Direction),
            ),
        ),
    }
    //trace_macros!(false);
}
impl CheeseChemistry {
    pub fn construct(
        place_units_method: PlaceUnitsMethod,
        config: ChemistryConfiguration,
    ) -> ChemistryInstance {
        let mut chemistry = CheeseChemistry {
            manifest: CheeseChemistry::default_manifest(),
            place_units_method: place_units_method,
            configuration: config,
        };

        chemistry.init_manifest();
        wrap_chemistry!(chemistry)
    }

    fn get_unit_placement(&self) -> PlaceUnitsMethod {
        self.place_units_method.clone()
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
            &"gobble_cheese",
            vec![],
            // execute action
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let unit_resources = defs::UnitResourcesLookup::new();
                    let pos_resources = defs::PositionResourcesLookup::new();
                    let sim_attributes = defs::SimulationAttributesLookup::new();

                    let max_gobble_amount = constants::MAX_GOBBLE_AMOUNT;
                    let pos = sim_cell.world.get_position_at(context.coord).unwrap();

                    let pos_cheese_amount =
                        pos.get_resource(pos_resources.cheese, sim_cell.world.tick);

                    let diff = pos_cheese_amount - constants::MAX_GOBBLE_AMOUNT;

                    let amount = if pos_cheese_amount >= constants::MAX_GOBBLE_AMOUNT {
                        constants::MAX_GOBBLE_AMOUNT
                    } else {
                        pos_cheese_amount
                    };

                    let new_pos_cheese = pos_cheese_amount - amount;
                    sim_cell.world.set_pos_resource_at(
                        context.coord,
                        pos_resources.cheese,
                        new_pos_cheese,
                    );
                    sim_cell.world.add_unit_resource_at(
                        context.coord,
                        unit_resources.cheese,
                        amount,
                    );

                    //println!("before: {:?}", sim_attr[sim_attributes.total_cheese_consumed]);

                    let next_val = sim_cell.attributes[sim_attributes.total_cheese_consumed]
                        .unwrap_integer()
                        + amount;
                    sim_cell.attributes[sim_attributes.total_cheese_consumed] =
                        SimulationAttributeValue::Integer(next_val);

                    true
                },
            ),
        )])
    }
}

impl Chemistry for CheeseChemistry {
    fn get_key(&self) -> String {
        "cheese".to_string()
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn get_unit_placement(&self) -> PlaceUnitsMethod {
        self.place_units_method.clone()
    }
    // fn construct_specs(
    //     &self,
    //     unit_placement: &PlaceUnitsMethod,
    // ) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
    //     vec![
    //         Box::new(PlaceUnits {
    //             method: unit_placement.clone(),
    //         }),
    //         Box::new(ResourceAllocation {
    //             stored_method: StoredResourceAllocationMethod::Every,
    //         }),
    //         Box::new(PhenotypeExecution {}),
    //         Box::new(specs::PostTick {}),
    //     ]
    // }
    fn get_next_unit_resources(
        &self,
        entry: &UnitEntryData,
        pos: &Position,
        unit: &Unit,
        world: &World,
        tick_multiplier: u32,
    ) -> UnitResources {
        //println!("unit resources before: {:?}", &unit.resources);

        // is_air_source
        let mut resources = unit.resources.clone();

        let position_attributes = defs::PositionAttributesLookup::new();
        let unit_resources = defs::UnitResourcesLookup::new();
        let unit_attributes = defs::UnitAttributesLookup::new();
        let sim_attributes = defs::SimulationAttributesLookup::new();
        let position_resources = defs::PositionResourcesLookup::new();

        let is_air_source = pos
            .get_attribute(position_attributes.is_air_source)
            .unwrap_bool();

        if is_air_source {
            resources[unit_resources.air] = 10;
        } else {
            resources[unit_resources.air] = std::cmp::max(resources[unit_resources.air] - 1, 0);
        }

        let is_cheese_source = pos
            .get_attribute(position_attributes.is_cheese_source)
            .unwrap_bool();
        //println!("is_cheese_source: {}", is_cheese_source);
        //let id_cheese: PositionAttributeIndex = self.get_manifest().unit_resource_by_key("cheese").id as usize;
        //println!("id_air: {}", id_air);
        //println!("id_cheese: {}", id_cheese);

        if is_cheese_source {
            resources[unit_resources.cheese] += 20;
        }

        // println!("resources: {:?}", resources);
        resources
    }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }
    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    // fn get_base_streamed_resource_allocation(
    //     &self,
    //     world: &mut World,
    //     coord: &Coord,
    // ) -> SomeUnitResources {
    //     return self.manifest.unit_resources_of(vec![("air", 11)]);
    // }

    // fn get_base_stored_resource_allocation(
    //     &self,
    //     world: &mut World,
    //     coord: &Coord,
    // ) -> SomeUnitResources {
    //     return self.manifest.unit_resources_of(vec![("cheese", 50)]);
    // }

    fn on_simulation_init(&self, sim: &mut SimCell) {
        self.init_pos_properties(&mut sim.world);
        self.init_world_custom(&mut sim.world);
        self.init_units(sim);
    }

    fn on_simulation_tick(&self, sim: &mut SimCell) {
        allocate_stored_resources(
            sim,
            sim.unit_manifest,
            &StoredResourceAllocationMethod::Every,
        );
        phenotype_execution(sim);

        let unit_resources = defs::UnitResourcesLookup::new();
        for coord in CoordIterator::new(sim.world.size) {
            let pos = sim.world.get_position_at(&coord).unwrap();
            if let Some(unit) = sim.world.get_unit_at(&coord) {
                let val = unit.get_resource(unit_resources.cheese);
                if val <= 50 {
                    //println!("destroying unit");
                    sim.world.destroy_unit(&coord);
                }
            }
        }

        // let pos_resources = defs::UnitResourcesLookup::new();
        // let cheese = sim.world.get_pos_resource_at(&(0, 0), pos_resources.cheese);
    }

    fn on_simulation_finish(&self, sim: &mut SimCell) {}

    fn init_units(&self, sim: &mut SimCell) {
        place_units(sim, &self.place_units_method);
    }

    fn init_world_custom(&self, world: &mut World) {
        for coord in CoordIterator::new(world.size.clone()) {
            if (coord.0 * world.size.1 + coord.1) % 2 == 0 {
                world.set_pos_attribute_at(
                    &coord,
                    self.get_manifest()
                        .position_attribute_by_key("is_air_source")
                        .id as usize,
                    PositionAttributeValue::Bool(true),
                );
            }

            use rand::Rng;
            let mut rng = rand::thread_rng();
            let is_bottom_left = coord.0 == 0 && coord.1 == 0 || coord.0 == 1 && coord.1 == 0;
            if is_bottom_left || rng.gen_range(0..(coord.0 + coord.1) % 5 + 10) == 0 {
                world.set_pos_attribute_at(
                    &coord,
                    self.get_manifest()
                        .position_attribute_by_key("is_cheese_source")
                        .id as usize,
                    PositionAttributeValue::Bool(true),
                );
            }

            if is_bottom_left || rng.gen_range(0..(coord.0 + coord.1) % 5 + 5) == 0 {
                let position_resources = defs::PositionResourcesLookup::new();
                world.set_pos_resource_tab_offset(&coord, position_resources.cheese, 2);
            }
        }
    }

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
        self.get_manifest().unit_attributes_of(vec![(
            "rolling_consumption",
            UnitAttributeValue::Integer(0),
        )])
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::chemistry::actions::*;
    #[test]
    fn make_cheese_manifest() {
        let cheese =
            CheeseChemistry::construct(PlaceUnitsMethod::Skip, ChemistryConfiguration::new());
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

            let actions = CheeseChemistry::custom_actions();
            let action = actions.by_key("gobble_cheese");

            let src_coord = (2, 0);

            let mut sim =
                fixtures::default_base_with_unit_placement(PlaceUnitsMethod::ManualSingleEntry {
                    attributes: None,
                    coords: vec![src_coord],
                });

            sim.world
                .set_pos_resource_at(&(2, 0), position_resources.cheese, 10);
            let params = vec![];
            assert_eq!(
                sim.world
                    .get_unit_resource_at(&(2, 0), unit_resources.cheese),
                0
            );

            assert!(execute_action(&action, &src_coord, &mut sim, &params));

            assert_eq!(
                sim.attributes[sim_attributes.total_cheese_consumed].unwrap_integer(),
                10
            );

            assert_eq!(
                sim.world
                    .get_unit_resource_at(&(2, 0), unit_resources.cheese),
                10,
                "unit cheese is incorrect"
            );

            assert_eq!(
                sim.world
                    .get_pos_resource_at(&(2, 0), position_resources.cheese),
                0,
                "position cheese is not correct"
            );
        }
    }
}

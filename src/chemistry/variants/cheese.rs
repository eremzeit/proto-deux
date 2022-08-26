use crate::chemistry::actions::ActionSet;
use crate::chemistry::actions::*;
use crate::chemistry::properties::*;
use crate::chemistry::reactions::*;
use crate::chemistry::*;

use crate::simulation::common::helpers::place_units::place_pct_region;
use crate::simulation::common::helpers::place_units::place_units;
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
            1
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
                unit_behavior_arg!(Direction)
            ),
        ),

        reaction!("new_unit",
            reagent!("offset_unit_resource",
                param_value!(UnitResourceKey, "cheese"),
                param_value!(UnitResourceAmount, -constants::NEW_UNIT_COST),
                param_value!(Boolean, false),
            ),
            reagent!("new_unit",
                unit_behavior_arg!(Direction),
            ),
        ),
    }
    //trace_macros!(false);
}
impl CheeseChemistry {
    pub fn unit_drop_area(&self, world: &World) -> [Coord; 2] {
        let x_size = if world.size.0 > 10 { 5 } else { world.size.0 };
        let y_size = if world.size.1 > 10 { 5 } else { world.size.1 };

        let x = (world.size.0 - x_size) / 2;
        let y = (world.size.1 - y_size) / 2;

        [(x, y), (x + x_size, y + y_size)]
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
                    let unit_entry_attributes = defs::UnitEntryAttributesLookup::new();

                    let max_gobble_amount = constants::MAX_GOBBLE_AMOUNT;
                    let pos = sim_cell.world.get_position_at(context.coord).unwrap();
                    let entry_id = &pos.unit.as_ref().unwrap().entry_id.clone();
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

                    let next_val = sim_cell.unit_entry_attributes[*entry_id]
                        [unit_entry_attributes.total_cheese_consumed]
                        .unwrap_integer()
                        + amount;

                    sim_cell.unit_entry_attributes[*entry_id]
                        [unit_entry_attributes.total_cheese_consumed] =
                        UnitEntryAttributeValue::Integer(next_val);

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

    fn construct(config: ChemistryConfiguration) -> Box<CheeseChemistry> {
        let mut chemistry = CheeseChemistry {
            manifest: CheeseChemistry::construct_manifest(&config),
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

        manifest.normalize_manifest(config); // TODO make this accect a config.

        manifest
    }

    fn default_manifest() -> ChemistryManifest {
        Self::construct_manifest(&ChemistryConfiguration::new())
    }

    fn default_config() -> ChemistryConfiguration {
        ChemistryConfiguration::new()
    }

    fn get_configuration(&self) -> ChemistryConfiguration {
        self.configuration.clone()
    }

    fn custom_place_units(&self, sim: &mut SimCell) {
        let area = self.unit_drop_area(sim.world);
        place_units_static_region(sim.world, self, sim.unit_manifest, &None, 2, &area);
    }

    fn get_default_place_units_method(&self) -> PlaceUnitsMethod {
        PlaceUnitsMethod::Chemistry
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

        let is_air_source = pos
            .get_attribute(position_attributes.is_air_source)
            .unwrap_bool();

        let is_cheese_source = pos
            .get_attribute(position_attributes.is_cheese_source)
            .unwrap_bool();

        let has_unit = pos.has_unit();

        let unit = pos.unit.as_mut().unwrap();

        let resources = &mut unit.resources;

        if is_air_source {
            resources[unit_resources.air] = 10;
        } else {
            resources[unit_resources.air] = std::cmp::max(resources[unit_resources.air] - 1, 0);
        }

        if is_cheese_source {
            let amount = 50;
            resources[unit_resources.cheese] += amount;

            sim.unit_entry_attributes[unit.entry_id]
                [unit_entry_attributes.total_cheese_consumed] +=
                UnitAttributeValue::Integer(amount);
        }
    }

    fn get_manifest(&self) -> &ChemistryManifest {
        &self.manifest
    }
    fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
        &mut self.manifest
    }

    fn on_simulation_init(&self, sim: &mut SimCell) {
        self.init_pos_properties(&mut sim.world);
        self.init_world_custom(&mut sim.world);
    }

    fn on_simulation_tick(&self, sim: &mut SimCell) {
        allocate_stored_resources(
            sim,
            sim.unit_manifest,
            &StoredResourceAllocationMethod::Every,
        );

        behavior_execution(sim);

        let unit_resources = defs::UnitResourcesLookup::new();
        for coord in CoordIterator::new(sim.world.size) {
            let pos = sim.world.get_position_at(&coord).unwrap();
            if let Some(unit) = sim.world.get_unit_at(&coord) {
                let val = unit.get_resource(unit_resources.cheese);
                if val <= 50 {
                    // println!("destroying unit");
                    sim.world.destroy_unit(&coord);
                }
            }
        }

        // let pos_resources = defs::UnitResourcesLookup::new();
        // let cheese = sim.world.get_pos_resource_at(&(0, 0), pos_resources.cheese);
    }

    fn on_simulation_finish(&self, sim: &mut SimCell) {}

    // fn init_units(&self, sim: &mut SimCell) {
    //     place_units(sim, &self.place_units_method);
    // }

    fn init_world_custom(&self, world: &mut World) {
        let unit_drop_area = self.unit_drop_area(&world);

        use rand::Rng;
        let mut rng = rand::thread_rng();
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

            let is_unit_drop_area = coord.0 >= unit_drop_area[0].0
                && coord.0 < unit_drop_area[1].0
                && coord.1 >= unit_drop_area[0].1
                && coord.1 < unit_drop_area[1].1;

            // if !is_unit_drop_area && rng.gen_range(0..(coord.0 + coord.1) % 5 + 5) == 0 {
            if rng.gen_range(0..(coord.0 + coord.1) % 5 + 5) == 0 {
                world.set_pos_attribute_at(
                    &coord,
                    self.get_manifest()
                        .position_attribute_by_key("is_cheese_source")
                        .id as usize,
                    PositionAttributeValue::Bool(true),
                );
            }

            // if !is_unit_drop_area && rng.gen_range(0..(coord.0 + coord.1) % 5 + 1) == 0 {
            if rng.gen_range(0..(coord.0 + coord.1) % 5 + 1) == 0 {
                let position_resources = defs::PositionResourcesLookup::new();
                world.set_pos_resource_tab_offset(&coord, position_resources.cheese, 2, Some(60));
            }
        }
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

pub fn place_units_static_region(
    world: &mut World,
    chemistry: &CheeseChemistry,
    unit_manifest: &UnitManifest,
    attributes: &Option<UnitAttributes>,
    units_per_entry: u32,
    region_rect: &[Coord; 2],
) {
    use rand::Rng;
    let manifest = unit_manifest.clone();
    let mut rng = rand::thread_rng();
    let mut attempts = 0;

    // println!("[PlaceUnits] placing units in region: {:?}", rect);

    let max_attempts = manifest.units.len() * units_per_entry as usize * 100;

    for (i, unit_entry) in manifest.units.iter().enumerate() {
        for i in 0..units_per_entry {
            loop {
                let x1 = region_rect[0].0;
                let x2 = region_rect[1].0;
                let y1 = region_rect[0].1;
                let y2 = region_rect[1].1;

                let coord = (rng.gen_range(x1..x2), rng.gen_range(y1..y2));
                let can_place = !world.has_unit_at(&coord);
                let a = Box::new(&1).as_ref();

                if can_place {
                    world.seed_unit_at(&coord, &unit_entry.info, attributes.clone(), chemistry);
                    break;
                } else {
                    attempts += 1;
                    if attempts > max_attempts {
                        panic!(
                            "Random unit placement failed too many times within rect: {:?}",
                            &region_rect
                        );
                    }
                }
            }
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    use crate::chemistry::actions::*;
    #[test]
    fn make_cheese_manifest() {
        let cheese = CheeseChemistry::construct(ChemistryConfiguration::new());
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

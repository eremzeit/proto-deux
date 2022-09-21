// use crate::chemistry::actions::*;
// use crate::chemistry::actions::{ActionSet};
// use crate::chemistry::properties::*;
// use crate::chemistry::reactions::*;
// use crate::chemistry::*;
//
// use crate::simulation::common::*;
// use crate::simulation::unit::*;
// use crate::simulation::world::World;
// use crate::simulation::Simulation;
// use crate::util::Coord;
//
// use std::rc::Rc;
//
// use crate::simulation::position::{
//     PositionAttributeIndex, PositionAttributeValue, PositionResourceAmount, PositionResourceIndex,
// };
//
// use crate::chemistry::properties::{PositionAttributeDefinition};
//
// use crate::simulation::unit::{
//     UnitAttributeIndex, UnitAttributeValue, UnitResourceAmount, UnitResourceIndex,
// };
// use crate::util::*;
// use std::collections::HashMap;
//
// #[macro_use]
// pub mod constants {
// }
//
//
// pub struct SimpleChemistry {
//     manifest: ChemistryManifest,
// }
//
// pub mod defs {
//     use super::*;
//
//     const REACTION_ID_GOBBLE_CHEESE: ReactionId = 0;
//     const REACTION_ID_MOVE_UNIT: ReactionId = 1;
//     const REACTION_ID_NEW_UNIT: ReactionId = 2;
//
//
//     def_unit_attributes!{[
//         [total_flowers_collected, Number]
//     ]}
//
//     def_position_attributes!{[
//         [has_flowers, Boolean]
//     ]}
//
//     def_position_resources!{[
//         [flowers, false]
//     ]}
//
//     def_unit_resources!{[
//        [flowers, false]
//     ]}
//
//     def_reactions!{
//         reaction!("make_cheese",
//             reagent!("make_cheese"),
//         ),
//
//         reaction!("move_unit",
//             reagent!("update_unit_resources",
//                 //reagent_value!(UnitResourceKey("cheese")),
//                 param_value!(UnitResourceKey, "cheese"),
//                 param_value!(UnitResourceAmount, -MOVE_COST!()),
//             ),
//             reagent!("move",
//                 unit_behavior_arg!(Direction)
//             ),
//         ),
//
//         reaction!("new_unit",
//             reagent!( "update_unit_resources",
//                 param_value!(UnitResourceKey, "cheese"),
//                 param_value!(UnitResourceAmount, -NEW_UNIT_COST!()),
//             ),
//             reagent!("new_unit",
//                 unit_behavior_arg!(Direction),
//             ),
//         ),
//     }
// }
//
// impl CheeseChemistry {
//     pub fn construct() -> ChemistryInstance {
//         wrap_chemistry!(CheeseChemistry {
//             manifest: CheeseChemistry::default_manifest(),
//         })
//     }
//     pub fn default_manifest() -> ChemistryManifest {
//         let mut manifest = ChemistryManifest {
//             action_set: default_actions().add(Self::custom_actions().actions),
//             unit_resources: defs::UnitResourcesLookup::make_defs(),
//             unit_attributes: defs::UnitAttributesLookup::make_defs(),
//             position_attributes:  defs::PositionAttributesLookup::make_defs(),
//             position_resources:  defs::PositionResourcesLookup::make_defs(),
//             reactions: defs::get_reactions(),
//         };
//
//         manifest
//     }
//
//     pub fn custom_actions() -> ActionSet {
//
//         ActionSet::from(
//             vec![
//                 ActionDefinition::new(
//                     &"make_cheese",
//                     vec![],
//                     // can execute action
//                     Rc::new(
//                         |sim: &Simulation, coord: &Coord, params: &[ActionParam]| -> bool {
//                             true
//                         },
//                     ),
//                     // execute action
//                     Rc::new(
//                         |sim: &mut Simulation, coord: &Coord, params: &[ActionParam]| -> bool {
//                             let unit_resources = defs::UnitResourcesLookup::new();
//                             let pos_resources = defs::PositionResourcesLookup::new();
//
//                             let max_make_cheese_amount = MAX_GOBBLE_AMOUNT!();
//                             let pos = sim.world.get_position_at(coord).unwrap();
//                             let pos_cheese_amount = pos.get_resource(pos_resources.cheese);
//
//
//                             let diff = pos_cheese_amount - MAX_GOBBLE_AMOUNT!();
//
//                             let amount = if pos_cheese_amount >= MAX_GOBBLE_AMOUNT!() {
//                                 MAX_GOBBLE_AMOUNT!()
//                             } else {
//                                 pos_cheese_amount
//                             };
//
//
//                             let new_pos_cheese = pos_cheese_amount - amount;
//                             sim.world.set_pos_resource_at(coord, pos_resources.cheese, new_pos_cheese);
//                             sim.world.add_unit_resource_at(coord, unit_resources.cheese, amount);
//
//                             true
//                         },
//                     ),
//                 ),
//             ]
//         )
//     }
// }
//
// impl Chemistry for CheeseChemistry {
//     fn get_key(&self) -> String {
//         "cheese".to_string()
//     }
//
//     fn default_specs(&self) -> Vec<std::boxed::Box<dyn SimulationSpec>> {
//         vec![Box::new(PlaceUnits {
//             method: PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
//         })]
//     }
//
//     fn get_next_unit_resources(
//         &self,
//         entry: &UnitEntryData,
//         pos: &Position,
//         unit: &Unit,
//         world: &World,
//         tick_multiplier: u32,
//     ) -> UnitResources {
//         //println!("unit resources before: {:?}", &unit.resources);
//
//         // is_air_source
//         let mut resources = unit.resources.clone();
//         let is_air_source_id: PositionAttributeIndex = self.get_manifest().position_attribute_by_key("is_air_source").id as usize;
//         let is_air_source = pos.get_attribute(is_air_source_id).unwrap_bool();
//
//         let id_air: PositionAttributeIndex = self.get_manifest().unit_resource_by_key("air").id as usize;
//
//         if is_air_source {
//             resources[id_air] = 10;
//         }
//
//         // is_cheese_dispenser
//         let is_cheese_dispenser_id: PositionAttributeIndex = self.get_manifest().position_attribute_by_key("is_cheese_dispenser").id as usize;
//         let is_cheese_dispenser = pos.get_attribute(is_cheese_dispenser_id).unwrap_bool();
//
//         //println!("is_cheese_dispenser: {}", is_cheese_dispenser);
//
//         let id_cheese: PositionAttributeIndex = self.get_manifest().unit_resource_by_key("cheese").id as usize;
//         //println!("id_air: {}", id_air);
//         //println!("id_cheese: {}", id_cheese);
//
//         if is_cheese_dispenser {
//             resources[id_cheese] += 50;
//         }
//
//         println!("resources: {:?}", resources);
//         resources
//     }
//
//     fn get_manifest(&self) -> &ChemistryManifest {
//         &self.manifest
//     }
//
//     fn get_manifest_mut(&mut self) -> &mut ChemistryManifest {
//         &mut self.manifest
//     }
//
//     fn get_base_streamed_resource_allocation(
//         &self,
//         world: &mut World,
//         coord: &Coord,
//     ) -> SomeUnitResources {
//         return self.manifest.unit_resources_of(vec![("air", 11)]);
//     }
//
//     fn get_base_stored_resource_allocation(
//         &self,
//         world: &mut World,
//         coord: &Coord,
//     ) -> SomeUnitResources {
//         return self.manifest.unit_resources_of(vec![("cheese", 50)]);
//     }
//
//     fn init_world_custom(&self, world: &mut World, size: &GridSize2D) {
//         for coord in CoordIterator::new(size.clone()) {
//             if (coord.0 * size.1 + coord.1) % 20 == 0 {
//                 world.set_pos_attribute_at(
//                     &coord,
//                     self.get_manifest()
//                         .position_attribute_by_key("is_air_source")
//                         .id as usize,
//                     PositionAttributeValue::Bool(true),
//                 );
//             }
//             if (coord.0 + coord.1) % 3 == 0 {
//                 world.set_pos_attribute_at(
//                     &coord,
//                     self.get_manifest()
//                         .position_attribute_by_key("is_cheese_dispenser")
//                         .id as usize,
//                     PositionAttributeValue::Bool(true),
//                 );
//             }
//         }
//     }
//
//     fn get_default_unit_seed_attributes(
//         &self,
//         world: &mut World,
//         coord: &Coord,
//         entry: &UnitEntryData,
//     ) -> UnitAttributes {
//         self.get_manifest().unit_attributes_of(vec![(
//             "rolling_consumption",
//             UnitAttributeValue::Integer(0),
//         )])
//     }
// }
//
// mod tests {
//     #[allow(unused_imports)]
//     use super::*;
//     use crate::chemistry::actions::*;
//
//     #[test]
//     fn make_cheese_manifest() {
//         let cheese = CheeseChemistry::construct();
//     }
//
//     #[test]
//     fn macros() {
//         let unit_resources = defs::UnitResourcesLookup::make_defs();
//         let unit_attributes = defs::UnitAttributesLookup::make_defs();
//         let position_attributes = defs::PositionAttributesLookup::make_defs();
//         let position_resources = defs::PositionResourcesLookup::make_defs();
//     }
//
//     mod make_cheese {
//         use super::*;
//         use tests::{can_execute, execute_action};
//         use crate::tests::fixtures;
//
//         #[test]
//         fn do_action() {
//             let unit_attributes = defs::UnitAttributesLookup::new();
//             let position_attributes = defs::PositionAttributesLookup::new();
//             let position_resources = defs::PositionResourcesLookup::new();
//             let unit_resources = defs::UnitResourcesLookup::new();
//
//             let actions = CheeseChemistry::custom_actions();
//             let action = actions.by_key("make_cheese");
//
//             let src_coord = (2,0);
//             let mut sim = fixtures::default_base(Some(vec![
//                 Box::new(PlaceUnits {
//                     method: PlaceUnitsMethod::ManualSingleGenome {
//                         attributes: None,
//                         coords: vec![src_coord],
//                     },
//                 })
//             ]));
//
//             sim.world.set_pos_resource_at(&(2,0), position_resources.cheese, 10);
//
//             let params = vec![];
//
//             assert_eq!(
//                 sim.world.get_unit_resource_at(&(2,0), unit_resources.cheese),
//                 0
//             );
//
//             assert!(can_execute(&action, &src_coord, &sim, &params));
//             assert!(execute_action(&action, &src_coord, &params, &mut sim));
//
//             assert_eq!(
//                 sim.world.get_unit_resource_at(&(2,0), unit_resources.cheese),
//                 10, "unit cheese is incorrect"
//             );
//
//             assert_eq!(
//                 sim.world.get_pos_resource_at(&(2,0), position_resources.cheese),
//                 0, "position cheese is not correct"
//             );
//
//         }
//     }
// }

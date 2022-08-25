pub mod tests;

use std::rc::Rc;

use crate::chemistry::ChemistryInstance;

use crate::simulation::common::SimCell;
use crate::simulation::config::SimulationConfig;
use crate::simulation::position::{
    PositionAttributeIndex, PositionAttributeValue, PositionResourceAmount, PositionResourceIndex,
};
use crate::simulation::unit::{
    UnitAttributeIndex, UnitAttributeValue, UnitResourceAmount, UnitResourceIndex,
};
use crate::simulation::unit_entry::UnitManifest;
use crate::simulation::unit_entry::{
    UnitEntryAttributeIndex, UnitEntryAttributeValue, UnitEntryAttributes,
};
use crate::simulation::SimulationAttributes;

use crate::simulation::world::World;
use crate::simulation::Simulation;
use crate::util::Coord;
use crate::util::*;
use crate::HashMap;
use std::fmt::{Debug, Formatter, Result};

pub type ActionParamNumber = i32;
pub type ActionDefinitionIndex = usize;
pub type ExecuteActionFunction = dyn Fn(&mut SimCell, &ActionExecutionContext) -> bool;

#[derive(Clone, PartialEq, Debug)]
pub enum ActionParamType {
    UnitResourceAmount,
    UnitResourceIndex,
    UnitAttributeIndex,
    UnitAttributeValue,
    PositionResourceIndex,
    PositionResourceAmount,
    PositionAttributeIndex,
    PositionAttributeValue,
    Direction,
    ConstantNum,
    Boolean,
}

// actually passed to the action functions
// #[derive(Clone)]
// pub enum ActionParamValue {
//     UnitResourceAmount(ActionParamNumber),
//     UnitResourceIndex(UnitResourceIndex),
//     UnitResourceKey(&'static str),
//
//     UnitAttributeValue(UnitAttributeValue),
//     UnitAttributeIndex(UnitAttributeIndex),
//     UnitAttributeKey(&'static str),
//
//     PositionResourceAmount(PositionResourceAmount),
//     PositionResourceIndex(PositionResourceIndex),
//     PositionResourceKey(&'static str),
//
//     PositionAttributeIndex(PositionAttributeIndex),
//     PositionAttributeValue(PositionAttributeValue),
//     PositionAttributeKey(&'static str),
//
//     Constant(ActionParamNumber),
//     Direction(GridDirection),
//
//     Nil,
// }

#[derive(Clone, PartialEq, Debug)]
pub enum ActionParam {
    UnitResourceAmount(ActionParamNumber),
    UnitResourceIndex(UnitResourceIndex),
    UnitResourceKey(&'static str),

    UnitAttributeValue(UnitAttributeValue),
    UnitAttributeIndex(UnitAttributeIndex),
    UnitAttributeKey(&'static str),

    UnitEntryAttributeValue(UnitEntryAttributeValue),
    UnitEntryAttributeIndex(UnitEntryAttributeIndex),
    UnitEntryAttributeKey(&'static str),

    PositionResourceAmount(PositionResourceAmount),
    PositionResourceIndex(PositionResourceIndex),
    PositionResourceKey(&'static str),

    PositionAttributeIndex(PositionAttributeIndex),
    PositionAttributeValue(PositionAttributeValue),
    PositionAttributeKey(&'static str),

    SimulationAttributeIndex(PositionAttributeIndex),
    SimulationAttributeValue(PositionAttributeValue),
    SimulationAttributeKey(&'static str),

    Constant(ActionParamNumber),
    Boolean(bool),
    Direction(GridDirection),

    // Placeholder(ActionParamType),
    /**
     * Specifies when an argument is meant to be given by the unit_behavior.
     */
    UnitBehaviorArgument(ActionParamType),

    /**
     * Specifies when an argument is meant to be given by the chemistry.
     */
    ChemistryArgument(String, ActionParamType),

    Nil,
}

impl ActionParam {
    pub fn to_unit_resource_index(&self) -> UnitResourceIndex {
        if let ActionParam::UnitResourceIndex(num) = self {
            return *num;
        }

        panic!["value is incorrect type"];
    }

    pub fn to_unit_resource_amount(&self) -> UnitResourceAmount {
        if let ActionParam::UnitResourceAmount(num) = self {
            return *num;
        }
        panic!["value is incorrect type"];
    }

    pub fn to_unit_entry_attribute_value(&self) -> &UnitEntryAttributeValue {
        if let ActionParam::UnitEntryAttributeValue(val) = self {
            return val;
        }

        panic!["value is incorrect type"];
    }

    pub fn to_position_attribute_value(&self) -> &PositionAttributeValue {
        if let ActionParam::PositionAttributeValue(val) = self {
            return val;
        }

        panic!["value is incorrect type"];
    }

    pub fn to_direction(&self) -> GridDirection {
        match self {
            ActionParam::Direction(dir) => dir.clone(),
            _ => panic!["value is incorrect type"],
        }
    }

    pub fn to_constant(&self) -> ActionParamNumber {
        if let ActionParam::Constant(num) = self {
            return *num;
        }

        panic!["value is incorrect type"];
    }

    pub fn to_bool(&self) -> bool {
        if let ActionParam::Boolean(bool) = self {
            return *bool;
        }

        panic!["value is incorrect type"];
    }
}

#[derive(Clone, Debug)]
pub struct ActionParamDefinition {
    pub name: String,
    pub param_type: ActionParamType,
}

#[derive(Clone)]
pub struct ActionDefinition {
    pub key: String,
    pub index: ActionDefinitionIndex,
    pub execute: Rc<ExecuteActionFunction>,
    pub params: Vec<ActionParamDefinition>,
}

impl Debug for ActionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "ActionDefinition {{ key: {}, index: {}, params: {:?} }}",
            self.key, self.index, self.params
        )
    }
}

impl ActionDefinition {
    pub fn new(
        key: &str,
        params: Vec<ActionParamDefinition>,
        execute: Rc<ExecuteActionFunction>,
    ) -> ActionDefinition {
        ActionDefinition {
            key: key.to_string(),
            index: 0,
            params,
            execute,
        }
    }
}

#[derive(Clone)]
pub struct ActionSet {
    pub actions: Vec<ActionDefinition>,
    pub by_string_key: HashMap<String, usize>,
}

impl Debug for ActionSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ActionSet {{")?;
        for i in 0..self.actions.len() {
            write!(f, "\n    {:?}", self.actions[i])?;
        }

        write!(f, "}}")
    }
}

impl ActionSet {
    pub fn from(actions: Vec<ActionDefinition>) -> Self {
        to_action_set(actions)
    }

    pub fn add(mut self, mut actions: Vec<ActionDefinition>) -> Self {
        self.actions.append(&mut actions);

        self.normalize();
        self
    }

    /**
     * Populate the mapping between the action keys and the action index
     */
    pub fn normalize(&mut self) {
        for i in 0..self.actions.len() {
            self.actions[i].index = i;
            let key = &self.actions[i].key;
            self.by_string_key.insert(key.to_string(), i as usize);
        }
    }

    pub fn by_key(&self, key: &str) -> &ActionDefinition {
        let maybe_i = self.by_string_key.get(key);
        self.actions
            .get(*maybe_i.expect(&format!("Cannot find action for key: {}", key)))
            .unwrap()
    }
}

fn to_action_set<'a>(mut actions: Vec<ActionDefinition>) -> ActionSet {
    let mut by_string_key: HashMap<String, usize> = HashMap::new();

    let actions = actions
        .into_iter()
        .enumerate()
        .map(|(i, mut action)| -> ActionDefinition {
            action.index = i as ActionDefinitionIndex;
            by_string_key.insert(action.key.to_string(), i);
            action
        })
        .collect::<Vec<_>>();

    ActionSet {
        actions,
        by_string_key,
    }
}

/*
 * actions to implement:
 * new_unit
 * give_neighbor_resource
 * set_attr_to_constant
 * increase_max_load
 * decrease_load
 * convert_to_material
*/

// pub struct ActionExecutionContext<'a> {
//     pub world: &'a World,
// }

pub struct ActionExecutionContext<'a> {
    pub coord: &'a Coord,
    pub params: &'a [ActionParam],
    // pub chemistry: &'a ChemistryInstance,
    // pub unit_manifest: &'a UnitManifest,
}

/*
 * A public registry of actions.  Any action that isn't shared between
 * chemistries should be included in the specific chemistry.
 */
pub fn default_actions() -> ActionSet {
    let actions = vec![
        ActionDefinition::new(
            &"move_unit",
            vec![ActionParamDefinition {
                name: "direction".to_string(),
                param_type: ActionParamType::Direction,
            }],
            // execute action
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let dir = context.params[0].to_direction();
                    //println!("moving {:?}", dir);
                    match coord_by_direction_offset(context.coord, &dir, sim_cell.world.size) {
                        Some(dest_coord) => {
                            let pos = sim_cell.world.get_position_at(&dest_coord).unwrap();
                            if !sim_cell.world.has_unit_at(&dest_coord) {
                                //println!("moving {:?} to {:?}", &context.coord, &dest_coord);
                                sim_cell.world.move_unit(
                                    context.coord,
                                    &dest_coord,
                                    &sim_cell.chemistry,
                                );
                                true
                            } else {
                                false
                            }
                        }

                        None => false,
                    }
                },
            ),
        ),
        ActionDefinition::new(
            &"new_unit",
            vec![ActionParamDefinition {
                name: "direction".to_string(),
                param_type: ActionParamType::Direction,
            }],
            // execute action
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let dir = context.params[0].to_direction();

                    let dest_coord =
                        coord_by_direction_offset(context.coord, &dir, sim_cell.world.size);
                    //println!("dest coord ______________: {:?}", &dest_coord);

                    if let Some(_dest_coord) = dest_coord {
                        if !sim_cell.world.has_unit_at(&_dest_coord) {
                            sim_cell.world.copy_unit_with_attributes(
                                context.coord,
                                &_dest_coord,
                                &sim_cell.unit_manifest,
                                &sim_cell.chemistry,
                            );
                            return true;
                        }
                    }

                    false
                },
            ),
        ),
        ActionDefinition::new(
            &"set_unit_resource",
            vec![
                ActionParamDefinition {
                    name: "resource".to_string(),
                    param_type: ActionParamType::UnitResourceIndex,
                },
                ActionParamDefinition {
                    name: "amount".to_string(),
                    param_type: ActionParamType::UnitResourceAmount,
                },
            ],
            // execute action
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let unit = sim_cell
                        .world
                        .get_unit_at(context.coord)
                        .expect("expected unit to exist");
                    let resource_idx = context.params[0].to_unit_resource_index();
                    let amount = context.params[1].to_unit_resource_amount();

                    sim_cell
                        .world
                        .set_unit_resource_at(context.coord, resource_idx, amount);

                    true
                },
            ),
        ),
        ActionDefinition::new(
            &"offset_unit_resource",
            vec![
                ActionParamDefinition {
                    name: "resource".to_string(),
                    param_type: ActionParamType::UnitResourceIndex,
                },
                ActionParamDefinition {
                    name: "offset".to_string(),
                    param_type: ActionParamType::ConstantNum,
                },
                ActionParamDefinition {
                    name: "allow_negative".to_string(),
                    param_type: ActionParamType::Boolean,
                },
            ],
            // execute action
            Rc::new(
                |sim_cell: &mut SimCell, context: &ActionExecutionContext| -> bool {
                    let unit = sim_cell
                        .world
                        .get_unit_at(context.coord)
                        .expect("cant execute action where theres no unit");
                    let resource_idx =
                        context.params[0].to_unit_resource_index() as UnitResourceIndex;
                    let offset_amount =
                        context.params[1].to_unit_resource_amount() as UnitResourceAmount;

                    let current_amount = unit.resources[resource_idx];
                    let new_amount = (current_amount + offset_amount) as UnitResourceAmount;

                    let allow_negative = context.params[2].to_bool();

                    if allow_negative || new_amount >= 0 {
                        sim_cell.world.set_unit_resource_at(
                            context.coord,
                            resource_idx,
                            new_amount,
                        );
                        true
                    } else {
                        false
                    }
                },
            ),
        ),
    ];

    to_action_set(actions)
}

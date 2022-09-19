pub mod tests;

use std::rc::Rc;

use crate::chemistry::ChemistryInstance;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionParamDefinition {
    pub name: String,
    pub param_type: ActionParamType,
}

pub type ActionLibrary = Vec<ActionDefinition>;

#[derive(Clone)]
pub struct ActionDefinition {
    pub key: String,
    pub execute: Rc<ExecuteActionFunction>,
    pub params: Vec<ActionParamDefinition>,
}

impl ActionDefinition {
    pub fn new(
        key: &str,
        params: Vec<ActionParamDefinition>,
        execute: Rc<ExecuteActionFunction>,
    ) -> Self {
        Self {
            key: key.to_string(),
            params,
            execute,
        }
    }

    // pub fn to_compiled_action(&self, index: usize) -> CompiledActionAoeu {
    //     CompiledActionAoeu {
    //         key: self.key.clone(),
    //         index_aoeu: index,
    //         execute: self.execute.clone(),
    //         params: self.params.clone(),
    //     }
    // }
}

impl Debug for ActionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "ActionDefinition {{ key: {}, params: {:?} }}",
            self.key, self.params
        )
    }
}

#[derive(Clone)]
pub struct CompiledActionDefinition {
    pub key: String,
    pub index: ActionDefinitionIndex,
    pub execute: Rc<ExecuteActionFunction>,
    pub params: Vec<ActionParamDefinition>,
}

impl CompiledActionDefinition {
    pub fn from_action(action: ActionDefinition, index: usize) -> CompiledActionDefinition {
        CompiledActionDefinition {
            key: action.key.to_string(),
            index,
            params: action.params,
            execute: action.execute,
        }
    }
}

impl Debug for CompiledActionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "CompiledActionDefinition {{ key: {}, params: {:?}, index: {} }}",
            self.key, self.params, self.index
        )
    }
}

/**
 * A serializable set of actions which reperesents the complete
 * space of actions that are available.  It might include actions that
 * aren't used by the chemitry.
 */
#[derive(Clone, Serialize, Deserialize)]
pub struct ActionManifestData {
    pub actions: Vec<String>,
    pub by_string_key: HashMap<String, usize>,
}

impl ActionManifestData {
    pub fn new(keys: Vec<String>) -> Self {
        let mut by_string_key: HashMap<String, usize> = HashMap::new();

        for (i, key) in keys.iter().enumerate() {
            by_string_key.insert(key.to_string(), i);
        }

        Self {
            actions: keys,
            by_string_key,
        }
    }

    pub fn by_key(&self, key: &str) -> usize {
        *self.by_string_key.get(key).unwrap()
    }

    pub fn to_manifest(&self, action_library: &ActionLibrary) -> ActionManifest {
        if self.actions.len() != action_library.len() {
            panic!("Probably something went wrong.");
        }

        let actions = self
            .actions
            .iter()
            .enumerate()
            .map(|(i, key)| {
                let action_def = action_library
                    .iter()
                    .find(|def| &def.key == key)
                    .unwrap()
                    .clone();
                CompiledActionDefinition {
                    key: action_def.key,
                    index: i,
                    execute: action_def.execute,
                    params: action_def.params,
                }
            })
            .collect::<Vec<_>>();

        let mut by_string_key = HashMap::new();

        for (i, action) in actions.iter().enumerate() {
            by_string_key.insert(action.key.to_string(), i);
        }

        ActionManifest {
            actions,
            by_string_key,
        }
    }

    pub fn from_manifest(manifest: &ActionManifest) -> Self {
        let keys = manifest
            .actions
            .iter()
            .map(|action| action.key.to_string())
            .collect::<Vec<_>>();

        // let mut by_string_key = HashMap::new();

        // for (i, key) in keys.iter().enumerate() {
        //     by_string_key.insert(key.to_string(), i);
        // }

        Self {
            actions: keys,
            by_string_key: manifest.by_string_key.clone(),
        }
    }
}

/**
 * Includes the implementation.  Not serializable.
 */
#[derive(Clone)]
pub struct ActionManifest {
    pub actions: Vec<CompiledActionDefinition>,
    pub by_string_key: HashMap<String, usize>,
}

impl Debug for ActionManifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "ActionSet {{")?;
        for i in 0..self.actions.len() {
            write!(f, "\n    {:?}", self.actions[i])?;
        }

        write!(f, "}}")
    }
}

impl ActionManifest {
    pub fn new(action_defs: Vec<ActionDefinition>) -> Self {
        let actions = action_defs
            .into_iter()
            .enumerate()
            .map(|(i, action_def)| CompiledActionDefinition::from_action(action_def, i))
            .collect::<Vec<_>>();

        let mut set = Self {
            actions,
            by_string_key: HashMap::new(),
        };
        set.normalize();
        set
    }

    pub fn add(mut self, mut actions: Vec<ActionDefinition>) -> Self {
        let mut _actions = actions
            .into_iter()
            .enumerate()
            .map(|(i, action_def)| CompiledActionDefinition::from_action(action_def, 0))
            .collect::<Vec<_>>();

        self.actions.append(&mut _actions);

        self.normalize();
        self
    }

    /**
     * Populate the mapping between the action keys and the action index
     */
    pub fn normalize(&mut self) {
        for i in 0..self.actions.len() {
            self.actions[i].index = i;
            let key = self.actions[i].key.to_string();
            self.by_string_key.insert(key.to_string(), i as usize);
        }
    }

    pub fn by_key(&self, key: &str) -> &CompiledActionDefinition {
        let maybe_i = self.by_string_key.get(key);
        self.actions
            .get(*maybe_i.expect(&format!("Cannot find action for key: {}", key)))
            .unwrap()
    }
}

// fn to_compiled_action_set<'a>(mut actions: Vec<CompiledActionAoeu>) -> ActionDefinitionSet {
//     let mut by_string_key: HashMap<String, usize> = HashMap::new();

//     let actions = actions
//         .into_iter()
//         .enumerate()
//         .map(|(i, mut action)| -> CompiledActionAoeu {
//             action.index_aoeu = i as ActionDefinitionIndex;
//             by_string_key.insert(action.key.to_string(), i);
//             action
//         })
//         .collect::<Vec<_>>();

//     ActionDefinitionSet {
//         actions,
//         by_string_key,
//     }
// }

/*
 * actions to implement:
 * new_unit
 * give_neighbor_resource
 * set_attr_to_constant
 * increase_max_load
 * decrease_load
 * convert_to_material
*/

/*
 * A public registry of actions.  Any action that isn't shared between
 * chemistries should be included in the specific chemistry.
 */
pub fn default_actions() -> Vec<ActionDefinition> {
    vec![
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
                                sim_cell.chemistry.as_ref(),
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
                    // println!("current: {}, new: {}", current_amount, new_amount);

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
    ]
}

pub struct ActionExecutionContext<'a> {
    pub coord: &'a Coord,
    pub params: &'a [ActionParam],
    // pub chemistry: &'a ChemistryInstance,
    // pub unit_manifest: &'a UnitManifest,
}

use serde::{Deserialize, Serialize};

use crate::biology::unit_behavior::NUM_REACTION_PARAMS;
use crate::chemistry::actions::{ActionDefinitionIndex, ActionParam, ActionParamType};
use crate::chemistry::ReactionId;
use crate::simulation::common::*;
use crate::util::grid_direction_from_num;
use std::time::Instant;

#[derive(Debug, Clone)]
// #[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionDefinition {
    pub key: String,
    pub reagents: Vec<ReagentDefinition>,
    pub id: usize,
}

pub type CompiledReactionDefinition = ReactionDefinition;

pub type ReactionCall = (
    ReactionId,
    ReactionCallParam,
    ReactionCallParam,
    ReactionCallParam,
);
pub type ReactionCallParam = u16;

impl ReactionDefinition {
    pub fn new(key: &str, reagents: Vec<ReagentDefinition>) -> ReactionDefinition {
        ReactionDefinition {
            id: 0,
            key: key.to_string(),
            reagents: reagents,
        }
    }
}

pub type ReagentValue = u8;
pub type ActionIndex = u8;
//use crate::chemistry::actions::{};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReagentDefinition {
    // pub action_key: String,
    pub action_key: &'static str,

    // action_index needs to be set after initialization
    pub action_index: ActionDefinitionIndex,

    pub params: Vec<ActionParam>,
}

impl ReagentDefinition {
    pub fn new(action_key: &'static str, params: Vec<ActionParam>) -> ReagentDefinition {
        ReagentDefinition {
            action_key,
            params,
            action_index: 0,
        }
    }
}

pub fn execute_reaction(
    sim_cell: &mut SimCell,
    coord: &Coord,
    reaction: &ReactionDefinition,
    chemistry: &ChemistryInstance,
    unit_manifest: &UnitManifest,
    reaction_call: ReactionCall,
) {
    let action_params: Vec<[ActionParam; 3]> =
        replace_unit_behavior_placeholders(reaction, reaction_call);

    for (i, reagent) in reaction.reagents.iter().enumerate() {
        //println!("reagent: {:?} with INDEX {}", reagent, reagent.action_index);
        //println!("chemistry.get_manifest().action_set: {:?}", chemistry.get_manifest().action_set);

        // let action_def = &action_set.actions[reagent.action_index];
        //println!("action_def: {}", action_def.key);
        let result = execute_reagent(
            sim_cell,
            coord,
            reagent,
            chemistry,
            &action_params[i],
            unit_manifest,
        );

        if !result {
            break;
        }
    }
}

/**
 * Executed during each reaction call to fill in parameters that are meant
 * to be supplied by the unit behavior.
 */
fn replace_unit_behavior_placeholders(
    reaction: &ReactionDefinition,
    reaction_call: ReactionCall,
) -> Vec<[ActionParam; 3]> {
    let mut params_by_reagent: Vec<[ActionParam; 3]> = _empty_params_list(reaction.reagents.len());
    let mut param_idx: usize = 0;
    for (j, reagent) in reaction.reagents.iter().enumerate() {
        //println!("replacing placeholders for action: {}", reagent.action_key);
        let mut params = [ActionParam::Nil, ActionParam::Nil, ActionParam::Nil];

        for (i, param_def) in reagent.params.iter().enumerate() {
            if param_idx > NUM_REACTION_PARAMS as usize {
                panic!("too many reagent params");
            }
            //println!("param: {}", i);

            match &param_def {
                ActionParam::UnitBehaviorArgument(param_type) => {
                    //println!("placeholder");
                    let param = get_param_by_index(reaction_call, param_idx);
                    params[i] = convert_raw_arg_val_to_param_val(param, param_type);
                    //println!("params[i]: {:?}", &params[i]);

                    param_idx += 1;
                }
                _ => {
                    params[i] = param_def.clone();
                }
            }
        }

        params_by_reagent[j] = params;
    }

    //println!("params_by_reagent: {:?}", params_by_reagent);
    params_by_reagent
}

fn execute_reagent(
    sim_cell: &mut SimCell,
    coord: &Coord,
    reagent: &ReagentDefinition,
    chemistry: &ChemistryInstance,
    action_args: &[ActionParam; 3],
    unit_manifest: &UnitManifest,
) -> bool {
    //println!("Executing reagent ({}, {}) with params: {:?}", reagent.action_key, reagent.action_index, action_args);

    let context = ActionExecutionContext {
        coord,
        params: action_args,
    };

    let action = &chemistry.get_manifest().action_manifest.actions[reagent.action_index];
    //println!("^Executed reagent {}", reagent.action_key);
    (action.execute)(sim_cell, &context)
}

fn _empty_params_list(size: usize) -> Vec<[ActionParam; 3]> {
    let mut params: Vec<[ActionParam; 3]> = vec![];

    for i in (0..size) {
        params.push([ActionParam::Nil, ActionParam::Nil, ActionParam::Nil]);
    }

    params
}

fn convert_raw_arg_val_to_param_val(
    raw_val: ReactionCallParam,
    param_type: &ActionParamType,
) -> ActionParam {
    //println!("CONVERTING");
    //println!("raw_val: {:?}", raw_val);
    //println!("param_type: {:?}", param_type);
    match param_type {
        ActionParamType::UnitResourceAmount => {
            ActionParam::UnitResourceAmount(raw_val as UnitResourceAmount)
        }
        ActionParamType::PositionResourceAmount => {
            ActionParam::PositionResourceAmount(raw_val as PositionResourceAmount)
        }

        ActionParamType::PositionAttributeIndex => {
            ActionParam::PositionAttributeIndex(raw_val as PositionAttributeIndex)
        }
        ActionParamType::UnitAttributeIndex => {
            ActionParam::UnitAttributeIndex(raw_val as UnitAttributeIndex)
        }

        ActionParamType::UnitResourceIndex => {
            ActionParam::UnitResourceIndex(raw_val as UnitResourceIndex)
        }
        ActionParamType::PositionResourceIndex => {
            ActionParam::PositionResourceIndex(raw_val as PositionResourceIndex)
        }

        // until these are actually used, just convert to nils
        ActionParamType::PositionAttributeValue => {
            ActionParam::PositionAttributeValue(PositionAttributeValue::Nil)
        }
        ActionParamType::UnitAttributeValue => {
            ActionParam::UnitAttributeValue(UnitAttributeValue::Nil)
        }

        ActionParamType::Direction => {
            ActionParam::Direction(grid_direction_from_num(raw_val as u8 % 4))
        }
        ActionParamType::ConstantNum => ActionParam::Constant(raw_val as ActionParamNumber),
        ActionParamType::Boolean => ActionParam::Boolean(raw_val as ActionParamNumber > 0),
        //_ => { panic!("not supported"); }
    }
}

pub fn get_param_by_index(reaction_call: ReactionCall, i: usize) -> ReactionCallParam {
    match i {
        0 => reaction_call.1,
        1 => reaction_call.2,
        2 => reaction_call.3,
        _ => {
            panic!("invalid index");
        }
    }
}

pub mod tests {
    use super::*;
    use crate::simulation::common::*;
    #[test]
    fn replace_placeholders__index_replacement() {
        //fn replace_placeholders(reaction: &ReactionDefinition, reaction_call: ReactionCall) -> Vec<[ActionParam; 3]> {
        /*
         * index replacement
         */
        let action_params: Vec<[ActionParam; 3]> = replace_unit_behavior_placeholders(
            &reaction![
                "foo_reaction",
                reagent![
                    "foo_reagent",
                    param_value!(UnitResourceIndex, 0),
                    param_value!(UnitResourceIndex, 0),
                    unit_behavior_arg!(UnitResourceIndex)
                ],
            ],
            (0, 10, 0, 0),
        );

        assert_eq!(
            action_params,
            vec![[
                param_value!(UnitResourceIndex, 0),
                param_value!(UnitResourceIndex, 0),
                param_value!(UnitResourceIndex, 10),
            ]]
        );
    }

    #[test]
    fn replace_placeholders__amount_replacement() {
        let action_params = replace_unit_behavior_placeholders(
            &reaction![
                "",
                reagent![
                    "",
                    param_value!(UnitResourceIndex, 0),
                    unit_behavior_arg!(PositionResourceAmount),
                    unit_behavior_arg!(PositionResourceAmount)
                ],
                reagent![
                    "",
                    unit_behavior_arg!(PositionResourceAmount),
                    param_value!(UnitResourceIndex, 0),
                    param_value!(UnitResourceIndex, 0)
                ],
            ],
            (42 /*not used*/, 1, 2, 3),
        );

        assert_eq!(
            action_params,
            vec![
                [
                    param_value!(UnitResourceIndex, 0),
                    param_value!(PositionResourceAmount, 1),
                    param_value!(PositionResourceAmount, 2),
                ],
                [
                    param_value!(PositionResourceAmount, 3),
                    param_value!(UnitResourceIndex, 0),
                    param_value!(UnitResourceIndex, 0),
                ],
            ]
        );
    }
    #[ignore] // not implemented yet
    #[test]
    fn replace_placeholders__attr_val_replacement() {}
    #[ignore] // not implemented yet
    #[test]
    fn replace_placeholders__direction_replacement() {}
}

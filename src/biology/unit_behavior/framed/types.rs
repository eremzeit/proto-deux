use crate::biology::genetic_manifest::predicates::OperatorParam;
use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::types::NUM_META_REACTIONS;
use crate::biology::sensor_manifest::SensorId;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
pub use crate::chemistry::properties::RawPropertyId;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use crate::biology::genome::framed::convert::param_meta;
use crate::biology::genome::framed::types::{FramedGenomeValue, RegisterId};
use std::convert::TryInto;

pub type PhenotypeRegisterValue = u16;
pub type PhenotypeRegisters = Vec<PhenotypeRegisterValue>;
pub type MetaReactionCallParam = u32;
pub type MetaReactionId = u8;
pub type PhenotypeRegisterChanges = Vec<PhenotypeRegisterChange>;

// is retured by the genome, but needs to be converted into an actual reaction call
//pub type ParamedMetaReactionCall = (MetaReactionId, ParsedGenomeParam, ParsedGenomeParam, ParsedGenomeParam);

pub type ParamedReactionCall = (
    ReactionId,
    ParsedGenomeParam,
    ParsedGenomeParam,
    ParsedGenomeParam,
);

/**
 * Used to express a parameter that can be used for either in a conditional and reactions
 */
#[derive(Clone, Debug, PartialEq)]
pub enum ParsedGenomeParam {
    Constant(OperatorParam),
    SensorLookup(SensorId),
    Register(RegisterId),
    Random(usize),
}

impl ParsedGenomeParam {
    pub fn as_values(&self) -> (FramedGenomeValue, FramedGenomeValue) {
        let mut meta_val: FramedGenomeValue = 0;
        let mut param_val: FramedGenomeValue = 0;
        match &self {
            ParsedGenomeParam::Constant(val) => {
                meta_val = param_meta::val_for_constant() as FramedGenomeValue;
                param_val = (*val).try_into().unwrap();
            }
            ParsedGenomeParam::SensorLookup(sensor_id) => {
                meta_val = param_meta::val_for_sensor_lookup() as FramedGenomeValue;
                param_val = (*sensor_id).try_into().unwrap();
            }
            ParsedGenomeParam::Register(register_id) => {
                meta_val = param_meta::val_for_register_lookup() as FramedGenomeValue;
                param_val = (*register_id).try_into().unwrap();
            }
            ParsedGenomeParam::Random(max_val) => {
                meta_val = param_meta::val_for_random_lookup() as FramedGenomeValue;
                param_val = (*max_val).try_into().unwrap();
            }
        }

        (meta_val, param_val)
    }
    pub fn from(
        param_meta: FramedGenomeValue,
        param_val: FramedGenomeValue,
        gm: &GeneticManifest,
    ) -> Self {
        let sm = &gm.sensor_manifest;
        //println!("param_meta: {}, param_val: {}", param_meta, param_val);
        let num_sensors = sm.sensors.len();
        if param_meta::is_constant(param_meta as u16) {
            ParsedGenomeParam::Constant(param_val as OperatorParam)
        } else if param_meta::is_sensor_lookup(param_meta) {
            let sensor_id = (param_val % (num_sensors as FramedGenomeValue)) as usize;
            let sensor = &sm.sensors[sensor_id];
            ParsedGenomeParam::SensorLookup(sensor.id)
        } else if param_meta::is_register_lookup(param_meta) {
            ParsedGenomeParam::Register(
                (param_val % gm.number_of_registers as FramedGenomeValue) as RegisterId,
            )
        } else if param_meta::is_random_lookup(param_meta) {
            ParsedGenomeParam::Random(param_val as usize)
        } else {
            ParsedGenomeParam::Constant(0)
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum ParamedMetaReactionCall {
    JumpAheadFrames(ParsedGenomeParam),
    SetRegister(ParsedGenomeParam, ParsedGenomeParam),
    SetChannel(ParsedGenomeParam),
    Nil,
}

impl Debug for ParamedMetaReactionCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ParamedMetaReactionCall::SetChannel(n) => {
                write!(f, "SetChannel({:?})", n)
            }
            ParamedMetaReactionCall::JumpAheadFrames(n) => {
                write!(f, "JumpAhead({:?})", n)
            }
            // MetaReactionCall::JumpBehindFrames(n) => {
            //     write!(f, "JumpBehind({})", *n)
            // },
            ParamedMetaReactionCall::SetRegister(r, v) => {
                write!(f, "SetRegister({:?}, {:?}", r, v)
            }

            ParamedMetaReactionCall::Nil => {
                write!(f, "NilReaction")
            }
        }
    }
}

#[derive(Debug)]
pub enum MetaReaction {
    JumpAheadFrames,
    SetRegister,

    // question: when a channel changes during genome execution, does it change immediately or does it wait until the next frame?
    SetChannel,
    Nil,
}

impl MetaReaction {
    pub fn from_key(val: &String) -> Option<Self> {
        if val == "jump_ahead_frames" {
            return Some(Self::JumpAheadFrames);
        } else if val == "set_register" {
            return Some(Self::SetRegister);
        } else if val == "set_channel" {
            return Some(Self::SetChannel);
        } else if val == "nil" {
            return Some(Self::Nil);
        } else {
            return None;
        }
    }
    pub fn from_val(val: FramedGenomeValue) -> Self {
        match val % NUM_META_REACTIONS {
            0 => Self::Nil,
            1 => Self::JumpAheadFrames,
            2 => Self::SetRegister,
            3 => Self::SetChannel,
            _ => Self::Nil,
        }
    }
    pub fn to_val(&self) -> FramedGenomeValue {
        match &self {
            Self::Nil => 0,
            Self::JumpAheadFrames => 1,
            Self::SetRegister => 2,
            Self::SetChannel => 3,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct PhenotypeRegisterChange {
    pub offset: usize,
    pub new_value: PhenotypeRegisterValue,
}

/**
 * Represents a gene operation call that has parameters that still need to be evaluated.  ie. they aren't literal
 */
#[derive(PartialEq, Debug, Clone)]
pub enum ParamedGeneOperationCall {
    Reaction(
        ParamedReactionCall, // ReactionId,
                             // ParsedGenomeParam,
                             // ParsedGenomeParam,
                             // ParsedGenomeParam
    ),
    MetaReaction(ParamedMetaReactionCall),
    Nil,
}

impl ParamedGeneOperationCall {
    pub fn as_reaction_call(self) -> ParamedReactionCall {
        match &self {
            //GeneOperationCall::Reaction(id, param1, param2, param3) => { (*id, param1.clone(), param2.clone(), param3.clone())},
            ParamedGeneOperationCall::Reaction(call) => {
                (call.0, call.1.clone(), call.2.clone(), call.3.clone())
            }
            _ => {
                panic!("cant coerce into reaction call")
            }
        }
    }
}

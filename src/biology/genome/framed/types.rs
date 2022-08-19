use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorId, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::render::render_frames;
use crate::biology::genome::framed::*;
use crate::biology::unit_behavior::framed::*;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use std::fmt::{Debug, Formatter, Result};

//use crate::biology::genome::framed::util as util;

pub type GenomeBoolResult = bool;
pub type FramedGenomeWord = u64;
pub type FramedGenomeValue = u16;
pub type RawFramedGenome = Vec<FramedGenomeWord>;
pub type RegisterId = usize;

pub const NUM_CHANNELS: usize = 4;

// two values: [frame_size, default_channel]
pub const FRAME_META_DATA_SIZE: usize = 2;

pub const CHANNEL_ZERO: usize = 0;
pub const FIXED_NUM_CONDITIONAL_PARAMS: usize = 3;
pub const FIXED_NUM_OPERATION_PARAMS: usize = 3;

pub const MIN_FRAME_SIZE: usize = 4;
pub const NUM_META_REACTIONS: FramedGenomeValue = 4;

#[derive(Clone)]
pub struct FramedGenome {
    pub genotype: RawFramedGenome,
    pub phenotype: CompiledFramedGenome,
}

#[derive(Clone)]
pub struct CompiledFramedGenome {
    pub frames: Vec<Frame>,
}

impl CompiledFramedGenome {
    pub fn display(
        &self,
        sm: &SensorManifest,
        cm: &ChemistryManifest,
        gm: &GeneticManifest,
    ) -> String {
        render_frames(&self.frames, sm, cm, gm)
    }

    pub fn new(frames: Vec<Frame>) -> Self {
        CompiledFramedGenome { frames }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub channels: [Vec<Gene>; NUM_CHANNELS],
    pub default_channel: u8, // question: is this something given by the raw genome, or is this meant to be a run-time configuration?
}

#[derive(Debug, Clone)]
pub struct Gene {
    pub conditional: DisjunctiveClause,
    pub operation: GeneOperationCall,
}

// impl Gene {
//     pub fn display(&self) -> String {
//         let mut s
//     }
// }

pub type DisjunctiveClause = (IsNegatedBool, Vec<ConjunctiveClause>);
pub type ConjunctiveClause = (IsNegatedBool, Vec<BooleanVariable>);

pub type IsNegatedBool = bool;

#[derive(Clone, PartialEq)]
pub enum BooleanVariable {
    Literal(bool),
    Conditional(
        OperatorId,
        IsNegatedBool,
        ParsedGenomeParam,
        ParsedGenomeParam,
        ParsedGenomeParam,
    ),
}

use super::render::render_param;

impl BooleanVariable {
    pub fn render(
        &self,
        genetic_manifest: &GeneticManifest,
        sensor_manifest: &SensorManifest,
    ) -> String {
        match self {
            BooleanVariable::Literal(v) => format!("Value({})", v),
            BooleanVariable::Conditional(op_id, is_negated, _param1, _param2, _param3) => {
                let op = &genetic_manifest.operator_set.operators[*op_id as usize];
                let op_key = op.name.to_string();
                let param_count = op.num_params;

                let param1 = render_param(_param1, sensor_manifest);
                let param2 = render_param(_param2, sensor_manifest);
                let param3 = render_param(_param3, sensor_manifest);

                let params: [String; 3] = [param1, param2, param3];
                let negated_str = if *is_negated { "NOT " } else { "" }.to_string();
                format!("{}{}", &negated_str, (op.render)(&params))
            }
        }
    }

    fn get_conditional_params(
        op_id: OperatorId,
        p1: &ParsedGenomeParam,
        p2: &ParsedGenomeParam,
        p3: &ParsedGenomeParam,
        cm: &ChemistryManifest,
        gm: &GeneticManifest,
    ) -> Vec<ParsedGenomeParam> {
        let mut params: Vec<ParsedGenomeParam> = vec![];
        let op = &gm.operator_set.operators[op_id as usize];
        if op.num_params > 0 {
            params.push(p1.clone());

            if op.num_params > 1 {
                params.push(p2.clone());

                if op.num_params > 2 {
                    params.push(p3.clone());

                    if op.num_params > 3 {
                        panic!("Num params not supported");
                    }
                }
            }
        }

        params
    }

    fn are_all_constants(
        params: Vec<ParsedGenomeParam>,
        cm: &ChemistryManifest,
        gm: &GeneticManifest,
    ) -> bool {
        for i in 0..params.len() {
            match &params[i] {
                ParsedGenomeParam::Constant(val) => {}
                _ => {
                    return false;
                }
            }
        }

        true
    }

    fn extract_constant_params(
        params: Vec<ParsedGenomeParam>,
        cm: &ChemistryManifest,
        gm: &GeneticManifest,
    ) -> Vec<OperatorParam> {
        return params
            .iter()
            .map(|p: &ParsedGenomeParam| -> OperatorParam {
                match &p {
                    ParsedGenomeParam::Constant(val) => *val,
                    _ => 0,
                }
            })
            .collect::<Vec<_>>();
    }

    fn normalize(&self, cm: &ChemistryManifest, gm: &GeneticManifest) -> BooleanVariable {
        match self {
            BooleanVariable::Literal(v) => return self.clone(),
            BooleanVariable::Conditional(op_id, is_negated, _param1, _param2, _param3) => {
                let op = &gm.operator_set.operators[*op_id as usize];

                let mut params =
                    Self::get_conditional_params(op.index, _param1, _param2, _param3, cm, gm);

                let mut used_params = params.clone();
                used_params.truncate(op.num_params);

                if Self::are_all_constants(used_params, cm, gm) {
                    let constant_params = Self::extract_constant_params(params, cm, gm);
                    let result: bool = (op.evaluate)(constant_params.as_slice());
                    return BooleanVariable::Literal(result ^ *is_negated);
                }
            }
        }

        return self.clone();
    }
}

impl Debug for BooleanVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            BooleanVariable::Literal(v) => write!(f, "Value({})", v),
            BooleanVariable::Conditional(op_id, is_negated, param1, param2, param3) => {
                let s = if *is_negated {
                    "NEGATED"
                } else {
                    "NOT_NEGATED"
                };
                write!(
                    f,
                    "Conditional({}, {}, {:?}, {:?}, {:?})",
                    op_id, s, param1, param2, param3
                )
            }
        }
    }
}

pub mod tests {
    use chemistry::variants::CheeseChemistry;

    use super::*;
    use crate::biology::genetic_manifest::predicates::default_operators;
    use crate::simulation::common::*;

    #[test]
    pub fn boolean_variable_normalize__literal() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();

        let b = BooleanVariable::Literal(true).normalize(&cm, &gm);
        assert_eq!(b, BooleanVariable::Literal(true));
    }

    pub fn boolean_variable_normalize__all_constants() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();

        let bool_var = BooleanVariable::Conditional(
            gm.operator_set.by_key("eq").index,
            false,
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::Constant(0),
            // this next one should end up being ignored. as in, it doesn't need to be
            // calculated, leaving only constants in the operator execution
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&cm, &gm);

        assert_eq!(bool_var, BooleanVariable::Literal(true));
    }

    #[test]
    pub fn boolean_variable_normalize__all_constants_failure() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();

        let bool_var = BooleanVariable::Conditional(
            gm.operator_set.by_key("eq").index,
            false,
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&cm, &gm);

        assert_eq!(
            bool_var,
            BooleanVariable::Conditional(
                0,
                false,
                ParsedGenomeParam::Constant(0),
                ParsedGenomeParam::SensorLookup(0),
                ParsedGenomeParam::SensorLookup(0)
            )
        );
    }

    pub fn boolean_variable_normalize__constant_operator() {
        let gm = GeneticManifest::new();
        let cm = CheeseChemistry::default_manifest();

        let true_var = BooleanVariable::Conditional(
            gm.operator_set.by_key("true").index,
            false,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&cm, &gm);
        assert_eq!(true_var, BooleanVariable::Literal(true));

        let false_var = BooleanVariable::Conditional(
            gm.operator_set.by_key("false").index,
            false,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&cm, &gm);

        assert_eq!(false_var, BooleanVariable::Literal(false));
        let true_negated_var = BooleanVariable::Conditional(
            gm.operator_set.by_key("false").index,
            true,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&cm, &gm);

        assert_eq!(true_negated_var, BooleanVariable::Literal(false));
        let false_negated_var = BooleanVariable::Conditional(
            gm.operator_set.by_key("false").index,
            true,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&cm, &gm);

        assert_eq!(false_negated_var, BooleanVariable::Literal(true));
    }
}

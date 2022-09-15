use crate::biology::genetic_manifest::predicates::{
    OperatorId, OperatorImplementation, OperatorManifest, OperatorParam, OperatorParamDefinition,
    OperatorParamType,
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
use std::cell::Cell;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

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

// impl FramedGenome {
//     pub fn from_raw(
//         raw_frames: RawFramedGenome,
//         genetic_manifest: Rc<GeneticManifest>,
//         // cm: &ChemistryManifest,
//         // gm: &GeneticManifest,
//         // sm: &SensorManifest,
//     ) -> Self {
//     }
// }

// #[derive(Clone)]
// pub struct CompiledFramedGenomeWithStats {
//     pub frames: Vec<Frame>,
// }

#[derive(Clone)]
pub struct CompiledFramedGenome {
    pub frames: Vec<Frame>,
    pub raw_size: usize,
    pub raw_values: Vec<FramedGenomeWord>,
}

impl CompiledFramedGenome {
    pub fn display(&self, gm: &GeneticManifest) -> String {
        render_frames(&self.frames, gm)
    }

    pub fn new(frames: Vec<Frame>, raw_values: Vec<FramedGenomeWord>) -> Self {
        let raw_size = raw_values.len();
        CompiledFramedGenome {
            frames,
            raw_values,
            raw_size,
        }
    }

    pub fn wrap_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn new_stats(&self) -> FramedGenomeExecutionStats {
        FramedGenomeExecutionStats::new(&self.frames)
    }

    pub fn render_with_stats(
        &self,
        gm: &GeneticManifest,
        stats: &FramedGenomeExecutionStats,
    ) -> String {
        render_frames_with_stats(&self.frames, gm, Some(stats))
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub channels: [Vec<Gene>; NUM_CHANNELS],
    pub default_channel: u8,

    pub address_range: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct Gene {
    pub conditional: Disjunction,
    pub operation: ParamedGeneOperationCall,
    pub address_range: (usize, usize),
}

impl Gene {
    pub fn new(conditional: Disjunction, operation: ParamedGeneOperationCall) -> Self {
        Self {
            conditional,
            operation,
            address_range: (0, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Disjunction {
    pub is_negated: IsNegatedBool,
    pub conjunctive_clauses: Vec<Conjunction>,
    pub address_range: (usize, usize),
}

impl Disjunction {
    /**
     * Meant to be used only in situations when you don't need the address_range (ie. testing, debugging)
     */
    pub fn new(is_negated: bool, clauses: Vec<Conjunction>) -> Self {
        Self {
            is_negated,
            conjunctive_clauses: clauses,
            address_range: (0, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Conjunction {
    pub is_negated: IsNegatedBool,
    pub boolean_variables: Vec<BooleanVariable>,
    pub address_range: (usize, usize),
}

impl Conjunction {
    /**
     * Meant to be used only in situations when you don't need the address_range (ie. testing, debugging)
     */
    pub fn new(is_negated: bool, boolean_variables: Vec<BooleanVariable>) -> Self {
        Self {
            is_negated,
            boolean_variables,
            address_range: (0, 0),
        }
    }
}

pub type IsNegatedBool = bool;

// TODO: maybe rename this to BooleanPredicate?
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

use super::annotated::FramedGenomeExecutionStats;
use super::render::render_param;
use super::render::with_stats::render_frames_with_stats;

impl BooleanVariable {
    pub fn render(&self, genetic_manifest: &GeneticManifest) -> String {
        match self {
            BooleanVariable::Literal(v) => format!("Value({})", v),
            BooleanVariable::Conditional(op_id, is_negated, _param1, _param2, _param3) => {
                let op = &genetic_manifest.operator_manifest.operators[*op_id as usize];
                let op_key = op.name.to_string();
                let param_count = op.num_params;

                let sensor_manifest = &genetic_manifest.sensor_manifest;
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
        gm: &GeneticManifest,
    ) -> Vec<ParsedGenomeParam> {
        let mut params: Vec<ParsedGenomeParam> = vec![];
        let op = &gm.operator_manifest.operators[op_id as usize];
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

    fn are_all_constants(params: Vec<ParsedGenomeParam>, gm: &GeneticManifest) -> bool {
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

    fn extract_constant_params(params: Vec<ParsedGenomeParam>) -> Vec<OperatorParam> {
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

    fn normalize(&self, gm: &GeneticManifest) -> BooleanVariable {
        match self {
            BooleanVariable::Literal(v) => return self.clone(),
            BooleanVariable::Conditional(op_id, is_negated, _param1, _param2, _param3) => {
                let op = &gm.operator_manifest.operators[*op_id as usize];

                let mut params =
                    Self::get_conditional_params(op.index, _param1, _param2, _param3, gm);

                let mut used_params = params.clone();
                used_params.truncate(op.num_params);

                if Self::are_all_constants(used_params, gm) {
                    let constant_params = Self::extract_constant_params(params);
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
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let cm = CheeseChemistry::default_manifest();

        let b = BooleanVariable::Literal(true).normalize(&gm);
        assert_eq!(b, BooleanVariable::Literal(true));
    }

    pub fn boolean_variable_normalize__all_constants() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let cm = CheeseChemistry::default_manifest();

        let bool_var = BooleanVariable::Conditional(
            gm.operator_manifest.by_key("eq").index,
            false,
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::Constant(0),
            // this next one should end up being ignored. as in, it doesn't need to be
            // calculated, leaving only constants in the operator execution
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&gm);

        assert_eq!(bool_var, BooleanVariable::Literal(true));
    }

    #[test]
    pub fn boolean_variable_normalize__all_constants_failure() {
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let cm = CheeseChemistry::default_manifest();

        let bool_var = BooleanVariable::Conditional(
            gm.operator_manifest.by_key("eq").index,
            false,
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&gm);

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
        let gm = GeneticManifest::from_default_chemistry_config::<CheeseChemistry>();
        let cm = gm.chemistry_manifest.as_ref().clone();

        let true_var = BooleanVariable::Conditional(
            gm.operator_manifest.by_key("true").index,
            false,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&gm);
        assert_eq!(true_var, BooleanVariable::Literal(true));

        let false_var = BooleanVariable::Conditional(
            gm.operator_manifest.by_key("false").index,
            false,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&gm);

        assert_eq!(false_var, BooleanVariable::Literal(false));
        let true_negated_var = BooleanVariable::Conditional(
            gm.operator_manifest.by_key("false").index,
            true,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&gm);

        assert_eq!(true_negated_var, BooleanVariable::Literal(false));
        let false_negated_var = BooleanVariable::Conditional(
            gm.operator_manifest.by_key("false").index,
            true,
            // these lookups aren't used
            ParsedGenomeParam::SensorLookup(0),
            ParsedGenomeParam::Constant(0),
            ParsedGenomeParam::SensorLookup(0),
        )
        .normalize(&gm);

        assert_eq!(false_negated_var, BooleanVariable::Literal(true));
    }
}

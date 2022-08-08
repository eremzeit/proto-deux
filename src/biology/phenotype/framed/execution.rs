use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::genome::framed::common::*;
use crate::biology::genome::framed::types::NUM_META_REACTIONS;
use crate::biology::phenotype::Phenotype;
use crate::chemistry;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use std::rc::Rc;
//use crate::simulation::world::World;
pub use crate::chemistry::properties::RawPropertyId;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};
//use crate::biology::phenotype::{ParamedReactionCall};

use crate::biology::genome::framed::*;
use crate::biology::phenotype::framed::types::*;
use crate::chemistry::reactions::ReactionCall;
use std::convert::TryInto;

pub struct GenomeExecutionContext<'a> {
    pub chemistry_manifest: &'a ChemistryManifest,
    pub sensor_manifest: &'a SensorManifest,
    pub genetic_manifest: &'a GeneticManifest,
    frames: &'a Vec<Frame>,
    current_frame: usize,
    override_channel: Option<u8>,
    consumed_compute_points: i32,
    allotted_compute_points: i32,

    pub sensor_context: &'a SensorContext<'a>,
    registers: PhenotypeRegisters,
    pub register_changes: PhenotypeRegisterChanges,
}

use crate::biology::genome::framed::render::render_gene;

impl<'a> GenomeExecutionContext<'a> {
    pub fn new(
        frames: &'a Vec<Frame>,
        sensor_context: &'a SensorContext,
        registers: PhenotypeRegisters,
        cm: &'a ChemistryManifest,
        sm: &'a SensorManifest,
        gm: &'a GeneticManifest,
        compute_points: i32,
    ) -> Self {
        Self {
            chemistry_manifest: cm,
            sensor_manifest: sm,
            genetic_manifest: gm,
            frames,
            sensor_context,
            current_frame: 0,
            override_channel: None,
            registers,
            register_changes: vec![],
            consumed_compute_points: 0,
            allotted_compute_points: compute_points,
        }
    }

    pub fn execute(&mut self) -> Vec<ReactionCall> {
        let mut reactions = vec![];
        while self.current_frame < self.frames.len()
            && self.consumed_compute_points < self.allotted_compute_points
        {
            let result = self.execute_frame();
            if result.is_some() {
                reactions.push(result.unwrap())
            }
            self.current_frame += 1;
        }

        reactions
    }

    pub fn execute_frame(&mut self) -> Option<ReactionCall> {
        let frame = &self.frames[self.current_frame];
        let channel = self.override_channel.unwrap_or(frame.default_channel) % NUM_CHANNELS as u8;

        let genes = &frame.channels[channel as usize];

        for (i, gene) in genes.iter().enumerate() {
            let cond = &gene.conditional;
            let cond_result = self.execute_conditional(&cond);
            if self.consumed_compute_points > self.allotted_compute_points {
                return None;
            }

            if cond_result {
                let op = &gene.operation;
                let result = self.execute_operation(op);

                if result.is_some() {
                    let s = render_gene(
                        gene,
                        &self.chemistry_manifest,
                        &self.genetic_manifest,
                        &self.sensor_manifest,
                    );
                    //flog!("Gene {} triggered: {:?}", i, &s);
                    return result;
                }
            }
        }

        None
    }
    pub fn execute_operation(&mut self, operation: &GeneOperationCall) -> Option<ReactionCall> {
        match operation {
            GeneOperationCall::MetaReaction(meta_reaction) => {
                self.consumed_compute_points += 1;
                None
                //TODO
                //panic!("meta reactions not supported yet");
            }
            GeneOperationCall::Reaction(paramed_reaction_call) => {
                self.consumed_compute_points += 1;

                let param_val1 = self.eval_param(&paramed_reaction_call.1);
                let param_val2 = self.eval_param(&paramed_reaction_call.2);
                let param_val3 = self.eval_param(&paramed_reaction_call.3);

                //flog!("REACTION TO EXECUTE: {:?}", paramed_reaction_call);
                return Some((
                    paramed_reaction_call.0,
                    (param_val1 % (u16::MAX as i32)).try_into().unwrap(),
                    (param_val2 % (u16::MAX as i32)).try_into().unwrap(),
                    (param_val3 % (u16::MAX as i32)).try_into().unwrap(),
                ));
            }
            GeneOperationCall::Nil => None,
        }
    }

    pub fn execute_conditional(&mut self, conditional: &DisjunctiveClause) -> bool {
        let mut or_result = false;
        let is_negated = conditional.0;

        for disjunctive in &conditional.1 {
            let is_negated = disjunctive.0;
            let mut and_result = true;
            for bool_expr in &disjunctive.1 {
                if !self.execute_boolean(bool_expr) {
                    and_result = false
                }
            }

            if and_result ^ is_negated {
                or_result = true;
            }
        }

        or_result ^ is_negated
    }
    pub fn eval_param(&mut self, parsed_param: &ParsedGenomeParam) -> i32 {
        use rand::Rng;

        match parsed_param {
            ParsedGenomeParam::Constant(c) => *c as i32,
            ParsedGenomeParam::SensorLookup(sensor_id) => {
                let sensor = &self.sensor_manifest.sensors[*sensor_id as usize];
                let val = sensor.calculate(self.sensor_context);
                val
            }

            ParsedGenomeParam::Register(register_id) => {
                // TODO!
                //self.registers[*register_id as usize] as i32
                0
            }
            ParsedGenomeParam::Random(max_val) => {
                let mut rng = rand::thread_rng();
                rng.gen_range(0..*max_val as usize) as i32
            }
        }
    }

    pub fn execute_boolean(&mut self, boolean_clause: &BooleanVariable) -> bool {
        match boolean_clause {
            BooleanVariable::Literal(v) => {
                //self.consumed_compute_points += 1;
                *v
            }
            BooleanVariable::Conditional(op_id, is_negated, param1, param2, param3) => {
                self.consumed_compute_points += 1;
                let op = &self.genetic_manifest.operator_set.operators[*op_id as usize];

                let param_val1 = self.eval_param(param1);
                let param_val2 = self.eval_param(param2);
                let param_val3 = self.eval_param(param3);

                let op_str = (op.render)(&[
                    param_val1.to_string(),
                    param_val2.to_string(),
                    param_val3.to_string(),
                ]);

                let result = (op.evaluate)(&[param_val1, param_val2, param_val3]);
                //println!("BOOL: {} = {}", &op_str, &result);
                if *is_negated {
                    !result
                } else {
                    result
                }
            }
        }
    }
}

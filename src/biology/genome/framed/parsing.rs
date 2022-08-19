use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorId, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;

use crate::biology::genome::framed::common::{
    BooleanVariable, CompiledFramedGenome, ConjunctiveClause, DisjunctiveClause, Frame,
    FramedGenomeValue, FramedGenomeWord, Gene, CHANNEL_ZERO, FIXED_NUM_CONDITIONAL_PARAMS,
    FRAME_META_DATA_SIZE, NUM_CHANNELS, NUM_META_REACTIONS,
};
use crate::biology::genome::framed::convert::{RawFrame, RawFrameParser};
use crate::biology::unit_behavior::framed::common::*;
use crate::perf::{perf_timer_start, perf_timer_stop};

use super::convert;
use super::convert::param_meta;

use crate::biology::genome::framed::*;
use crate::biology::unit_behavior::framed::*;

use crate::biology::genome::framed::render::{
    render_conjunction, render_disjunction, render_gene_operation,
};

pub use crate::chemistry::properties::RawPropertyId;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};

pub use crate::biology::sensor_manifest::{SensorContext, SensorManifest, SensorType, SensorValue};

pub struct FramedGenomeParser {
    genetic_manifest: GeneticManifest,
    sensor_manifest: SensorManifest,
    chemistry_manifest: ChemistryManifest,
    raw_frames: Vec<RawFrame>,

    // state variables for parsing
    current_frame: usize,
    idx: usize,
    current_channel: usize,
}

const NUM_VALS_FOR_OPERATION: usize = 8;

impl FramedGenomeParser {
    pub fn parse(
        raw_values: Vec<FramedGenomeWord>,
        chemistry_manifest: ChemistryManifest,
        sensor_manifest: SensorManifest,
        genetic_manifest: GeneticManifest,
    ) -> CompiledFramedGenome {
        flog!("Parsing genome of size {}", raw_values.len());
        flog!("raw genome values: {:?}", &raw_values);
        let cm = chemistry_manifest.clone();

        perf_timer_start!("genome_parsing");
        let mut s = Self::new(raw_values, sensor_manifest, cm, genetic_manifest);
        let frames = s.compile_frames();
        perf_timer_stop!("genome_parsing");
        flog!("FINISHED COMPILING FRAMES");
        CompiledFramedGenome { frames }
    }

    pub fn new(
        values: Vec<FramedGenomeWord>,
        sensor_manifest: SensorManifest,
        chemistry_manifest: ChemistryManifest,
        genetic_manifest: GeneticManifest,
    ) -> Self {
        let raw_frames = RawFrameParser::parse(values);
        flog!("raw frames: {:?}", &raw_frames);

        Self {
            sensor_manifest,
            chemistry_manifest,
            genetic_manifest,
            current_channel: 0,
            current_frame: 0,
            idx: 0,
            raw_frames,
        }
    }
    pub fn pop_n_in_frame(&mut self, n: usize) -> Option<Vec<FramedGenomeValue>> {
        let mut result = vec![];
        for i in 0..n {
            if let Some(v) = self.pop_in_frame() {
                result.push(v);
            }
        }

        if result.len() == n {
            return Some(result);
        } else {
            return None;
        }
    }
    pub fn pop_in_frame(&mut self) -> Option<FramedGenomeValue> {
        let frame = &self.raw_frames[self.current_frame];
        let channel_vals = &frame.channel_values[self.current_channel];

        if self.idx >= channel_vals.len() {
            return None;
        }

        let val = self.get_value_at(self.idx);
        self.idx += 1;
        Some(val)
    }

    pub fn compile_frames(&mut self) -> Vec<Frame> {
        let mut frames = vec![];
        self.current_frame = 0;

        while self.current_frame < self.raw_frames.len() {
            let frame = self.compile_frame();

            if frame.is_some() {
                frames.push(frame.unwrap());
            }

            self.current_frame += 1;
        }

        frames
    }

    pub fn compile_frame(&mut self) -> Option<Frame> {
        let default_channel =
            self.raw_frames[self.current_frame].default_channel % NUM_CHANNELS as u8;

        flog!("COMPILING FRAME (current_frame: {:?})", &self.current_frame);
        let mut channels = [vec![], vec![], vec![], vec![]];
        for channel in 0..NUM_CHANNELS {
            self.idx = 0;
            self.current_channel = channel;

            let _frame_channel = self.compile_frame_channel().clone();
            if let Some(frame_channel) = _frame_channel {
                channels[channel] = frame_channel;
            }
        }
        let frame = Frame {
            channels: channels,
            default_channel: default_channel,
        };

        Some(frame)
    }

    pub fn current_channel_vals_len(&self) -> usize {
        let frame = &self.raw_frames[self.current_frame];
        let channel_vals = &frame.channel_values[self.current_channel];
        return channel_vals.len();
    }
    pub fn compile_frame_channel(&mut self) -> Option<Vec<Gene>> {
        let mut genes: Vec<Gene> = vec![];

        while self.idx < self.current_channel_vals_len() {
            if let Some(gene) = self.compile_gene() {
                genes.push(gene);
            } else {
                break;
            }
        }

        if genes.len() > 0 {
            Some(genes)
        } else {
            None
        }
    }

    pub fn compile_gene(&mut self) -> Option<Gene> {
        flog!(
            "COMPILING GENE... (index: {:?}, {})",
            &self.idx,
            &self.current_frame
        );
        let predicate = self.compile_disjunctive_predicate();
        let operation = self.compile_operation();

        return if predicate.is_some() && operation.is_some() {
            let pred = predicate.unwrap();
            let _operation = operation.unwrap();

            flog!(
                "compiled disjunction: {:?}",
                &render_disjunction(
                    &pred,
                    &self.chemistry_manifest,
                    &self.genetic_manifest,
                    &self.sensor_manifest
                )
            );
            flog!(
                "compiled operation: {:?}",
                &render_gene_operation(
                    &_operation,
                    &self.chemistry_manifest,
                    &self.sensor_manifest,
                    &self.genetic_manifest
                )
            );

            Some(Gene {
                conditional: pred,
                operation: _operation,
            })
        } else {
            None
        };
    }

    pub fn compile_disjunctive_predicate(&mut self) -> Option<DisjunctiveClause> {
        let n_clauses = match self.pop_in_frame() {
            Some(v) => convert::val_to_n_or_clauses(v),
            None => {
                return None;
            }
        };
        let is_negated = match self.pop_in_frame() {
            Some(v) => v % 2 > 0,
            None => {
                return None;
            }
        };
        flog!("num OR clauses: {}", n_clauses);

        let mut clauses: Vec<ConjunctiveClause> = vec![];

        for i in (0..n_clauses) {
            let pred = self.compile_conjunctive_predicate();
            if pred.is_some() {
                clauses.push(pred.unwrap());
            }
        }

        if clauses.len() > 0 {
            Some((is_negated, clauses))
        } else {
            None
        }
    }

    pub fn compile_conjunctive_predicate(&mut self) -> Option<ConjunctiveClause> {
        let n_conditionals = match self.pop_in_frame() {
            Some(v) => convert::val_to_n_and_clauses(v),
            None => {
                return None;
            }
        };
        let is_negated = match self.pop_in_frame() {
            Some(v) => v % 2 > 0,
            None => {
                return None;
            }
        };
        flog!("num AND clauses: {}", n_conditionals);

        let mut vars: Vec<BooleanVariable> = vec![];
        for i in (0..n_conditionals) {
            let bool_var = self.compile_boolean_variable();
            if bool_var.is_some() {
                vars.push(bool_var.unwrap());
            }
        }
        if vars.len() > 0 {
            Some((is_negated, vars))
        } else {
            None
        }
    }

    pub fn compile_boolean_variable(&mut self) -> Option<BooleanVariable> {
        let num_sensors = self.sensor_manifest.sensors.len();
        let manifest = &self.chemistry_manifest;
        let operator_id = match &self.pop_in_frame() {
            Some(v) => (*v as usize) % self.genetic_manifest.operator_set.operators.len(),
            None => {
                return None;
            }
        };
        let is_negated = match &self.pop_in_frame() {
            Some(v) => (*v as usize) % 2 > 0,
            None => {
                return None;
            }
        };

        let op: &Operator = &self.genetic_manifest.operator_set.operators[operator_id].clone();

        let mut _popped_params: Vec<Option<FramedGenomeValue>> = vec![];

        // pop the max number of params and verify that we have enough to at least execute the call
        let num_required_vals = FIXED_NUM_CONDITIONAL_PARAMS * 2;
        for i in 0..num_required_vals {
            _popped_params.push(self.pop_in_frame());
        }

        let popped_params = _popped_params.iter().flatten().collect::<Vec<_>>();
        flog!("Conditional popped_params: {:?}", popped_params);

        // if we don't have enough to make the call then just give up
        if popped_params.len() < num_required_vals {
            return None;
        }

        let mut processed_params: Vec<ParsedGenomeParam> =
            vec![ParsedGenomeParam::Constant(98765); 3];
        for i in 0..op.num_params {
            let param_meta = popped_params[i * 2];
            let param_val = popped_params[i * 2 + 1];

            processed_params[i] = ParsedGenomeParam::from(
                *param_meta,
                *param_val,
                &self.sensor_manifest,
                &self.genetic_manifest,
            );
            flog!("processed param {}: {:?}", i, &processed_params[i]);
        }

        let conditional = BooleanVariable::Conditional(
            operator_id as OperatorId,
            is_negated,
            processed_params[0].clone(),
            processed_params[1].clone(),
            processed_params[2].clone(),
        );

        Some(conditional)
    }
    pub fn pop_operation(
        &mut self,
    ) -> Option<(
        FramedGenomeValue,
        FramedGenomeValue,
        ParsedGenomeParam,
        ParsedGenomeParam,
        ParsedGenomeParam,
    )> {
        let op_type = match self.pop_in_frame() {
            Some(v) => v,
            None => {
                return None;
            }
        };
        let raw_operation_id = match self.pop_in_frame() {
            Some(v) => v,
            None => {
                return None;
            }
        };
        let param1_vals = self.pop_n_in_frame(2);
        let mut param1 = ParsedGenomeParam::Constant(10);
        if let Some(vals) = param1_vals {
            param1 = ParsedGenomeParam::from(
                vals[0],
                vals[1],
                &self.sensor_manifest,
                &self.genetic_manifest,
            );
            //println!("PARSED PARAM FROM param1: {:?}", &param1);
        }
        let param2_vals = self.pop_n_in_frame(2);
        let mut param2 = ParsedGenomeParam::Constant(10);
        if let Some(vals) = param2_vals {
            param2 = ParsedGenomeParam::from(
                vals[0],
                vals[1],
                &self.sensor_manifest,
                &self.genetic_manifest,
            );
        }
        let param3_vals = self.pop_n_in_frame(2);
        let mut param3 = ParsedGenomeParam::Constant(10);
        if let Some(vals) = param3_vals {
            param3 = ParsedGenomeParam::from(
                vals[0],
                vals[1],
                &self.sensor_manifest,
                &self.genetic_manifest,
            );
        }

        return Some((op_type, raw_operation_id, param1, param2, param3));
    }

    pub fn compile_operation(&mut self) -> Option<GeneOperationCall> {
        //println!("COMPILING OPERATION");
        let maybe_op = self.pop_operation();

        if maybe_op.is_none() {
            return None;
        }

        let (op_type, op_id, param1, param2, param3) = maybe_op.unwrap();

        if convert::operation::is_meta_reaction(op_type) {
            let meta_reaction: Option<ParamedMetaReactionCall> =
                match &MetaReaction::from_val(op_id) {
                    MetaReaction::JumpAheadFrames => {
                        Some(ParamedMetaReactionCall::JumpAheadFrames(param1))
                    }
                    MetaReaction::SetChannel => Some(ParamedMetaReactionCall::SetChannel(param1)),
                    MetaReaction::SetRegister => {
                        Some(ParamedMetaReactionCall::SetRegister(param1, param2))
                    }
                    MetaReaction::Nil => None,
                };

            if let Some(_meta_reaction) = meta_reaction {
                return Some(GeneOperationCall::MetaReaction(_meta_reaction));
            } else {
                return None;
            }
        } else {
            let num_reactions = self.chemistry_manifest.reactions.len();
            let reaction_id = (op_id as usize % num_reactions) as ReactionId;
            return Some(GeneOperationCall::Reaction((
                reaction_id,
                param1,
                param2,
                param3,
            )));
        }

        None
    }

    pub fn get_value_at(&self, idx: usize) -> FramedGenomeValue {
        let frame = &self.raw_frames[self.current_frame];
        let channel_vals = &frame.channel_values[self.current_channel];
        channel_vals[idx]
    }
}

pub mod tests {
    use super::super::render::render_genes;
    use super::*;
    //use crate::biology::genome::framed::tests::{full_test_framed_genome_one};

    // #[test]
    // pub fn parsing() {
    //     let values  = full_test_framed_genome_one();
    //     let chemistry = Box::new(CheeseChemistry::construct());
    //     let gm = GeneticManifest::new();

    //     let sm = SensorManifest::with_default_sensors(chemistry.get_manifest());

    //     let frames = FramedGenomeParser::parse(values, chemistry.get_manifest().clone(), sm.clone(), gm.clone());

    //     let mut s = String::new();
    //     for frame in &frames.frames {
    //         s = format!("{}\n{}", s, render_genes(&frame.channels[0], &sm, &chemistry.get_manifest(), &gm));
    //         //s = format!("{}---\n{:?}", s, frame.channels[0].len());
    //     }

    //     println!("RESULT: {}", s);
    // }
}

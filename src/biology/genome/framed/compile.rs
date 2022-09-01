use crate::biology::genetic_manifest::predicates::{
    OperatorId, OperatorImplementation, OperatorManifest, OperatorParam, OperatorParamDefinition,
    OperatorParamType,
};
use crate::biology::genome::framed::common::{
    BooleanVariable, CompiledFramedGenome, Conjunction, Disjunction, Frame, FramedGenomeValue,
    FramedGenomeWord, Gene, CHANNEL_ZERO, FIXED_NUM_CONDITIONAL_PARAMS, FRAME_META_DATA_SIZE,
    NUM_CHANNELS, NUM_META_REACTIONS,
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
use std::rc::Rc;

pub use crate::biology::sensor_manifest::{SensorContext, SensorManifest, SensorType, SensorValue};

pub struct FramedGenomeCompiler<'a> {
    genetic_manifest: &'a GeneticManifest,
    // sensor_manifest: &'a SensorManifest,
    // chemistry_manifest: &'a ChemistryManifest,
    raw_frames: Vec<RawFrame>,

    // state variables for parsing
    current_frame: usize,

    // is the index within the current frame
    idx: usize,

    current_channel: usize,
}

const NUM_VALS_FOR_OPERATION: usize = 8;

impl<'a> FramedGenomeCompiler<'a> {
    pub fn compile(
        raw_genome: Vec<FramedGenomeWord>,
        genetic_manifest: &'a GeneticManifest,
    ) -> Rc<CompiledFramedGenome> {
        flog!("\n\nCompiling genome of size {}", raw_genome.len());
        // flog!("raw genome values: {:?}", &raw_genome);

        let raw_size = raw_genome.len();
        perf_timer_start!("genome_compiling");
        let mut s = Self::new(raw_genome.clone(), genetic_manifest);
        let frames = s.compile_frames();
        perf_timer_stop!("genome_compiling");
        flog!("FINISHED COMPILING FRAMES");

        Rc::new(CompiledFramedGenome {
            frames,
            raw_size,
            raw_values: raw_genome,
        })
    }

    pub fn get_global_val_index(&self, frame_idx: usize, index_in_frame: usize) -> usize {
        self.raw_frames[frame_idx].address_range.0 + FRAME_META_DATA_SIZE + index_in_frame
    }

    pub fn new(values: Vec<FramedGenomeWord>, genetic_manifest: &'a GeneticManifest) -> Self {
        let raw_frames = RawFrameParser::parse(values);
        println!("raw frames: {:?}", &raw_frames);

        Self {
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
            flog!(
                "Cant fulfill pop_n_in_frame (channel: {}) (idx: {})",
                &self.current_channel,
                &self.idx
            );
            return None;
        }
    }

    pub fn pop_in_frame(&mut self) -> Option<FramedGenomeValue> {
        let frame = &self.raw_frames[self.current_frame];
        let channel_vals = &frame.channel_values[self.current_channel];

        if self.idx >= channel_vals.len() {
            // println!(
            //     "breaking because {} is >= channel_vals.len() {}",
            //     self.idx,
            //     channel_vals.len()
            // );
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
            flog!("\n\nFRAME {}", self.current_frame);
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

        let channel_lengths = self.raw_frames[self.current_frame]
            .channel_values
            .iter()
            .map(|v| v.len())
            .collect::<Vec<_>>();
        flog!(
            "COMPILING FRAME (current_frame: {:?}) (channel_lengths: {:?}",
            &self.current_frame,
            channel_lengths
        );

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
            address_range: self.raw_frames[self.current_frame].address_range.clone(),
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
                flog!("[{}] no more genes", self.idx);
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
        let gene_start_address = self.get_global_val_index(self.current_frame, self.idx);

        println!(
            "COMPILING GENE... (index: {:?}, frame: {}, channel: {})",
            &self.idx, &self.current_frame, &self.current_channel,
        );
        let predicate = self.compile_disjunctive_predicate();
        let operation = self.compile_operation();

        flog!("compiled disjunction: {:?}", &predicate);
        flog!("compiled operation: {:?}", &operation);

        return if predicate.is_some() && operation.is_some() {
            let pred = predicate.unwrap();
            let _operation = operation.unwrap();

            // flog!(
            //     "compiled disjunction: {:?}",
            //     &render_disjunction(&pred, &self.genetic_manifest,)
            // );
            // flog!(
            //     "compiled operation: {:?}",
            //     &render_gene_operation(&_operation, &self.genetic_manifest)
            // );

            let gene_end_address = self.get_global_val_index(self.current_frame, self.idx);
            Some(Gene {
                conditional: pred,
                operation: _operation,
                address_range: (gene_start_address, gene_end_address),
            })
        } else {
            None
        };
    }

    pub fn compile_disjunctive_predicate(&mut self) -> Option<Disjunction> {
        let start_address = self.get_global_val_index(self.current_frame, self.idx);
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

        // flog!("num OR clauses: {}", n_clauses);

        let mut clauses: Vec<Conjunction> = vec![];

        for i in (0..n_clauses) {
            let pred = self.compile_conjunctive_predicate();
            if pred.is_some() {
                clauses.push(pred.unwrap());
            } else {
                flog!("empty AND predicate");
            }
        }

        if clauses.len() == n_clauses {
            let end_address = self.get_global_val_index(self.current_frame, self.idx);
            Some(Disjunction {
                conjunctive_clauses: clauses,
                is_negated,
                address_range: (start_address, end_address),
            })
        } else {
            flog!("clauses {}", clauses.len());
            None
        }
    }

    pub fn compile_conjunctive_predicate(&mut self) -> Option<Conjunction> {
        let start_address = self.get_global_val_index(self.current_frame, self.idx);

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
        // flog!("num AND clauses: {}", n_conditionals);

        // let gene_start_address = self.get_global_val_index(self.current_frame, start_idx);

        let mut vars: Vec<BooleanVariable> = vec![];
        for i in (0..n_conditionals) {
            let bool_var = self.compile_boolean_variable();
            if bool_var.is_some() {
                vars.push(bool_var.unwrap());
            }
        }

        if vars.len() == n_conditionals {
            let end_address = self.get_global_val_index(self.current_frame, self.idx);
            Some(Conjunction {
                is_negated,
                boolean_variables: vars,
                address_range: (start_address, end_address),
            })
        } else {
            None
        }
    }

    pub fn compile_boolean_variable(&mut self) -> Option<BooleanVariable> {
        let sm = &self.genetic_manifest.sensor_manifest;
        let num_sensors = sm.sensors.len();
        let manifest = &self.genetic_manifest.chemistry_manifest;
        let operator_id = match &self.pop_in_frame() {
            Some(v) => (*v as usize) % self.genetic_manifest.operator_manifest.operators.len(),
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

        let op: &OperatorImplementation =
            &self.genetic_manifest.operator_manifest.operators[operator_id].clone();

        let mut _popped_params: Vec<Option<FramedGenomeValue>> = vec![];

        // pop the max number of params and verify that we have enough to at least execute the call
        let num_required_vals = FIXED_NUM_CONDITIONAL_PARAMS * 2;
        for i in 0..num_required_vals {
            _popped_params.push(self.pop_in_frame());
        }

        let popped_params = _popped_params.iter().flatten().collect::<Vec<_>>();
        // flog!("Conditional popped_params: {:?}", popped_params);

        // if we don't have enough to make the call then just give up
        if popped_params.len() < num_required_vals {
            return None;
        }

        let mut processed_params: Vec<ParsedGenomeParam> =
            vec![ParsedGenomeParam::Constant(98765); 3];
        for i in 0..op.num_params {
            let param_meta = popped_params[i * 2];
            let param_val = popped_params[i * 2 + 1];

            processed_params[i] =
                ParsedGenomeParam::from(*param_meta, *param_val, &self.genetic_manifest);
            // flog!("processed param {}: {:?}", i, &processed_params[i]);
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
            param1 = ParsedGenomeParam::from(vals[0], vals[1], &self.genetic_manifest);
            //println!("PARSED PARAM FROM param1: {:?}", &param1);
        }
        let param2_vals = self.pop_n_in_frame(2);
        let mut param2 = ParsedGenomeParam::Constant(10);
        if let Some(vals) = param2_vals {
            param2 = ParsedGenomeParam::from(vals[0], vals[1], &self.genetic_manifest);
        }
        let param3_vals = self.pop_n_in_frame(2);
        let mut param3 = ParsedGenomeParam::Constant(10);
        if let Some(vals) = param3_vals {
            param3 = ParsedGenomeParam::from(vals[0], vals[1], &self.genetic_manifest);
        }

        return Some((op_type, raw_operation_id, param1, param2, param3));
    }

    pub fn compile_operation(&mut self) -> Option<ParamedGeneOperationCall> {
        //println!("COMPILING OPERATION");
        let maybe_op = self.pop_operation();

        if maybe_op.is_none() {
            return None;
        }

        let (op_type, op_id, param1, param2, param3) = maybe_op.unwrap();

        if convert::operation::is_meta_reaction(op_type) {
            let meta_reaction: ParamedMetaReactionCall = match &MetaReaction::from_val(op_id) {
                MetaReaction::JumpAheadFrames => ParamedMetaReactionCall::JumpAheadFrames(param1),
                MetaReaction::SetChannel => ParamedMetaReactionCall::SetChannel(param1),
                MetaReaction::SetRegister => ParamedMetaReactionCall::SetRegister(param1, param2),
                MetaReaction::Nil => ParamedMetaReactionCall::Nil,
            };

            return Some(ParamedGeneOperationCall::MetaReaction(meta_reaction));
        } else {
            let num_reactions = self.genetic_manifest.chemistry_manifest.reactions.len();
            let reaction_id = (op_id as usize % num_reactions) as ReactionId;
            return Some(ParamedGeneOperationCall::Reaction((
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
    use variants::CheeseChemistry;

    use super::super::common::*;
    use super::super::render::render_genes;
    use super::*;

    #[test]
    pub fn test_compile_multiple_frames() {
        use super::super::common::*;
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let mut frame1 = frame_from_single_channel(vec![gene(
            if_any(vec![if_all(vec![
                conditional!(is_truthy, pos_attr::is_cheese_source(0, 0)),
                conditional!(gt, unit_res::cheese, 100),
            ])]),
            then_do!(move_unit, 75),
        )])
        .build(&gm);

        let mut frame2 = frame_from_single_channel(vec![gene(
            if_none(vec![if_not_all(vec![conditional!(
                lt,
                sim_attr::total_cheese_consumed,
                100
            )])]),
            then_do!(new_unit, register(1), 69, 69),
        )])
        .build(&gm);

        let mut genome_words = vec![];
        genome_words.append(&mut frame1.clone());
        genome_words.append(&mut frame2.clone());

        let compiled = FramedGenomeCompiler::compile(genome_words.clone(), &gm);

        println!("genome:\n{}", compiled.display(&gm));
        assert_eq!(compiled.frames.len(), 2);

        assert_eq!(compiled.frames[0].address_range, (0, frame1.len()));
        assert_eq!(
            compiled.frames[1].address_range,
            (frame1.len(), frame1.len() + frame2.len())
        );

        let gene_address_range = compiled.frames[0].channels[0][0].address_range;
        println!("genome_vals: {:?}", &genome_words);
        println!(
            "frame1_address_range: {:?}",
            &compiled.frames[0].address_range
        );
        println!(
            "gene_vals: {:?}",
            &get_from_range(&genome_words, gene_address_range)
        );

        let first_disjunction_addr = &compiled.frames[0].channels[0][0].conditional.address_range;

        let num_clauses = genome_words[first_disjunction_addr.0];
        assert_eq!(num_clauses, 1);
    }

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

pub fn get_from_range<T: Copy>(vec: &Vec<T>, range: (usize, usize)) -> Vec<T> {
    let mut res = vec![];

    for i in range.0..range.1 {
        res.push(vec[i]);
    }

    res
}

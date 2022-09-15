use super::annotated::{FrameExecutionStats, FramedGenomeExecutionStats};
use crate::biology::genome::framed::render::render_gene;
use chemistry::reactions::ReactionCallParam;

use crate::biology::genetic_manifest::predicates::{
    OperatorImplementation, OperatorLibrary, OperatorManifest, OperatorParamDefinition,
    OperatorParamType,
};
use crate::biology::genome::framed::common::*;
use crate::biology::genome::framed::types::NUM_META_REACTIONS;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use std::rc::Rc;
//use crate::simulation::world::World;
pub use crate::chemistry::properties::RawPropertyId;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};
//use crate::biology::unit_behavior::{ParamedReactionCall};

use crate::biology::genome::framed::*;
use crate::biology::unit_behavior::framed::types::*;
use crate::chemistry::reactions::ReactionCall;
use std::convert::TryInto;

#[macro_export]
macro_rules! if_stats_enabled {
    ($code:tt) => {{
        #[cfg(any(not(feature = "skip_genome_stats"), test))]
        {
            $code
        }
    }};
}

pub const MAX_JUMP_AHEAD_FRAMES: u64 = 3;
/** Represents the final result of a gene operation with the literal values that can be used for executing. */
pub enum ExecutableGeneOperation {
    ReactionCall(ReactionCall),

    // jumping ahead 0 frames means terminating the current frame and jumping to the next one
    JumpAheadFrames(u8),
    SetChannel(u8),
    SetRegister(RegisterId, PhenotypeRegisterValue),
}

pub struct GenomeExecutionContext<'a> {
    pub genetic_manifest: &'a GeneticManifest,
    frames: &'a Vec<Frame>,
    current_frame: usize,
    pub override_channel: Option<u8>,
    pub consumed_compute_points: u64,
    pub allotted_compute_points: u64,

    pub sensor_context: &'a SensorContext<'a>,
    pub registers: PhenotypeRegisters,

    pub stats: &'a mut FramedGenomeExecutionStats,
}

impl<'a> GenomeExecutionContext<'a> {
    pub fn new(
        frames: &'a Vec<Frame>,
        sensor_context: &'a SensorContext,
        registers: PhenotypeRegisters,
        gm: &'a GeneticManifest,
        compute_points: u64,
        stats: &'a mut FramedGenomeExecutionStats,
    ) -> Self {
        stats.initialize(frames);
        Self {
            stats: stats,
            genetic_manifest: gm,
            frames,
            sensor_context,
            current_frame: 0,
            override_channel: None,
            registers,
            consumed_compute_points: 0,
            allotted_compute_points: compute_points,
        }
    }

    pub fn execute(&mut self) -> Vec<ReactionCall> {
        self.stats.mark_eval();

        let mut reactions = vec![];
        while self.current_frame < self.frames.len()
            && self.consumed_compute_points < self.allotted_compute_points
        {
            // println!("frame {:?}", &self.frames[self.current_frame]);
            let result = self.execute_frame();
            if result.is_some() {
                reactions.push(result.unwrap());
                return reactions; // ie. we can have only one reaction, for right now
            }

            self.current_frame += 1;
        }

        reactions
    }

    pub fn execute_frame(&mut self) -> Option<ReactionCall> {
        let frame = &self.frames[self.current_frame];

        let channel =
            (self.override_channel.unwrap_or(frame.default_channel) % NUM_CHANNELS as u8) as usize;

        if_stats_enabled! {{
            self.stats.frames[self.current_frame].mark_eval();
            self.stats.frames[self.current_frame].channels[channel].mark_eval();
        }}

        let genes = &frame.channels[channel as usize];

        let frame_channel_idx = (self.current_frame, channel as usize);

        let mut has_evaluated_to_true = false;

        'gene_loop: for (i, gene) in genes.iter().enumerate() {
            let cond = &gene.conditional;

            if_stats_enabled! {{
                self.stats.frames[self.current_frame].channels[channel].genes[i].mark_eval();
            }}

            if self.consumed_compute_points > self.allotted_compute_points {
                return None;
            }

            // println!("executing conditional: {:?}", &cond);
            let cond_result = self.execute_conditional(&cond, &frame_channel_idx, i);

            if cond_result {
                if !has_evaluated_to_true {
                    if_stats_enabled! {{
                        self.stats.frames[self.current_frame].mark_eval_true();
                        self.stats.frames[self.current_frame].channels[channel].mark_eval_true();
                    }}
                }

                if_stats_enabled! {{
                        self.stats.frames[self.current_frame].channels[channel].genes[i].mark_eval_true();
                }}

                has_evaluated_to_true = true;

                let op = &gene.operation;
                let _result = self.evaluate_gene_operation_call(op);
                if let Some(result) = _result {
                    match result {
                        ExecutableGeneOperation::ReactionCall(reaction_call) => {
                            return Some(reaction_call);
                        }
                        ExecutableGeneOperation::JumpAheadFrames(frames) => {
                            self.current_frame += frames as usize;
                            break 'gene_loop;
                        }
                        ExecutableGeneOperation::SetChannel(channel) => {
                            self.override_channel = Some(channel);
                            break 'gene_loop;
                        }
                        ExecutableGeneOperation::SetRegister(reg_id, reg_val) => {
                            self.registers[reg_id] = reg_val;
                        }
                    }
                }
            }
        }

        None
    }
    pub fn evaluate_gene_operation_call(
        &mut self,
        operation: &ParamedGeneOperationCall,
    ) -> Option<ExecutableGeneOperation> {
        self.consumed_compute_points += 1;

        match operation {
            ParamedGeneOperationCall::MetaReaction(meta_reaction) => match meta_reaction {
                ParamedMetaReactionCall::SetChannel(param) => {
                    let param_val = self.eval_param(&param);
                    let channel: u8 = (param_val % NUM_CHANNELS as i32).try_into().unwrap();
                    Some(ExecutableGeneOperation::SetChannel(channel))
                }
                ParamedMetaReactionCall::JumpAheadFrames(param) => {
                    let mut frame_count: u8 = (self.eval_param(&param)
                        % MAX_JUMP_AHEAD_FRAMES as i32)
                        .try_into()
                        .unwrap();
                    Some(ExecutableGeneOperation::JumpAheadFrames(frame_count))
                }
                ParamedMetaReactionCall::SetRegister(r, v) => {
                    let reg_id = (self.eval_param(&r)
                        % self.genetic_manifest.number_of_registers as i32)
                        .try_into()
                        .unwrap();
                    let reg_val = (self.eval_param(&v) % u16::MAX as i32).try_into().unwrap();

                    Some(ExecutableGeneOperation::SetRegister(reg_id, reg_val))
                }

                ParamedMetaReactionCall::Nil => None,
            },
            ParamedGeneOperationCall::Reaction(paramed_reaction_call) => {
                self.consumed_compute_points += 1;

                let param_val1 = self.eval_param(&paramed_reaction_call.1);
                let param_val2 = self.eval_param(&paramed_reaction_call.2);
                let param_val3 = self.eval_param(&paramed_reaction_call.3);

                //flog!("REACTION TO EXECUTE: {:?}", paramed_reaction_call);
                Some(ExecutableGeneOperation::ReactionCall((
                    paramed_reaction_call.0,
                    (param_val1 % (u16::MAX as i32)).try_into().unwrap(),
                    (param_val2 % (u16::MAX as i32)).try_into().unwrap(),
                    (param_val3 % (u16::MAX as i32)).try_into().unwrap(),
                )))
            }
            ParamedGeneOperationCall::Nil => None,
        }
    }

    pub fn execute_conditional(
        &mut self,
        conditional: &Disjunction,
        frame_and_channel_idx: &(usize, usize),
        gene_idx: usize,
    ) -> bool {
        if_stats_enabled! {{
            self.stats.frames[frame_and_channel_idx.0].channels[frame_and_channel_idx.1].genes
                [gene_idx]
                .disjunction_expression
                .mark_eval();
        }}

        let mut or_result = false;
        let is_negated = conditional.is_negated;

        // loop OR sub-expressions
        for (conjunction_idx, conjunctive) in conditional.conjunctive_clauses.iter().enumerate() {
            if_stats_enabled! {{
                self.stats.frames[frame_and_channel_idx.0].channels[frame_and_channel_idx.1].genes
                    [gene_idx]
                    .disjunction_expression
                    .conjunctive_expressions[conjunction_idx]
                    .mark_eval();
            }}

            let is_negated = conjunctive.is_negated;
            let mut and_result = true;

            // loop AND sub-expressions
            'and_loop: for (bool_idx, bool_expr) in conjunctive.boolean_variables.iter().enumerate()
            {
                if_stats_enabled! {{
                        self.stats.frames[frame_and_channel_idx.0].channels[frame_and_channel_idx.1].genes
                            [gene_idx]
                            .disjunction_expression
                            .conjunctive_expressions[conjunction_idx]
                            .bool_conditionals[bool_idx]
                            .mark_eval();
                }}

                if !self.execute_boolean(bool_expr) {
                    and_result = false;
                    break 'and_loop;
                } else {
                    if_stats_enabled! {{
                        self.stats.frames[frame_and_channel_idx.0].channels[frame_and_channel_idx.1]
                            .genes[gene_idx]
                            .disjunction_expression
                            .conjunctive_expressions[conjunction_idx]
                            .bool_conditionals[bool_idx]
                            .mark_eval_true();
                    }}
                }
            }

            if and_result ^ is_negated {
                or_result = true;

                if_stats_enabled! {{
                        self.stats.frames[frame_and_channel_idx.0].channels[frame_and_channel_idx.1].genes
                            [gene_idx]
                            .disjunction_expression
                            .conjunctive_expressions[conjunction_idx]
                            .mark_eval_true();
                }}
                break; // break early
            }
        }

        if or_result ^ is_negated {
            if_stats_enabled! {{
                self.stats.frames[frame_and_channel_idx.0].channels[frame_and_channel_idx.1].genes
                    [gene_idx]
                    .disjunction_expression
                    .mark_eval_true();
            }}
            true
        } else {
            false
        }
    }
    pub fn eval_param(&mut self, parsed_param: &ParsedGenomeParam) -> i32 {
        use rand::Rng;

        match parsed_param {
            ParsedGenomeParam::Constant(c) => *c as i32,
            ParsedGenomeParam::SensorLookup(sensor_id) => {
                let sensor = &self.genetic_manifest.sensor_manifest.sensors[*sensor_id as usize];
                let val = sensor.calculate(self.sensor_context);
                val
            }

            ParsedGenomeParam::Register(register_id) => {
                // panic!("registers not supported yet");
                // TODO!
                //self.registers[*register_id as usize] as i32
                0
            }
            ParsedGenomeParam::Random(max_val) => {
                let mut rng = rand::thread_rng();

                if max_val == &0 {
                    0 as i32
                } else {
                    rng.gen_range(0..*max_val as usize) as i32
                }
            }
        }
    }

    pub fn execute_boolean(&mut self, boolean_clause: &BooleanVariable) -> bool {
        self.consumed_compute_points += 1;
        match boolean_clause {
            BooleanVariable::Literal(v) => {
                //self.consumed_compute_points += 1;
                *v
            }
            BooleanVariable::Conditional(op_id, is_negated, param1, param2, param3) => {
                self.consumed_compute_points += 1;
                let op = &self.genetic_manifest.operator_manifest.operators[*op_id as usize];

                let param_val1 = self.eval_param(param1);
                let param_val2 = self.eval_param(param2);
                let param_val3 = self.eval_param(param3);

                // let op_str = (op.render)(&[
                //     param_val1.to_string(),
                //     param_val2.to_string(),
                //     param_val3.to_string(),
                // ]);
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

pub mod tests {
    use crate::{
        biology::{
            genome::framed::annotated::FramedGenomeExecutionStats,
            unit_behavior::framed::GenomeExecutionContext,
        },
        simulation::common::{
            helpers::place_units::PlaceUnitsMethod, properties::CheeseChemistry,
            variants::FooChemistry, Chemistry, ChemistryConfiguration, ChemistryInstance,
            ChemistryManifest, GeneticManifest, NullBehavior, SensorContext, SimulationBuilder,
            UnitEntry, UnitManifest,
        },
    };

    use super::super::common::*;

    pub fn sim_builder(chemistry: ChemistryInstance) -> SimulationBuilder {
        SimulationBuilder::default()
            .chemistry(chemistry)
            .size((5, 5))
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(1, 1)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
    }
    #[test]
    pub fn test_set_register() {
        let chemistry = FooChemistry::construct_with_default_config();
        let gm =
            GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new()).wrap_rc();

        let mut frame1 = frame_from_single_channel(vec![
            gene(
                if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                then_do!(set_register, 0, 100),
            ),
            gene(
                if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                then_do!(set_register, 1, 101),
            ),
            gene(
                if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                then_do!(set_register, 2, 102),
            ),
            gene(
                if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                then_do!(set_register, 3, 103),
            ),
            gene(
                if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                then_do!(set_register, 4, 104),
            ),
            gene(
                if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                then_do!(set_register, 5, 111),
            ),
        ])
        .build(&gm);

        let mut genome_words = vec![];
        genome_words.append(&mut frame1);

        let compiled = FramedGenomeCompiler::compile(genome_words, &gm);

        let registers = gm.empty_registers();

        let sim = sim_builder(chemistry).to_simulation();
        let sensor_context = SensorContext::from(&sim.world, &sim.attributes, &(1, 1));

        let mut stats = FramedGenomeExecutionStats::empty();

        let mut execution = GenomeExecutionContext::new(
            &compiled.frames,
            &sensor_context,
            registers,
            &gm,
            10000,
            &mut stats,
        );
        let result = execution.execute();

        println!("execution: {:?}", result);
        println!("registers: {:?}", execution.registers);

        // setting the register at r_id=5 means overflowing to the r_id=0 register
        assert_eq!(execution.registers, vec![111, 101, 102, 103, 104]);
    }

    #[test]
    pub fn test_set_channel() {
        let chemistry = FooChemistry::construct_with_default_config();
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let raw_genome = framed_genome(vec![
            frame(
                vec![
                    gene(
                        if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                        then_do!(set_register, 0, 100),
                    ),
                    gene(
                        if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                        then_do!(set_channel, 1),
                    ),
                    gene(
                        if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                        then_do!(set_register, 0, 101),
                    ),
                ],
                vec![],
                vec![],
                vec![],
            ),
            frame(
                vec![],
                vec![gene(
                    if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                    then_do!(set_register, 1, 1337),
                )],
                vec![],
                vec![],
            ),
        ])
        .build(&gm);

        let compiled = FramedGenomeCompiler::compile(raw_genome, &gm);

        let registers = gm.empty_registers();

        let sim = sim_builder(chemistry).to_simulation();
        let sensor_context = SensorContext::from(&sim.world, &sim.attributes, &(1, 1));

        let mut stats = FramedGenomeExecutionStats::empty();
        let mut execution = GenomeExecutionContext::new(
            &compiled.frames,
            &sensor_context,
            registers,
            &gm,
            10000,
            &mut stats,
        );
        let result = execution.execute();

        println!("execution: {:?}", result);
        println!("registers: {:?}", execution.registers);

        // setting the register at r_id=5 means overflowing to the r_id=0 register
        assert_eq!(execution.registers[0], 100);
        assert_eq!(execution.registers[1], 1337);
        assert_eq!(execution.override_channel, Some(1));

        assert_eq!(execution.stats.eval_count.get(), 1);
        assert_eq!(execution.stats.frames[0].channels[0].eval_count.get(), 1);
        assert_eq!(execution.stats.frames[0].channels[1].eval_count.get(), 0);
        assert_eq!(execution.stats.frames[1].channels[1].eval_count.get(), 1);

        assert_eq!(
            execution.stats.frames[0].channels[0].eval_true_count.get(),
            1
        );
    }

    #[test]
    pub fn test_jump_ahead() {
        let chemistry = FooChemistry::construct_with_default_config();
        let gm = GeneticManifest::construct::<CheeseChemistry>(&ChemistryConfiguration::new());

        let raw_genome = framed_genome(vec![
            frame(
                vec![
                    gene(
                        if_any(vec![
                            if_all(vec![conditional!(false)]),
                            if_all(vec![conditional!(true)]),
                            if_all(vec![conditional!(false)]),
                        ]),
                        then_do!(set_register, 0, 100),
                    ),
                    gene(
                        if_any(vec![if_all(vec![conditional!(true)])]),
                        then_do!(jump_ahead_frames, 1),
                    ),
                    gene(
                        if_any(vec![if_all(vec![conditional!(true)])]),
                        then_do!(set_register, 0, 101),
                    ),
                ],
                vec![],
                vec![],
                vec![],
            ),
            frame(
                vec![gene(
                    if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                    then_do!(set_register, 2, 13),
                )],
                vec![],
                vec![],
                vec![],
            ),
            frame(
                vec![gene(
                    if_any(vec![if_all(vec![conditional!(is_truthy, 1)])]),
                    then_do!(set_register, 1, 1337),
                )],
                vec![],
                vec![],
                vec![],
            ),
        ])
        .build(&gm);

        let compiled = FramedGenomeCompiler::compile(raw_genome, &gm);

        let registers = gm.empty_registers();

        let sim = sim_builder(chemistry).to_simulation();
        let sensor_context = SensorContext::from(&sim.world, &sim.attributes, &(1, 1));

        let mut stats = FramedGenomeExecutionStats::empty();
        let mut execution = GenomeExecutionContext::new(
            &compiled.frames,
            &sensor_context,
            registers,
            &gm,
            10000,
            &mut stats,
        );
        let result = execution.execute();

        println!("execution: {:?}", result);
        println!("registers: {:?}", execution.registers);

        assert_eq!(execution.registers[0], 100);
        assert_eq!(execution.registers[1], 1337);
        assert_ne!(execution.registers[2], 13);

        assert_eq!(execution.stats.eval_count.get(), 1);
        assert_eq!(execution.stats.frames[0].eval_count.get(), 1);
        assert_eq!(
            execution.stats.frames[0].channels[0].genes[0]
                .disjunction_expression
                .conjunctive_expressions[0]
                .eval_true_count
                .get(),
            0
        );
        assert_eq!(
            execution.stats.frames[0].channels[0].genes[0]
                .disjunction_expression
                .conjunctive_expressions[1]
                .eval_true_count
                .get(),
            1
        );

        assert_eq!(
            execution.stats.frames[0].channels[0].genes[0]
                .disjunction_expression
                .conjunctive_expressions[2]
                .eval_count
                .get(),
            0
        );
        assert_eq!(
            execution.stats.frames[0].channels[0].genes[0]
                .disjunction_expression
                .conjunctive_expressions[2]
                .eval_true_count
                .get(),
            0
        );
    }
}

use super::types::{BooleanVariable, Conjunction};
use super::types::{
    FramedGenomeValue, FramedGenomeWord, CHANNEL_ZERO, FIXED_NUM_CONDITIONAL_PARAMS,
    FIXED_NUM_OPERATION_PARAMS, FRAME_META_DATA_SIZE, MIN_FRAME_SIZE, NUM_CHANNELS,
};
use crate::biology::genetic_manifest::predicates::{
    OperatorId, OperatorImplementation, OperatorManifest, OperatorParam, OperatorParamDefinition,
    OperatorParamType,
};
use crate::biology::sensor_manifest::SensorManifest;
use crate::biology::unit_behavior::framed::ParsedGenomeParam;
use crate::biology::unit_behavior::UnitBehavior;
use crate::chemistry::properties::AttributeIndex;
use crate::chemistry::{ChemistryInstance, ReactionId};
use crate::simulation::common::*;
use crate::simulation::world::World;
use crate::util::Coord;
use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use crate::chemistry;

pub const BITS_PER_CHANNEL: usize = 16;

pub mod param_meta {
    use super::*;
    const MAX_PARAM_META_VAL: u16 = 4;
    pub fn is_constant(val: u16) -> bool {
        val % MAX_PARAM_META_VAL == val_for_constant() as u16
    }

    pub fn is_sensor_lookup(val: u16) -> bool {
        val % MAX_PARAM_META_VAL == val_for_sensor_lookup() as u16
    }
    pub fn is_register_lookup(val: u16) -> bool {
        val % MAX_PARAM_META_VAL == val_for_register_lookup() as u16
    }
    pub fn is_random_lookup(val: u16) -> bool {
        val % MAX_PARAM_META_VAL == val_for_random_lookup() as u16
    }

    pub fn val_for_constant() -> u8 {
        0
    }

    pub fn val_for_sensor_lookup() -> u8 {
        1
    }
    pub fn val_for_register_lookup() -> u8 {
        2
    }
    pub fn val_for_random_lookup() -> u8 {
        3
    }

    // three meta flags are encoded into a single FramedGenomeValue
    pub fn params_meta_to_value(params_meta: [u8; 3]) -> FramedGenomeValue {
        let quad: u8 = 0b1111;

        let first = ((params_meta[0] & quad) as u16) << 0;
        let second = ((params_meta[1] & quad) as u16) << 4 as u16;
        let third = ((params_meta[2] & quad) as u16) << 8 as u16;
        (first | second | third) as FramedGenomeValue
    }

    pub fn val_to_params_meta(v: FramedGenomeValue) -> [u8; 3] {
        let quad: u16 = 0b1111;
        let mut params_meta: [u8; FIXED_NUM_CONDITIONAL_PARAMS] = [0; FIXED_NUM_CONDITIONAL_PARAMS];

        params_meta[0] = ((quad << 0 & v) >> 0) as u8;
        params_meta[1] = ((quad << 4 & v) >> 4) as u8;
        params_meta[2] = ((quad << 8 & v) >> 8) as u8;

        return params_meta;
    }
}

pub mod operation {
    pub const OPERATION_TYPE_REACTION: FramedGenomeValue = 0;
    pub const OPERATION_TYPE_META_REACTION: FramedGenomeValue = 1;

    use crate::biology::genome::framed::common::FramedGenomeValue;

    pub fn val_for_reaction_operation_type() -> FramedGenomeValue {
        0
    }

    pub fn val_for_metareaction_operation_type() -> FramedGenomeValue {
        1
    }

    pub fn is_meta_reaction(op_type: FramedGenomeValue) -> bool {
        op_type % 2 == OPERATION_TYPE_META_REACTION
    }
}

pub fn val_to_n_conditionals(val: FramedGenomeValue) -> usize {
    val as usize & 0b11
}

pub fn val_to_n_or_clauses(val: FramedGenomeValue) -> usize {
    val as usize & 0b11
}

pub fn val_for_n_or_clauses(n: usize) -> FramedGenomeWord {
    n as FramedGenomeWord
}

pub fn val_to_n_and_clauses(val: FramedGenomeValue) -> usize {
    val as usize & 0b11
}

pub fn val_for_n_and_clauses(n: usize) -> FramedGenomeWord {
    n as FramedGenomeWord
}

pub fn val_for_frame_size(val: usize) -> FramedGenomeWord {
    (val) as FramedGenomeWord
}

/**
 * get the raw val that would represent that channel
 */
pub fn val_for_channel(channel: u8) -> FramedGenomeValue {
    // TODO: ANDing it here as a sanity check, but should it be an error?
    (channel & 0b011) as FramedGenomeValue
}

pub fn val_to_frame_size(val: FramedGenomeValue) -> usize {
    (val as usize & 0b111111) // range [0-63)
}

pub fn val_to_channel(val: FramedGenomeValue) -> u8 {
    (val & 0b11) as u8
}

/**
 * Given a genomic program expressed in FramedGenomeValues (ie. u16), convert this
 * into a full FramedGenome, with the destination being the zeroeth channel and the
 * default channel being the zeroeth channel.
 */
pub fn simple_convert_into_frames(values: Vec<FramedGenomeValue>) -> Vec<FramedGenomeWord> {
    convert_into_framed_channels(values, None, 0, 0)
}

/**
 * Given a list of genome values, creates a list of genome words that expresses those values
 * as framed genome.
 */
pub fn convert_into_framed_channels(
    values: Vec<FramedGenomeValue>,

    /*
     * frame size is optional.  if it's given but is smaller than the actual size of the list of values,
     * then undefined behavior could occur when executing the genome.
     */
    frame_size: Option<usize>,
    default_channel: u8,
    dest_channel: u8,
) -> Vec<FramedGenomeWord> {
    let mut result = values
        .iter()
        .map(|v| -> u64 { convert_val_to_channel!(dest_channel, *v) as u64 })
        .collect::<Vec<u64>>();

    let frame_size = frame_size.unwrap_or(values.len());

    // encode frame size (ie. first value in frame)
    result.insert(
        0,
        convert_val_to_channel!(CHANNEL_ZERO, val_for_frame_size(frame_size)),
    );
    // encode the henceforth active channel on the zeroeth channel (ie. second value in frame)
    result.insert(
        1,
        convert_val_to_channel!(CHANNEL_ZERO, val_for_channel(default_channel)),
    );

    result
}

pub fn mask_for_channel(channel: u8) -> FramedGenomeWord {
    let bits = (channel % 4) * 16;
    0xffff << bits as FramedGenomeWord
}

pub fn merge_value_into_word(
    word: FramedGenomeWord,
    value: FramedGenomeValue,
    into_channel: u8,
) -> FramedGenomeWord {
    let mask = 0;

    let word1 = word & !mask_for_channel(into_channel);
    let word2 = (value as FramedGenomeWord) << into_channel * 16;

    word1 | word2
}

/**
 * Maybe this could be renamed to compile_channels_into_frame, because
 * it's also handling the compilation of metadata in a way that's specific to the Framed genome design
 * */
pub fn merge_channels_into_frame(
    channel_values: Vec<Vec<FramedGenomeValue>>,
    default_channel: u8,
) -> Vec<FramedGenomeWord> {
    let mut max_length = 0;

    // find length of the longest channel
    for values in channel_values.iter() {
        if values.len() > MAX_FRAME_LENGTH as usize {
            panic!(
                "Channel length exceeds the max frame size: {}",
                MAX_FRAME_LENGTH
            );
        }
        if max_length < values.len() {
            max_length = values.len()
        }
    }

    let mut result: Vec<FramedGenomeWord> = vec![];
    let frame_size = max_length;

    // let meta_data = vec![
    //     // encode the henceforth active channel on the zeroeth channel (ie. second value in frame)
    //     convert_val_to_channel!(CHANNEL_ZERO, val_for_frame_size(frame_size)),
    //     // encode frame size (ie. first value in frame)
    //     convert_val_to_channel!(CHANNEL_ZERO, val_for_channel(default_channel)),
    // ];

    // encode the henceforth active channel on the zeroeth channel (ie. second value in frame)
    result.insert(
        0,
        convert_val_to_channel!(CHANNEL_ZERO, val_for_channel(default_channel)),
    );
    // encode frame size (ie. first value in frame)
    result.insert(
        0,
        convert_val_to_channel!(CHANNEL_ZERO, val_for_frame_size(frame_size)),
    );

    for i in (0..max_length) {
        let mut merged: FramedGenomeWord = 0;
        for channel in (0..NUM_CHANNELS) {
            let channel_val = if i < channel_values[channel].len() {
                channel_values[channel][i]
            } else {
                0
            };

            merged = merged | convert_val_to_channel!(channel, channel_val);
        }

        result.push(merged)
    }

    result
}

#[derive(Clone, Debug)]
pub struct RawFrame {
    pub default_channel: u8,
    pub channel_values: [Vec<FramedGenomeValue>; NUM_CHANNELS],

    // this address range includes the metadata.
    pub address_range: (usize, usize),
}

/**
 * Receives a raw genome and outputs a list of raw frames.
 */
pub struct RawFrameParser {
    idx: usize,
    current_frame: (usize, usize),
    values: Vec<FramedGenomeWord>,
}

pub const MAX_FRAME_LENGTH: u64 = 300;

impl RawFrameParser {
    pub fn parse(values: Vec<FramedGenomeWord>) -> Vec<RawFrame> {
        let mut parser = RawFrameParser {
            idx: 0,
            current_frame: (0, 0),
            values,
        };

        parser.unroll_channels_from_from_frames()
    }

    fn frame_val_to_frame_size(val: FramedGenomeWord) -> u16 {
        (val % MAX_FRAME_LENGTH) as u16
    }

    /**
     *
     */
    pub fn unroll_channels_from_from_frames(&mut self) -> Vec<RawFrame> {
        let mut result = vec![];

        while (self.idx as i64) < (self.values.len() as i64 - FRAME_META_DATA_SIZE as i64) {
            let frame_size = Self::frame_val_to_frame_size(self.values[self.idx]) as usize;

            let mut default_channel = self.values[self.idx + 1] as u8;

            /*
              current_frame specifies the address that starts the metadata and the end address (which I think is non-inclusive)
              the frame_size does NOT include the meta_data because the meta_data always needs to exist,
              even if the encoded frame_size is zero
            */

            self.current_frame = (self.idx, self.idx + FRAME_META_DATA_SIZE + frame_size);

            let channel_values = self.unroll_inner_frame(frame_size);

            result.push(RawFrame {
                channel_values,
                default_channel,
                address_range: self.current_frame.clone(),
            })
        }
        result
    }

    /**
     *
     *
     * */
    pub fn unroll_inner_frame(&mut self, size: usize) -> [Vec<FramedGenomeValue>; 4] {
        let mut channels = vec![];

        for channel in (0..NUM_CHANNELS) {
            let mut channel_values = vec![];

            self.idx = self.current_frame.0 + FRAME_META_DATA_SIZE;

            while self.idx < self.values.len() && self.idx < self.current_frame.1 {
                channel_values.push(get_val_from_channel!(channel, self.values[self.idx]));
                self.idx += 1;
            }

            channels.push(channel_values);
        }

        [
            channels[0].clone(),
            channels[1].clone(),
            channels[2].clone(),
            channels[3].clone(),
        ]
    }
}

pub mod tests {
    use variants::CheeseChemistry;

    use super::param_meta;
    use super::*;
    use crate::biology::genetic_manifest::predicates::default_operators;
    use crate::simulation::common::*;

    #[test]
    pub fn params_meta_to_value() {
        assert_eq!(param_meta::params_meta_to_value([0, 0, 0]), 0);
        assert_eq!(
            param_meta::params_meta_to_value([0b0101, 0b1111, 0b0011]),
            0b001111110101
        );
    }

    #[test]
    pub fn val_to_params_meta() {
        assert_eq!(
            param_meta::val_to_params_meta(0b101011110000),
            [0b0000, 0b1111, 0b1010]
        );
    }
    #[test]
    pub fn test__merge_channels() {
        let channel_values = vec![
            vec![1, 2, 3, 4, 5],
            vec![5, 4, 3, 2, 1],
            vec![10], // a smaller channel
            vec![15, 16, 17, 18, 19],
        ];
        let result = merge_channels_into_frame(channel_values, 0);

        assert_eq!(result.len(), 7);
        assert_eq!(result[0], 5);
        assert_eq!(result[1], 0);

        assert_eq!(get_val_from_channel!(0, result[2]), 1);
        assert_eq!(get_val_from_channel!(1, result[2]), 5);
        assert_eq!(get_val_from_channel!(2, result[2]), 10);
        assert_eq!(get_val_from_channel!(3, result[2]), 15);
        assert_eq!(get_val_from_channel!(0, result[6]), 5);
        assert_eq!(get_val_from_channel!(1, result[6]), 1);
        assert_eq!(get_val_from_channel!(2, result[6]), 0);
        assert_eq!(get_val_from_channel!(3, result[6]), 19);
    }
    #[test]
    pub fn convert_into_framed_channels() {
        assert_eq!(
            super::convert_into_framed_channels(vec![1, 2, 3, 4, 5], None, 0, 0),
            vec![5, 0, 1, 2, 3, 4, 5]
        );
    }
    #[test]
    pub fn raw_frame_parser__one_frame() {
        let channel_values = vec![
            vec![1, 2, 3, 4, 5],
            vec![5, 4, 3, 2, 1],
            vec![10], // a smaller channel
            vec![15, 16, 17, 18, 19],
        ];
        let channel_values = merge_channels_into_frame(channel_values, 1);
        let result = RawFrameParser::parse(channel_values);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].default_channel, 1);
    }

    #[test]
    pub fn raw_frame_parser__n_frames() {
        let mut raw_frame_1 = merge_channels_into_frame(
            vec![
                vec![1, 2, 3, 4, 5],
                vec![5, 4, 3, 2, 1],
                vec![10], // a smaller channel
                vec![15, 16, 17, 18, 19],
            ],
            1,
        );

        let mut raw_frame_2 = merge_channels_into_frame(
            vec![
                vec![9, 10], // a smaller channel
                vec![1, 2, 3, 4, 5],
                vec![15, 16, 17, 18, 19],
                vec![5, 4, 3, 2, 1],
            ],
            2,
        );
        let mut raw_frames: Vec<FramedGenomeWord> = vec![];
        raw_frames.append(&mut raw_frame_1);
        raw_frames.append(&mut raw_frame_2);

        let result = RawFrameParser::parse(raw_frames);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].default_channel, 1);
        assert_eq!(result[1].default_channel, 2);

        assert_eq!(result[0].channel_values[0][0], 1);
        assert_eq!(result[0].channel_values[1][0], 5);
        assert_eq!(result[0].channel_values[2][0], 10);
        assert_eq!(result[0].channel_values[3][0], 15);
        assert_eq!(result[1].channel_values[0][3], 0);
        assert_eq!(result[1].channel_values[1][3], 4);
        assert_eq!(result[1].channel_values[2][3], 18);
        assert_eq!(result[1].channel_values[3][3], 2);
    }

    #[test]
    pub fn test_mask_for_channel() {
        let test = 0x1234567887654321;

        assert_eq!(mask_for_channel(0), 0xffff);
        assert_eq!(mask_for_channel(1), 0xffff0000);
        assert_eq!(mask_for_channel(2), 0xffff00000000);
        assert_eq!(mask_for_channel(0) & test, 0x4321);
        assert_eq!(mask_for_channel(1) & test, 0x87650000);

        assert_eq!(mask_for_channel(2) & test, 0x567800000000);
        assert_eq!(mask_for_channel(3) & test, 0x1234000000000000);
    }

    #[test]
    pub fn test_merge_value_into_word() {
        let test = 0x1234567887654321;

        assert_eq!(merge_value_into_word(test, 0x1111, 0), 0x1234567887651111);
        assert_eq!(merge_value_into_word(test, 0x1111, 1), 0x1234567811114321);
        assert_eq!(merge_value_into_word(test, 0x1111, 2), 0x1234111187654321);
        assert_eq!(merge_value_into_word(test, 0x1111, 3), 0x1111567887654321);
    }

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

        let frames = RawFrameParser::parse(genome_words);
        assert_eq!(frames.len(), 2);

        assert_eq!(
            frames[0].channel_values[0].len() + FRAME_META_DATA_SIZE,
            frame1.len()
        );
        assert_eq!(
            frames[0].channel_values[1].len() + FRAME_META_DATA_SIZE,
            frame1.len()
        );

        for i in 0..frames.len() {
            assert_eq!(frames[i].default_channel, 0);
        }

        // let compiled = FramedGenomeCompiler::compile(genome_words, &gm);

        // println!("genome:\n{}", frames.display(&gm));
        assert_eq!(frames.len(), 2);
    }
}

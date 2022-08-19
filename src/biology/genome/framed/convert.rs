use super::types::{BooleanVariable, ConjunctiveClause};
use super::types::{
    FramedGenomeValue, FramedGenomeWord, CHANNEL_ZERO, FIXED_NUM_CONDITIONAL_PARAMS,
    FIXED_NUM_OPERATION_PARAMS, FRAME_META_DATA_SIZE, MIN_FRAME_SIZE, NUM_CHANNELS,
};
use crate::biology::genetic_manifest::predicates::{
    Operator, OperatorId, OperatorParam, OperatorParamDefinition, OperatorParamType, OperatorSet,
};
use crate::biology::genetic_manifest::GeneticManifest;
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

// get the raw val that would represent that channel
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

pub fn simple_convert_into_frames(values: Vec<FramedGenomeValue>) -> Vec<FramedGenomeWord> {
    convert_into_framed_channels(values, None, 0, 0, None)
}

pub fn convert_into_framed_channels(
    values: Vec<FramedGenomeValue>,
    frame_size: Option<usize>,
    default_channel: u8,
    dest_channel: u8,
    padding: Option<usize>,
) -> Vec<FramedGenomeWord> {
    let mut result = values
        .iter()
        .map(|v| -> u64 { convert_val_to_channel!(dest_channel, *v) as u64 })
        .collect::<Vec<u64>>();
    let frame_padding = padding.unwrap_or(5);
    let frame_size = frame_size.unwrap_or(FRAME_META_DATA_SIZE + values.len() + frame_padding);

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

    // the padding is in its own frame
    result.push(convert_val_to_channel!(CHANNEL_ZERO, frame_padding));
    for i in 0..frame_padding {
        result.push(convert_val_to_channel!(CHANNEL_ZERO, 123));
    }

    result
}

pub fn merge_channels_into_frame(
    channel_values: Vec<Vec<FramedGenomeValue>>,
    default_channel: u8,
) -> Vec<FramedGenomeWord> {
    let mut max_length = 0;
    for values in channel_values.iter() {
        if max_length < values.len() {
            max_length = values.len()
        }
    }

    let mut result: Vec<FramedGenomeWord> = vec![];
    let frame_size = max_length;

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
}

pub struct RawFrameParser {
    idx: usize,
    current_frame: (usize, usize),
    values: Vec<FramedGenomeWord>,
}

impl RawFrameParser {
    pub fn parse(values: Vec<FramedGenomeWord>) -> Vec<RawFrame> {
        let mut parser = RawFrameParser {
            idx: 0,
            current_frame: (0, 0),
            values,
        };

        parser.unroll_channels_from_from_frames()
    }

    pub fn unroll_channels_from_from_frames(&mut self) -> Vec<RawFrame> {
        let mut result = vec![];

        while (self.idx as i64) < (self.values.len() as i64 - FRAME_META_DATA_SIZE as i64) {
            let frame_size = self.values[self.idx] as usize;
            let mut default_channel = self.values[self.idx + 1] as u8;

            self.current_frame = (self.idx, self.idx + FRAME_META_DATA_SIZE + frame_size);

            let channel_values = self.unroll_inner_frame(frame_size);

            result.push(RawFrame {
                channel_values,
                default_channel,
            })
        }
        result
    }

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
            super::convert_into_framed_channels(vec![1, 2, 3, 4, 5], None, 0, 0, Some(2)),
            vec![9, 0, 1, 2, 3, 4, 5, 2, 123, 123]
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
}

use crate::biology::genetic_manifest::predicates::OperatorParam;
use crate::biology::genome::framed::common::*;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::*;
use crate::util::{grid_direction_from_string, grid_direction_to_num};

/**
 *  Given a value, and a channel number, create a Word that has that value in that channel
 * but all other channels zeroed out.
 */
#[macro_export]
macro_rules! convert_val_to_channel {
    ($channel:expr, $val:expr) => {{
        use crate::biology::genome::framed::types::FramedGenomeWord;

        let bits = (($channel as usize % 4) * 16) as u64;
        let result = (($val as FramedGenomeWord & 0xffff) << bits);
        //println!("val_to_channel({:x}) | bits to shift: {}, | result:{:x}", $val, bits, result);
        //println!("val_to_channel(val: 0x{:x}, channel:0x{}) -> 0x{:x}", $val, $channel, result);
        result as FramedGenomeWord
    }};
}

/**
 * Given a word value and channel, extract the value from that word
 * from the given channel.
 */
#[macro_export]
macro_rules! get_val_from_channel {
    ($channel:expr, $val:expr) => {{
        let bits = ($channel as usize % 4) * 16;
        (($val as FramedGenomeWord >> bits as usize) & 0xffff) as FramedGenomeValue
    }};
}

macro_rules! is_execution_logging_enabled {
    //( ) => { false }
    ( ) => {
        false
    };
}

#[macro_export]
macro_rules! flog {
    ($($arg:tt)*) => ({
        if is_execution_logging_enabled!() {println!($($arg)*)} else {}
    })
}

/**
 * Converts a human-readable string describing a parameter to a conditional into an
 * enum encoding that parameter.
 *
 * eg.
 *
 * "pos_attr::is_cheese_source"
 */
pub fn identify_raw_param_string(str_param: &String, gm: &GeneticManifest) -> ParsedGenomeParam {
    //println!("identify_raw_param_string() called for {}", &str_param);
    let s = str_param.to_ascii_lowercase().clone();
    if s.starts_with("constant") {
        let new_s = s.replace("constant", "");
        let key = new_s.trim_start_matches("(").trim_end_matches(")").clone();

        let num = key.parse::<OperatorParam>();
        if num.is_ok() {
            // if it's just a number then we treat it as a raw property id
            return ParsedGenomeParam::Constant(num.unwrap() as OperatorParam);
        }

        panic!(
            "Can't process op param as constant: {}, {}",
            str_param, &key
        );
    }
    if s.starts_with("random") && s.ends_with(")") {
        let new_s = s.replace("random", "");
        let key = new_s
            .trim_start_matches("(")
            .trim_end_matches(")")
            .to_string();

        // println!("RANDOM: {}, {}, {}", &str_param, &new_s, &key);
        let maybe_random_max = key.parse::<RegisterId>();
        if maybe_random_max.is_ok() {
            let random_max = maybe_random_max.unwrap() % gm.number_of_registers;

            return ParsedGenomeParam::Random(random_max);
        }
        panic!("Can't process op param as register: {}", str_param);
    }

    if s.starts_with("register") {
        let new_s = s.replace("register", "");
        let key = new_s
            .trim_start_matches("(")
            .trim_end_matches(")")
            .to_string();

        println!("REGISTER_PARAM: {}, {}, {}", &str_param, &new_s, &key);
        let maybe_register_id = key.parse::<RegisterId>();
        let maybe_register_id = key.parse::<RegisterId>();
        if maybe_register_id.is_ok() {
            let register_id = maybe_register_id.unwrap() % gm.number_of_registers;

            return ParsedGenomeParam::Register(register_id as RegisterId);
        }
        panic!("Can't process op param as register: {}", str_param);
    }

    let key = s.trim_start_matches("(").trim_end_matches(")");
    let maybe_sensor_id = gm.sensor_manifest.identify_sensor_from_key(str_param);
    if maybe_sensor_id.is_some() {
        return ParsedGenomeParam::SensorLookup(maybe_sensor_id.unwrap().id);
    }
    let num = str_param.parse::<OperatorParam>();
    if num.is_ok() {
        // if it's just a number then we treat it as a raw property id
        return ParsedGenomeParam::Constant(num.unwrap() as OperatorParam);
    }

    if let Some(dir) = grid_direction_from_string(str_param) {
        return ParsedGenomeParam::Constant(grid_direction_to_num(dir) as i32);
    }

    panic!("Invalid input for conditional parameter: {}", str_param);
}

#[cfg(test)]
pub mod tests {
    #[test]
    pub fn test_macro__basic() {
        assert_eq!(convert_val_to_channel!(0, 0x1), 0x1);
        assert_eq!(convert_val_to_channel!(1, 0x1), 0x10000);
        assert_eq!(convert_val_to_channel!(2, 0x1), 0x100000000);
        assert_eq!(convert_val_to_channel!(3, 0x1), 0x1000000000000);
    }
}

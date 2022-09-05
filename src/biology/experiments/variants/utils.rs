use crate::biology::experiments::alterations;
use crate::biology::genetic_manifest::GeneticManifest;
use crate::biology::unit_behavior::framed::common::*;
use crate::simulation::common::builder::ChemistryBuilder;
use crate::simulation::common::*;
use crate::{
    biology::genome::framed::common::*, simulation::common::helpers::place_units::PlaceUnitsMethod,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Result};
use std::path::PathBuf;

const DATA_DIR_NAME: &str = "data";

pub fn get_data_dir() -> PathBuf {
    find_folder::Search::Parents(1)
        .for_folder(DATA_DIR_NAME)
        .expect("cant find data dir")
        .to_path_buf()
}

pub fn get_experiments_dir() -> PathBuf {
    let mut dir = get_data_dir();
    dir.push("experiments");
    dir
}

pub fn get_exp_genomes_dir(exp_key: &str) -> PathBuf {
    let mut dir = get_data_dir();
    dir.push("experiments");
    dir.push(exp_key);
    dir.push("genomes");
    dir
}

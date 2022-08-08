pub mod genetic_manifest;
pub mod sensor_manifest;

#[macro_use]
pub mod genome;

pub mod phenotype;

pub mod experiments;

use simulation::{Simulation};

pub type GenomeValue = u8;
pub mod genetic_manifest;
pub mod sensor_manifest;

#[macro_use]
pub mod genome;

pub mod unit_behavior;

pub mod experiments;

use crate::simulation::Simulation;

pub type GenomeValue = u8;

use std::{cell::Cell, rc::Rc};

use serde::Serialize;

use crate::{
    biology::{
        experiments::types::{CullStrategy, ExperimentGenomeUid, GenomeEntryId},
        genome::framed::common::{CompiledFramedGenome, RawFramedGenome},
    },
    simulation::{
        common::{builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod},
        fitness::FitnessScore,
        unit::{UnitAttributeValue, UnitResourceAmount},
    },
};

pub mod predicates;
use std::rc::Rc;

use self::predicates::*;

use crate::biology::genome::framed::types::FramedGenomeValue;
use crate::chemistry::actions::ActionSet;
use crate::simulation::common::{ChemistryManifest, SensorManifest};

/**
 * Contains the information that is required to interpret a genome.
 */
pub struct GeneticManifest {
    pub chemistry_manifest: Rc<ChemistryManifest>,
    pub sensor_manifest: Rc<SensorManifest>,
    pub operator_set: Rc<OperatorSet>,

    /**
     * question: what ultimately should determine this?  should this be defined by the chemistry?
     */
    pub number_of_registers: usize,
}

impl GeneticManifest {
    pub fn defaults(chemistry_manifest: &ChemistryManifest) -> Self {
        Self {
            chemistry_manifest: Rc::new(chemistry_manifest.clone()),
            sensor_manifest: Rc::new(SensorManifest::with_default_sensors(&chemistry_manifest)),
            operator_set: Rc::new(OperatorSet::default_operators()),
            number_of_registers: 3,
        }
    }

    pub fn operator_id_for_key(&self, s: &str) -> OperatorId {
        self.operator_set.by_key(s).index
    }
}

// #[derive(Clone)]
// pub struct GeneticManifest {
//     pub operator_set: OperatorSet,
//     pub number_of_registers: usize,
// }

// impl GeneticManifest {
//     pub fn new() -> GeneticManifest {
//         Self {
//             operator_set: default_operators(),
//             number_of_registers: 5,
//         }
//     }

//     pub fn operator_id_for_key(&self, s: &str) -> OperatorId {
//         self.operator_set.by_key(s).index
//     }
// }

// fn test() {
//     let manifest = GeneticManifest {
//         //action_set: ActionSet {},
//         operator_set: default_operators(),
//         number_of_registers: 5,
//     };
// }

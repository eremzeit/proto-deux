pub mod predicates;
use self::predicates::{*};

use chemistry::actions::{ActionSet};
use biology::genome::framed::types::{FramedGenomeValue};

#[derive(Clone)]
pub struct GeneticManifest {
  pub operator_set: OperatorSet,
  pub number_of_registers: usize,
}

impl GeneticManifest {
  pub fn new() -> GeneticManifest {
    Self {
        operator_set: default_operators(),
        number_of_registers: 5,
    }
  }

  pub fn operator_id_for_key(&self, s: &str) -> OperatorId {
    self.operator_set.by_key(s).index
  }
}

fn test() {
  let manifest = GeneticManifest {
    //action_set: ActionSet {},
    operator_set: default_operators(),
    number_of_registers: 5,
  };
}


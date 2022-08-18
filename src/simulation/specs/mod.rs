use crate::tests::GeneticManifest;

use super::common::{
    get_chemistry_by_key, helpers::place_units::PlaceUnitsMethod, ChemistryConfiguration,
    ChemistryInstance, ChemistryManifest, SensorManifest,
};

/**
 * Defines behavioral configuraton for simulation.  Contains everything
 * needed to construct a chemistry instance.
 */
#[derive(Default, Clone)]
pub struct SimulationSpecs {
    pub chemistry_key: String,
    pub chemistry_configuration: ChemistryConfiguration,
    pub place_units_method: PlaceUnitsMethod,
}

/**
 * note: For a genome to execute correctly, the simulation specs need to be
 * exactly the same across executions.
 */
impl SimulationSpecs {
    pub fn construct_chemistry(&self) -> ChemistryInstance {
        get_chemistry_by_key(
            &self.chemistry_key,
            self.place_units_method.clone(),
            self.chemistry_configuration.clone(),
        )
    }

    pub fn chemistry_manifest(&self) -> ChemistryManifest {
        self.construct_chemistry().get_manifest().clone()
    }

    pub fn sensors(&self) -> SensorManifest {
        SensorManifest::with_default_sensors(&self.chemistry_manifest())
    }

    pub fn genetic_manifest(&self) -> GeneticManifest {
        GeneticManifest::new()
    }

    pub fn context(&self) -> (ChemistryManifest, SensorManifest, GeneticManifest) {
        (
            self.chemistry_manifest(),
            self.sensors(),
            self.genetic_manifest(),
        )
    }
}

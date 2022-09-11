pub mod predicates;
use std::rc::Rc;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::biology::genome::framed::types::FramedGenomeValue;
use crate::chemistry::actions::ActionManifest;
use crate::simulation::common::serialize::ChemistryManifestData;
use crate::simulation::common::{
    ActionDefinition, ActionLibrary, Chemistry, ChemistryConfiguration, ChemistryInstance,
    ChemistryManifest,
};

use self::predicates::{OperatorId, OperatorLibrary, OperatorManifest, OperatorManifestData};

use super::sensor_manifest::{
    CustomSensorImplementation, CustomSensorLibrary, LocalPropertySensorManifest, SensorManifest,
    SensorManifestData,
};
use super::unit_behavior::framed::PhenotypeRegisterValue;

/**
 * Used when actually executing genomes
 *
 * This might be broken up into multiple objects... ie. those that change when the ChemistryConfig changes and those that dont
 * Because the genetic manifest is used to compile genomes which is seperate from chemistry config.
 */
#[derive(Clone)]
pub struct GeneticManifest {
    pub chemistry_manifest: Arc<ChemistryManifest>,

    pub sensor_manifest: Arc<SensorManifest>,

    pub operator_manifest: Arc<OperatorManifest>,

    /**
     * question: what ultimately should determine this?  should this be defined by the chemistry?
     */
    pub number_of_registers: usize,
}

impl GeneticManifest {
    pub fn from_chemistry(chemistry: &ChemistryInstance) -> Self {
        let cm = chemistry.get_manifest();
        Self::new(
            cm.clone(),
            chemistry.default_local_property_sensor_manifest(),
        )
    }

    pub fn from_default_chemistry_config<C: Chemistry>() -> Self {
        let config = C::default_config();
        Self::construct::<C>(&config)
    }

    pub fn construct<C: Chemistry>(chemistry_config: &ChemistryConfiguration) -> Self {
        let cm = C::construct_manifest(&chemistry_config);

        let local_property_sensor_manifest =
            C::get_default_local_property_sensor_manifest(&cm.all_properties);
        // LocalPropertySensorManifest::from_all_props(&cm.all_properties);

        Self::new(cm, local_property_sensor_manifest)
    }

    pub fn new(
        chemistry_manifest: ChemistryManifest,
        local_property_sensors: LocalPropertySensorManifest,
    ) -> Self {
        Self {
            sensor_manifest: Arc::new(SensorManifest::new(
                &chemistry_manifest,
                &local_property_sensors,
            )),
            chemistry_manifest: Arc::new(chemistry_manifest),
            operator_manifest: Arc::new(OperatorManifest::default_operators()),
            number_of_registers: 5,
        }
    }

    pub fn wrap_rc(self) -> Rc<Self> {
        Rc::new(self)
    }

    pub fn operator_id_for_key(&self, s: &str) -> OperatorId {
        self.operator_manifest.by_key(s).index
    }

    pub fn empty_registers(&self) -> Vec<PhenotypeRegisterValue> {
        let mut v = Vec::new();
        v.resize(self.number_of_registers, 0);
        v
    }
}

/**
 * Contains the information that is required to interpret a genome.
 *
 * Question: Is this constant per each chemistry type or does it change across unit entries?
 *
 *
 */
// #[derive(Clone, Serialize, Deserialize)]
#[derive(Clone)]
pub struct GeneticManifestData {
    pub chemistry_manifest: ChemistryManifestData,

    pub sensor_manifest: SensorManifestData,

    pub operator_manifest: OperatorManifestData,

    /**
     * question: what ultimately should determine this?  should this be defined by the chemistry?
     */
    pub number_of_registers: usize,
}

impl GeneticManifestData {
    // pub fn to_compiled(
    //     &self,
    //     action_library: ActionLibrary,
    //     custom_sensor_library: CustomSensorLibrary,
    //     operator_library: OperatorLibrary,
    // ) -> GeneticManifest {
    //     GeneticManifest {
    //         chemistry_manifest: Rc::new(
    //             self.chemistry_manifest.to_compiled(action_library.clone()),
    //         ),
    //         sensor_manifest: Rc::new(
    //             self.sensor_manifest
    //                 .to_compiled_sensor_manifest(custom_sensor_library),
    //         ),
    //         operator_manifest: Rc::new(self.operator_manifest.to_compiled(operator_library)),
    //         number_of_registers: self.number_of_registers,
    //     }
    // }
}

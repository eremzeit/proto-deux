use std::rc::Rc;

use crate::biology::genetic_manifest::predicates::OperatorManifest;

use super::common::{
    helpers::place_units::PlaceUnitsMethod, Chemistry, ChemistryConfiguration, ChemistryInstance,
    ChemistryManifest, SensorManifest,
};

// /**
//  * Contains everything needed to construct a chemistry instance.
//  */
// #[derive(Clone)]
// pub struct ChemistryOptions {
//     pub chemistry_key: String,
//     pub chemistry_configuration: Option<ChemistryConfiguration>,
// }

// /**
//  * note: For a genome to execute correctly, the simulation specs need to be
//  * exactly the same across executions.
//  */
// impl ChemistryOptions {
//     pub fn construct(&self) -> ChemistryInstance {
//         get_chemistry_by_key(
//             &self.chemistry_key,
//             // self.place_units_method.clone(),
//             self.chemistry_configuration.clone(),
//         )
//     }

//     // pub fn chemistry_manifest(&self) -> ChemistryManifest {
//     //     self.construct_chemistry().get_manifest().clone()
//     // }

//     // pub fn sensors(&self) -> SensorManifest {
//     //     SensorManifest::with_default_sensors(&self.chemistry_manifest())
//     // }

//     // pub fn genetic_manifest(&self) -> GeneticManifest {
//     //     GeneticManifest::new()
//     // }

//     // // pub fn context(&self) -> (ChemistryManifest, SensorManifest, GeneticManifest) {
//     // pub fn context(&self) -> Rc<BehaviorManifest> {
//     //     Rc::new()(
//     //         self.chemistry_manifest(),
//     //         self.sensors(),
//     //         self.genetic_manifest(),
//     //     )
//     // }
// }

// #[derive(Clone)]

// // pub mod builder {
// //     use std::rc::Rc;

// //     use crate::{
// //         biology::genetic_manifest::predicates::OperatorSet,
// //         simulation::common::{
// //             get_chemistry_by_key, helpers::place_units::PlaceUnitsMethod, Chemistry,
// //             ChemistryConfiguration, ChemistryInstance, SensorManifest,
// //         },
// //     };

// //     use super::BehaviorManifest;

// //     #[derive(Builder)]
// //     #[builder(pattern = "owned", setter(strip_option), build_fn(skip))]
// //     pub struct SimulationSpecs {
// //         pub chemistry_key: String,
// //         pub chemistry: Rc<dyn Chemistry>,

// //         #[builder(default)]
// //         pub chemistry_configuration: ChemistryConfiguration,

// //         #[builder(default)]
// //         pub place_units_method: PlaceUnitsMethod,

// //         operator_set: OperatorSet,
// //     }

// //     impl SimulationSpecsBuilder {
// //         pub fn build(mut self) -> super::SimulationSpecs {
// //             if self.chemistry.is_none() {
// //                 self.chemistry = Some(get_chemistry_by_key(
// //                     &self.chemistry_key.unwrap(),
// //                     self.place_units_method.unwrap_or_default(),
// //                     self.chemistry_configuration.unwrap_or_default(),
// //                 ));
// //             }

// //             let chemistry = self.chemistry.unwrap();

// //             super::SimulationSpecs {
// //                 chemistry,
// //                 place_units_method: self.place_units_method.unwrap(),
// //                 // behavior_manifest: todo!(),
// //             }
// //         }
// //     }
// // }

// pub struct GeneticManifest {
//     chemistry_manifest: Rc<ChemistryManifest>,
//     sensor_manifest: Rc<SensorManifest>,
//     operator_set: Rc<OperatorSet>,
//     number_of_registers: usize,
// }

// BehaviorManifest {
//     chemistry_manifest: Rc::new(chemistry.get_manifest().clone()),
//     sensor_manifest: SensorManifest::with_default_sensors(chemistry.get_manifest()),
//     operator_set: self.operator_set.unwrap_or(default_operators()),
//     number_of_registers: self.,
// }

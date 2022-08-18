use crate::chemistry::*;
use crate::simulation;
use crate::simulation::common::{
    get_chemistry_by_key, EmptyPhenotype, Phenotype, SimulationControlEvent,
    SimulationControlEventReceiver, UnitAttributeValue, UnitEntry, UnitEntryBuilder, UnitEntryData,
    UnitManifest, UnitResourceAmount,
};
use crate::simulation::fitness::*;
use crate::simulation::unit::util::convert_maybe_resources_to_resources;
use crate::simulation::unit::{UnitAttributes, UnitResources};
use crate::simulation::Simulation;
use crate::util::GridSize2D;
use std::rc::Rc;
use std::sync::Arc;

pub mod builder {
    use crate::simulation::{
        common::helpers::place_units::PlaceUnitsMethod, specs::SimulationSpecs,
    };

    use super::*;
    #[derive(Builder)]
    #[builder(pattern = "owned", setter(strip_option))]
    pub struct Simulation {
        pub size: GridSize2D,
        pub unit_entries: Vec<UnitEntryBuilder>,
        pub unit_manifest: UnitManifest,
        pub iterations: u64,
        pub specs: SimulationSpecs,
        pub chemistry_key: String,
        // pub chemistry_configuration: ChemistryConfiguration,
        // pub place_units_method: PlaceUnitsMethod,
        // pub chemistry: ChemistryInstance,
        pub headless: bool,
    }

    impl SimulationBuilder {
        // fn _init_chemistry(&mut self) -> ChemistryInstance {
        //     match &self.chemistry {
        //         None => {
        //             let specs = self.specs.unwrap_or(SimulationSpecs {
        //                 chemistry_key: self
        //                     .chemistry_key
        //                     .expect("Must either give specs object or chemistry key"),
        //                 ..Default::default()
        //             });
        //             self.chemistry = Some(specs.construct_chemistry());
        //         }
        //     }
        // }

        pub fn to_simulation(mut self) -> simulation::Simulation {
            let specs = self.specs.unwrap_or_else(|| SimulationSpecs {
                chemistry_key: self
                    .chemistry_key
                    .expect("Must either give specs object or chemistry key"),
                ..Default::default()
            });

            let chemistry = specs.construct_chemistry();

            let size = self.size.unwrap();

            let chemistry_manifest = specs.chemistry_manifest();

            /*
             * INIT UNIT MANIFEST
             */
            if self.unit_manifest.is_none() && self.unit_entries.is_some() {
                let mut unit_entries: Vec<_> = vec![];
                let mut _builders = None;
                std::mem::swap(&mut self.unit_entries, &mut _builders);
                let mut builders = _builders.unwrap();

                while builders.len() > 0 {
                    let builder = builders.remove(0);
                    unit_entries.push(builder.build(&chemistry_manifest, None));
                }
                self.unit_manifest = Some(UnitManifest {
                    units: unit_entries,
                });
            }

            if self.unit_manifest.is_none() {
                self.unit_manifest = Some(UnitManifest {
                    units: vec![UnitEntry::new("default", EmptyPhenotype::construct())],
                });
            }

            // let chemistry = self.chemistry.as_mut().unwrap();

            let iterations = match self.iterations {
                Some(i) => i,
                None => 100,
            };

            let mut unit_manifest = std::mem::replace(&mut self.unit_manifest, None);
            // let mut chemistry = std::mem::replace(&mut self.chemistry, None);

            let mut sim =
                simulation::Simulation::new(chemistry, size, iterations, unit_manifest.unwrap());

            sim
        }
    }
}

pub use super::builder::SimulationBuilder;

#[derive(Clone)]
pub struct SimulationConfig {
    pub size: GridSize2D,
    pub unit_manifest: UnitManifest,
    pub iterations: u64,
}

impl SimulationConfig {}

#[derive(Clone)]
pub struct SimulationConfigData {
    pub size: GridSize2D,
    pub unit_manifest: Vec<UnitEntryData>,
    pub iterations: u64,
    pub chemistry_key: String,
}

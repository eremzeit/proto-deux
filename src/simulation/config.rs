use chemistry::*;
use simulation;
use simulation::common::{
    get_chemistry_by_key, EmptyPhenotype, Phenotype, PlaceUnitsMethod, SimulationControlEvent,
    SimulationControlEventReceiver, SimulationEvent, SimulationEventSender, SimulationSpec,
    UnitAttributeValue, UnitEntry, UnitEntryBuilder, UnitEntryData, UnitManifest,
    UnitResourceAmount,
};
use simulation::fitness::*;
use simulation::unit::util::convert_maybe_resources_to_resources;
use simulation::unit::{UnitAttributes, UnitResources};
use simulation::Simulation;
use std::rc::Rc;
use std::sync::Arc;
use util::GridSize2D;

pub mod builder {
    use super::*;
    #[derive(Builder)]
    #[builder(pattern = "owned", setter(strip_option))]
    pub struct Simulation {
        pub size: GridSize2D,
        pub unit_entries: Vec<UnitEntryBuilder>,
        pub unit_manifest: UnitManifest,
        pub iterations: u64,
        pub chemistry_key: String,
        pub chemistry: ChemistryInstance,
        pub specs: Vec<Box<dyn SimulationSpec>>,
        pub sim_events: SimulationEventSender,
        pub control_events: SimulationControlEventReceiver,
        pub headless: bool,
        pub unit_placement: PlaceUnitsMethod,
    }

    impl SimulationBuilder {
        fn _init_chemistry(&mut self) {
            match &self.chemistry {
                Some(chemistry) => {
                    //println!("here: {}", chemistry.get_key());
                    //chemistry
                }
                None => {
                    self.chemistry = Some(get_chemistry_by_key(
                        &self.chemistry_key.as_ref().unwrap_or(&"base".to_string()),
                    ));
                }
            }
        }

        pub fn chemistry_specs(
            &mut self,
            unit_placement: PlaceUnitsMethod,
        ) -> Vec<Box<dyn SimulationSpec>> {
            self._init_chemistry();
            let chemistry = self.chemistry.as_mut().unwrap();
            chemistry.construct_specs(self.unit_placement.as_mut().unwrap())
        }

        pub fn to_simulation(mut self) -> simulation::Simulation {
            let size = self.size.unwrap();

            /*
             * INIT CHEMISTRY
             */
            self._init_chemistry();
            if self.chemistry.is_none() {
                self.chemistry = Some(get_chemistry_by_key(
                    self.chemistry_key.as_ref().unwrap_or(&"base".to_string()),
                ));
            }
            let chemistry_manifest = self.chemistry.as_ref().unwrap().get_manifest();

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

            /*
             * INIT SPECS
             */
            let mut _specs: Option<Vec<Box<dyn SimulationSpec>>> = None;
            std::mem::swap(&mut self.specs, &mut _specs);

            let specs: Vec<Box<dyn SimulationSpec>> = match _specs {
                Some(s) => s,
                None => self.chemistry_specs(self.unit_placement.as_ref().unwrap().clone()),
            };

            let iterations = match self.iterations {
                Some(i) => i,
                None => 100,
            };
            let sim_events = match self.sim_events {
                Some(x) => x,
                None => {
                    if !self.headless.unwrap_or(false) {
                        panic!("An event channel is required if the simulation isn't headless");
                    }
                    let (sender, receiver) = std::sync::mpsc::channel::<SimulationEvent>();
                    sender
                }
            };
            let control_events = match self.control_events {
                Some(x) => x,
                None => {
                    if !self.headless.unwrap_or(false) {
                        panic!("An event channel is required if the simulation isn't headless");
                    }
                    let (sender, receiver) = std::sync::mpsc::channel::<SimulationControlEvent>();
                    receiver
                }
            };

            let mut unit_manifest = std::mem::replace(&mut self.unit_manifest, None);
            let mut chemistry = std::mem::replace(&mut self.chemistry, None);

            let mut sim = simulation::Simulation::new(
                chemistry.unwrap(),
                size,
                iterations,
                unit_manifest.unwrap(),
                specs,
                &sim_events,
                control_events,
            );

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

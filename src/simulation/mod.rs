pub mod common;
pub mod config;
pub mod coord_set;
pub mod executors;
pub mod fitness;
pub mod iterators;
pub mod position;
pub mod simulation_data;
pub mod specs;
pub mod text_grid;
pub mod unit;
pub mod unit_entry;
pub mod world;

use ndarray::*;
use ndarray::{Array, Array2, Dim, Ix, Shape};

use self::common::helpers::place_units::{self, PlaceUnitsMethod};
use self::config::SimulationConfigData;
use self::config::*;
use self::iterators::CoordIterator;
use self::position::*;
use self::simulation_data::{SimulationData, ThreadedSimulationReference};
use self::unit::*;
use self::unit_entry::{UnitEntry, UnitEntryData, UnitManifest};
use self::unit_entry::{UnitEntryAttributes, UnitEntryId};
use self::world::*;
use crate::chemistry::properties::UnitEntryAttributeDefinition;
use crate::chemistry::properties::{AttributeIndex, AttributeValue, ResourceAmount, ResourceIndex};
use crate::chemistry::variants::cheese::CheeseChemistry;
use crate::chemistry::{Chemistry, ChemistryInstance, ChemistryManifest};
use crate::perf::{perf_timer_start, perf_timer_stop};
use crate::util::{Coord, GridSize2D};

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::SendError;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub type PhenotypeId = usize;
pub type SimulationAttributeValue = AttributeValue;
pub type SimulationAttributes = Vec<SimulationAttributeValue>;
pub type SimulationAttributeIndex = AttributeIndex;
pub type SimulationResourceIndex = AttributeIndex;
pub type SimulationResourceAmount = ResourceAmount;

pub struct Simulation {
    pub world: World,
    pub chemistry: ChemistryInstance,
    pub attributes: Vec<SimulationAttributeValue>,
    pub unit_manifest: UnitManifest,
    pub unit_entry_attributes: Vec<UnitEntryAttributes>,
    pub iterations: u64,

    pub place_units_method: PlaceUnitsMethod,
    // pub control_events: Option<SimulationControlEventReceiver>,
}

// pub enum SimulationUiEvent {
//     SimulationEvent(SimulationEvent),
//     Nil,
// }

// #[derive(Debug)]
// pub enum SimulationEvent {
//     // UnitAttributeUpdated(UnitAttributeIndex, UnitAttributeValue),
//     // PositionAttributeUpdated(PositionAttributeIndex, PositionAttributeValue),
//     // SimulationAttributeUpdated(SimulationAttributeIndex, SimulationAttributeValue),
//     //
//     // UnitResourceUpdated(UnitResourceIndex, UnitResourceAmount),
//     // PositionResourceUpdated(PositionResourceIndex, PositionResourceAmount),
//     UnitEntryDescriptionUpdated(UnitEntryId, String, String),
//     PositionUpdated(Coord),
//     Nil,
// }
#[derive(Debug)]
pub enum SimulationControlEvent {
    Pause,
    Resume,
    Start,
    Halt,
}

// pub type SimulationEventSender = Sender<SimulationEvent>;
//pub type SimulationEventSender = Receiver<SimulationEvent>;
pub type SimulationControlEventSender = Sender<SimulationControlEvent>;
pub type SimulationControlEventReceiver = Receiver<SimulationControlEvent>;

#[macro_export]
macro_rules! sim_log {
    ($($x:expr),*) => {
        // println!($(
        //     $x,
        // )*);
    };
}

impl Simulation {
    pub fn new(
        mut chemistry: ChemistryInstance,
        size: GridSize2D,
        iterations: u64,
        mut unit_manifest: UnitManifest,
        place_units_method: PlaceUnitsMethod,
    ) -> Simulation {
        let world = World::new(size, &chemistry);
        unit_manifest.init_manifest();

        let attributes = chemistry.get_default_simulation_attributes();

        let unit_entry_attributes = unit_manifest
            .units
            .iter()
            .map(|entry| {
                if let Some(attr) = &entry.info.default_entry_attributes {
                    attr.clone()
                } else {
                    chemistry.get_default_unit_entry_attributes()
                }
            })
            .collect::<Vec<_>>();

        let mut simulation = Simulation {
            world,
            chemistry,
            iterations,
            unit_manifest,
            attributes,
            unit_entry_attributes,
            place_units_method,
        };

        simulation.init();

        simulation
    }

    pub fn init(&mut self) {
        self.world.tick = 1; // we start at one.  zero means pre-initialized.
        sim_log!("INIT SIMULATION: {}", self.chemistry.get_key());

        let entries = self
            .unit_manifest
            .units
            .iter()
            .map(|x| &x.info)
            .collect::<Vec<_>>();
        sim_log!("UNIT ENTRIES: {:?}", &entries);

        self.chemistry.on_simulation_init(&mut SimCell {
            attributes: &mut self.attributes,
            world: &mut self.world,
            unit_entry_attributes: &mut self.unit_entry_attributes,
            unit_manifest: &self.unit_manifest,
            chemistry: &self.chemistry,
        });

        self.place_units();
    }

    pub fn place_units(&mut self) {
        let method = match &self.place_units_method {
            PlaceUnitsMethod::Default => self.chemistry.get_default_place_units_method(),
            _ => self.place_units_method.clone(),
        };

        PlaceUnitsMethod::place_units(&mut self.editable(), &method);
    }

    pub fn _start(&mut self) {
        while self.world.tick < self.iterations {
            self.tick();
        }
    }

    pub fn editable(&mut self) -> SimCell {
        SimCell {
            attributes: &mut self.attributes,
            world: &mut self.world,
            unit_entry_attributes: &mut self.unit_entry_attributes,
            unit_manifest: &self.unit_manifest,
            chemistry: &self.chemistry,
        }
    }

    // pub fn sim_cell(&mut self) -> SimCell {
    //     SimCell {
    //         attributes: &mut self.attributes,
    //         world: &mut self.world,
    //         unit_entry_attributes: &mut self.unit_entry_attributes,
    //         unit_manifest: &self.unit_manifest,
    //         chemistry: &self.chemistry,
    //     }
    // }

    pub fn finish(&mut self) {
        self.chemistry.on_simulation_finish(&mut SimCell {
            attributes: &mut self.attributes,
            world: &mut self.world,
            unit_entry_attributes: &mut self.unit_entry_attributes,
            unit_manifest: &self.unit_manifest,
            chemistry: &self.chemistry,
        });
    }

    /**
     * Returns true if finished
     */
    pub fn tick(&mut self) -> bool {
        // if self.world.tick % 1000 == 0 {
        //     println!("tick: {}", self.world.tick);
        // }

        // println!("TICK");

        perf_timer_start!("sim_tick");
        self.chemistry.on_simulation_tick(&mut SimCell {
            attributes: &mut self.attributes,
            world: &mut self.world,
            unit_entry_attributes: &mut self.unit_entry_attributes,
            unit_manifest: &self.unit_manifest,
            chemistry: &self.chemistry,
        });
        perf_timer_stop!("sim_tick");

        self.world.tick = self.world.tick + 1;

        // if self.world.tick < self.iterations {
        //     for (i, spec) in self.specs.iter_mut().enumerate() {
        //         //println!("spec tick: {}", spec.get_name());
        //         let before = Instant::now();

        //         spec.on_tick(
        //             &mut SimCell {
        //                 attributes: &mut self.attributes,
        //                 world: &mut self.world,
        //                 unit_entry_attributes: &mut self.unit_entry_attributes,
        //                 unit_manifest: &self.unit_manifest,
        //                 chemistry: &self.chemistry,
        //             },
        //             &SpecContext {},
        //         );

        //         let spec_ms = Instant::now().duration_since(before).as_micros();
        //         //println!("{} - {}microsecs", spec.get_name(), spec_ms);
        //         self._spec_timings[i] += spec_ms;
        //     }

        // let ms_since_perf_update = Instant::now()
        //     .duration_since(self._last_perf_update_time)
        //     .as_millis();
        // let total_ticks = (self.world.tick - self._last_perf_update_tick).max(1);

        // if ms_since_perf_update > 10000 {
        //     let averages = self
        //         ._spec_timings
        //         .iter()
        //         .map(|x| -> u128 { *x / total_ticks as u128 })
        //         .collect::<Vec<_>>();
        //     print_spec_time_averages(&averages, &self.specs);
        //     self._spec_timings = vec![0; self.specs.len()];

        //     self._last_perf_update_time = Instant::now();
        //     self._last_perf_update_tick = self.world.tick;
        // }

        // self.world.tick = self.world.tick + 1;
        // }

        // note: the zeroith tick means pre-initialized.  Our first real tick starts at 1
        // so our ending conditional is plus one (ie. >= the iteration count)
        if self.world.tick >= self.iterations {
            self.finish();
            true
        } else {
            false
        }
    }
    pub fn to_data(&self) -> SimulationData {
        SimulationData {
            grid: self.world.grid.clone(),
            config: self.to_config_data(),
            tick: self.world.tick,
        }
    }

    pub fn to_config_data(&self) -> SimulationConfigData {
        SimulationConfigData {
            size: self.world.size.clone(),
            unit_manifest: self
                .unit_manifest
                .units
                .iter()
                .map(|unit_entry: &UnitEntry| -> UnitEntryData { unit_entry.info.clone() })
                .collect::<Vec<UnitEntryData>>(),
            iterations: self.iterations,
            chemistry_key: self.chemistry.get_key(),
        }
    }

    pub fn unit_attr_id_by_key(&self, key: &'static str) -> UnitAttributeIndex {
        self.chemistry.get_manifest().unit_attribute_by_key(key).id
    }
    pub fn unit_resource_id_by_key(&self, key: &'static str) -> UnitResourceIndex {
        self.chemistry.get_manifest().unit_resource_by_key(key).id
    }
    pub fn pos_resource_id_by_key(&self, key: &'static str) -> PositionResourceIndex {
        if 0 == 0 {
            panic!("Position resources aren't implemented yet");
        }
        0
        //self.chemistry.get_manifest().position_resource_by_key(key).id
    }
    pub fn pos_attribute_id_by_key(&self, key: &'static str) -> PositionAttributeIndex {
        self.chemistry
            .get_manifest()
            .position_attribute_by_key(key)
            .id
    }
}

fn add_times(times: &Vec<u128>) -> u128 {
    let mut total: u128 = 0;
    for time in times.iter() {
        total += time;
    }

    total
}

// fn print_spec_time_averages(spec_times: &Vec<u128>, specs: &SimulationSpecs) {
//     let mut times = spec_times.iter().enumerate().collect::<Vec<_>>();
//     times.sort_by(|a, b| b.1.cmp(a.1));
//     let mut slowest = times.to_vec();
//     slowest.truncate(2);

//     let mut s = "".to_string();

//     for (i, time) in &slowest {
//         s = format!(
//             "{}, [{} - ({}us)]",
//             s,
//             specs.get(*i).unwrap().get_name(),
//             *time
//         );
//     }

//     println!("PERF: {}us | {}", add_times(spec_times), s);
// }

pub fn increment_simulation_attribute_integer(
    val: &SimulationAttributeValue,
    inc: i32,
) -> SimulationAttributeValue {
    SimulationAttributeValue::Integer(val.unwrap_integer() + inc)
}

pub struct SimCell<'a> {
    pub world: &'a mut World,
    pub attributes: &'a mut SimulationAttributes,
    pub unit_entry_attributes: &'a mut Vec<UnitEntryAttributes>,
    pub chemistry: &'a ChemistryInstance,
    pub unit_manifest: &'a UnitManifest,
}

mod tests {
    use crate::simulation::common::{
        builder::ChemistryBuilder, helpers::place_units::PlaceUnitsMethod,
    };

    use super::common::*;
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_attribute_initialization() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();
        let mut sim = SimulationBuilder::default()
            .chemistry(chemistry)
            .size((5, 5))
            .place_units_method(PlaceUnitsMethod::ManualSingleEntry {
                attributes: None,
                coords: vec![(1, 1)],
            })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();
        assert_eq!(sim.world.has_unit_at(&(1, 1)), true);

        let id = sim
            .chemistry
            .get_manifest()
            .unit_attribute_by_key("rolling_consumption")
            .id;

        match sim.world.get_unit_attribute_at(&(1, 1), id as usize) {
            UnitAttributeValue::Integer(b) => {
                assert_eq!(b, 0);
            }

            _ => {
                panic!("should be a number");
            }
        }
    }
}

// pub fn send_event(channel: &mut SimulationEventSender, event: SimulationEvent) {
//     //println!(">>> SENDING EVENT {:?}", event);
//     match &channel.send(event) {
//         Ok(result) => {}

//         Err(e) => {
//             let _e: &SendError<SimulationEvent> = e;
//             //println!("send error; {:?} {}", e, e);
//         }
//     }
// }

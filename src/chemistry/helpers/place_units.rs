use crate::chemistry::variants::CheeseChemistry;
use crate::simulation::common::{
    ChemistryInstance, Coord, GridSize2D, NullBehavior, SimCell, Simulation, SimulationAttributes,
    UnitEntry, UnitEntryData, UnitManifest,
};
use crate::simulation::config::SimulationConfig;
use crate::simulation::config::*;
use crate::simulation::unit::{UnitAttributeValue, UnitAttributes};
use crate::simulation::world::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use typemap::{CloneMap, Key};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum PlaceUnitsMethod {
    LinearBottomMiddle {
        attributes: Option<UnitAttributes>,
    },
    ManualSingleEntry {
        attributes: Option<UnitAttributes>,
        coords: Vec<Coord>,
    },
    SimpleDrop {
        attributes: Option<UnitAttributes>,
    },
    SimpleDropMultiple {
        attributes: Option<UnitAttributes>,
        units_per_entry: u32,
    },

    // StaticRegionRandomDrop {
    //     attributes: Option<UnitAttributes>,
    //     units_per_entry: u32,

    //     // (pct_start_x, pct_start_y, pct_end_x, pct_end_y)
    //     region_pct_rect: ((Coord, Coord), (Coord, Coord)),
    // },
    RandomPctRegionDrop {
        attributes: Option<UnitAttributes>,
        units_per_entry: u32,

        // (pct_start_x, pct_start_y, pct_end_x, pct_end_y)
        region_pct_rect: (f32, f32, f32, f32),
    },

    Skip,

    #[default]
    Default,

    Chemistry,
}

impl PlaceUnitsMethod {
    pub fn place_units(sim: &mut SimCell, method: &PlaceUnitsMethod) {
        place_units(sim, method);
    }
}

pub fn place_units(sim: &mut SimCell, method: &PlaceUnitsMethod) {
    let max_units_possible = sim.world.size.0 * sim.world.size.1;

    let mut total_units = 0;
    for entry in sim.unit_manifest.units.iter() {
        // for now we assume 1 unit dropped per entry, but it might not always be like this
        total_units += 1;
    }

    if total_units > max_units_possible {
        panic!(
            "World has {} cells so cant place {} units",
            max_units_possible, total_units
        );
    }

    match method {
        PlaceUnitsMethod::LinearBottomMiddle { attributes } => {
            place_linear_middle_bottom(
                &mut sim.world,
                attributes,
                sim.unit_manifest,
                sim.chemistry,
            );
        }

        PlaceUnitsMethod::RandomPctRegionDrop {
            attributes,
            region_pct_rect,
            units_per_entry,
        } => {
            place_pct_region(
                &mut sim.world,
                sim.chemistry,
                sim.unit_manifest,
                &attributes,
                *units_per_entry,
                &region_pct_rect,
            );
        }
        PlaceUnitsMethod::SimpleDropMultiple {
            attributes,
            units_per_entry,
        } => {
            let manifest = sim.unit_manifest.clone();
            for (i, unit) in manifest.units.iter().enumerate() {
                for j in 0..*units_per_entry {
                    let idx = i * *units_per_entry as usize + j as usize;
                    // fill from the left to right, bottom to top
                    let x = idx % sim.world.size.0;
                    let y = idx / sim.world.size.0;

                    place_manual(
                        &unit.info,
                        &vec![(x, y)],
                        &mut sim.world,
                        attributes,
                        sim.chemistry,
                    );
                }
            }
        }

        PlaceUnitsMethod::SimpleDrop { attributes } => {
            let manifest = sim.unit_manifest.clone();
            for (i, unit) in manifest.units.iter().enumerate() {
                // fill from the left to right, bottom to top
                let x = i % sim.world.size.0;
                let y = i / sim.world.size.0;

                place_manual(
                    &unit.info,
                    &vec![(x, y)],
                    &mut sim.world,
                    attributes,
                    sim.chemistry,
                );
            }
        }
        PlaceUnitsMethod::ManualSingleEntry { attributes, coords } => {
            let manifest = sim.unit_manifest.clone();
            assert_eq!(
                manifest.units.len(),
                1,
                "[PlaceUnits] Cant seed because the gm should have size 1"
            ); //this only supports 1 genome right now
            place_manual(
                &sim.unit_manifest.units[0].info,
                coords,
                &mut sim.world,
                attributes,
                sim.chemistry,
            );
        }

        PlaceUnitsMethod::Skip => {}
        PlaceUnitsMethod::Default => {
            panic!("Not meant to be called directly");
        }
        PlaceUnitsMethod::Chemistry => {
            sim.chemistry.custom_place_units(sim);
        }
    }
}

pub fn place_linear_middle_bottom(
    world: &mut World,
    attributes: &Option<UnitAttributes>,
    unit_manifest: &UnitManifest,
    chemistry: &ChemistryInstance,
) {
    let x_start = (world.size.0 - unit_manifest.units.len()) / 2;
    let x = x_start;

    let len = unit_manifest.units.len();
    for i in 0..len {
        world.seed_unit_at(
            &(x + i as usize, 0 as usize),
            &unit_manifest.units[i].info,
            attributes.clone(),
            chemistry.as_ref(),
        );
    }
}

pub fn place_pct_region(
    world: &mut World,
    chemistry: &ChemistryInstance,
    unit_manifest: &UnitManifest,
    attributes: &Option<UnitAttributes>,
    units_per_entry: u32,
    region_pct_rect: &(f32, f32, f32, f32),
) {
    let c = chemistry.as_ref();
    let manifest = unit_manifest.clone();
    let mut rng = rand::thread_rng();
    let mut attempts = 0;

    let rect = [
        (world.size.0 as f32 * region_pct_rect.0) as usize,
        (world.size.1 as f32 * region_pct_rect.1) as usize,
        (world.size.0 as f32 * region_pct_rect.2) as usize,
        (world.size.1 as f32 * region_pct_rect.3) as usize,
    ];
    // println!("[PlaceUnits] placing units in region: {:?}", rect);

    let max_attempts = manifest.units.len() * units_per_entry as usize * 5;

    for (i, unit_entry) in manifest.units.iter().enumerate() {
        for i in 0..units_per_entry {
            loop {
                let coord = (
                    rng.gen_range(rect[0]..rect[2]),
                    rng.gen_range(rect[1]..rect[3]),
                );
                let can_place = !world.has_unit_at(&coord);
                if can_place {
                    world.seed_unit_at(
                        &coord,
                        &unit_entry.info,
                        attributes.clone(),
                        chemistry.as_ref(),
                    );
                    break;
                } else {
                    attempts += 1;
                    if attempts > max_attempts {
                        panic!(
                            "Random unit placement failed too many times within rect: {:?}",
                            &rect
                        );
                    }
                }
            }
        }
    }
}

pub fn place_manual(
    unit_entry: &UnitEntryData,
    coords: &Vec<Coord>,
    world: &mut World,
    attributes: &Option<UnitAttributes>,
    chemistry: &ChemistryInstance,
) {
    // println!("[PlaceUnits] placing units at coords: {:?}", coords);
    for coord in coords {
        world.seed_unit_at(coord, unit_entry, attributes.clone(), chemistry.as_ref());
    }
}

mod tests {
    use crate::simulation::common::builder::ChemistryBuilder;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_place_units() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();

        let mut sim = SimulationBuilder::default()
            .size((5, 5))
            .chemistry(chemistry)
            .place_units_method(PlaceUnitsMethod::LinearBottomMiddle { attributes: None })
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", NullBehavior::construct())],
            })
            .to_simulation();
        assert_eq!(sim.world.has_unit_at(&(2, 0)), true);
        assert_eq!(sim.world.has_unit_at(&(3, 0)), false);
    }

    #[test]
    fn test_random_region_drop() {
        let chemistry = ChemistryBuilder::with_key("cheese").build();

        let mut sim = SimulationBuilder::default()
            .size((100, 100))
            .place_units_method(PlaceUnitsMethod::RandomPctRegionDrop {
                attributes: None,
                units_per_entry: 2,
                region_pct_rect: (0.25, 0.25, 0.75, 0.75),
            })
            .chemistry(chemistry)
            .unit_manifest(UnitManifest {
                units: vec![
                    UnitEntry::new("main", NullBehavior::construct()),
                    UnitEntry::new("main", NullBehavior::construct()),
                ],
            })
            .to_simulation();

        let mut unit_count = 0;
        for x in 0..100 {
            for y in 0..100 {
                if sim.world.has_unit_at(&(x, y)) {
                    unit_count += 1;
                    if x < 25 || x >= 75 || y < 25 || y >= 75 {
                        panic!("incorrect unit placement: {:?}", &(x, y));
                    }
                }
            }
        }

        if unit_count != 4 {
            panic!("incorrect number of units placed: {}", unit_count);
        }
    }
}

// #[derive(Clone)]
// pub struct OuterStruct {
//     trait_obj: Box<dyn MyTrait>,
// }

// pub trait MyTrait: Clone {}

// pub struct MyStruct {}

// impl MyTrait for MyStruct {}

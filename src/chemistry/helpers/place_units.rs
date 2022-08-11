use crate::chemistry::variants::{BaseChemistry, CheeseChemistry};
use crate::simulation::common::{
    ChemistryInstance, Coord, EmptyPhenotype, GridSize2D, SimCell, Simulation,
    SimulationAttributes, UnitEntry, UnitEntryData, UnitManifest,
};
use crate::simulation::config::SimulationConfig;
use crate::simulation::config::*;
use crate::simulation::unit::{UnitAttributeValue, UnitAttributes};
use crate::simulation::world::*;
use std::sync::Arc;
use typemap::{CloneMap, Key};

// pub trait CustomUnitPlacement: Clone {
//     // fn place_units(&self, sim: &mut SimCell);
//     // fn place_units(&self);
// }

#[derive(Clone)]
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
    Skip,
    Default,
}

pub fn place_units(sim: &mut SimCell, method: &PlaceUnitsMethod) {
    match method {
        PlaceUnitsMethod::LinearBottomMiddle { attributes } => {
            place_linear_middle_bottom(
                &mut sim.world,
                attributes,
                sim.unit_manifest,
                sim.chemistry,
            );
        }

        PlaceUnitsMethod::SimpleDrop { attributes } => {
            let manifest = sim.unit_manifest.clone();
            for (i, unit) in manifest.units.iter().enumerate() {
                // println!(
                //     "PLACING UNIT species_name: {} ({})",
                //     &unit.info.species_name, unit.info.id
                // );
                place_manual(
                    &unit.info,
                    &vec![(i, 0)],
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
        PlaceUnitsMethod::Default => {}
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
            &chemistry,
        );
    }
}

pub fn place_manual(
    unit_entry: &UnitEntryData,
    coords: &Vec<Coord>,
    world: &mut World,
    attributes: &Option<UnitAttributes>,
    chemistry: &ChemistryInstance,
) {
    println!("[PlaceUnits] placing units at coords: {:?}", coords);
    for coord in coords {
        world.seed_unit_at(coord, unit_entry, attributes.clone(), &chemistry);
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_place_units() {
        let mut sim = SimulationBuilder::default()
            .size((5, 5))
            .chemistry(CheeseChemistry::construct(
                PlaceUnitsMethod::LinearBottomMiddle { attributes: None },
            ))
            .headless(true)
            .unit_manifest(UnitManifest {
                units: vec![UnitEntry::new("main", EmptyPhenotype::construct())],
            })
            .to_simulation();
        assert_eq!(sim.world.has_unit_at(&(2, 0)), true);
        assert_eq!(sim.world.has_unit_at(&(3, 0)), false);
    }
}

// #[derive(Clone)]
// pub struct OuterStruct {
//     trait_obj: Box<dyn MyTrait>,
// }

// pub trait MyTrait: Clone {}

// pub struct MyStruct {}

// impl MyTrait for MyStruct {}
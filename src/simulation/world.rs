use std::sync::mpsc::{channel, Receiver, Sender};

use crate::chemistry::{Chemistry, ChemistryInstance};
use crate::simulation::common::*;
use crate::util::{coord_by_coord_offset, coord_by_direction_offset, Coord, GridDirection};
use ndarray::*;
use ndarray::{Array, Array2, Dim, Ix, Shape};

pub type Grid = Array2<Option<Position>>;

#[derive(Clone)]
pub struct World {
    pub grid: Grid,
    pub size: GridSize2D,
    pub last_unit_id: UnitId,
    pub tick: u64,
    pub _unit_count: u64,
}

impl World {
    pub fn new(size: GridSize2D, chemistry: &ChemistryInstance) -> World {
        let mut grid: Grid = Array2::from_elem((size.0, size.1).f(), None);
        for x in 0..size.0 {
            for y in 0..size.1 {
                grid[[x, y]] = Some(empty_position((x, y), chemistry.get_manifest()));
            }
        }

        World {
            grid,
            size,
            last_unit_id: 0,
            tick: 0,
            _unit_count: 0,
        }
    }

    pub fn set_unit_last_update_tick(&mut self, coord: &Coord, last_update_tick: u64) {
        //let mut unit = self.get_unit_at(coord).unwrap();
        //unit.set_last_update_tick(last_update_tick);

        let mut pos = self
            .grid
            .get_mut([coord.0, coord.1])
            .unwrap()
            .as_mut()
            .unwrap();
        if let Some(unit) = &mut pos.unit {
            unit.set_last_update_tick(last_update_tick);
        }
    }

    pub fn destroy_unit(&mut self, coord: &Coord) {
        if self.has_unit_at(coord) {
            self._unit_count -= 1;
        }

        self.set_unit_at(coord, None);
    }

    pub fn move_unit(
        &mut self,
        src_coord: &Coord,
        dest_coord: &Coord,
        chemistry: &ChemistryInstance,
    ) {
        let src_unit = self.get_unit_at(src_coord).unwrap().clone(); // TODO PERF
        println!("moving unit {:?}, {:?}", dest_coord, src_coord);
        self.set_unit_at(dest_coord, Some(src_unit));
        self.set_unit_at(src_coord, None);
    }

    pub fn copy_unit_with_attributes(
        &mut self,
        src_coord: &Coord,
        dest_coord: &Coord,
        unit_manifest: &UnitManifest,
        chemistry: &ChemistryInstance,
    ) {
        let manifest = chemistry.get_manifest();
        let src_unit = self.get_unit_at(src_coord).unwrap();
        let unit_entry = &unit_manifest.units[src_unit.entry_id];

        let mut resources: UnitResources =
            chemistry.get_unit_seed_stored_resource_amounts(self, dest_coord, &unit_entry.info);
        let mut attributes: UnitAttributes =
            chemistry.get_unit_seed_attributes(self, dest_coord, &unit_entry.info);

        // maybe eventually the chemistry can define a list of attributes that are copied by
        // default from the src unit
        self.seed_unit_at(dest_coord, &unit_entry.info, None, &chemistry);
    }

    pub fn seed_unit_at(
        &mut self,
        coord: &Coord,
        unit_entry: &UnitEntryData,
        _attributes: Option<UnitAttributes>,
        chemistry: &ChemistryInstance,
    ) {
        let manifest = chemistry.get_manifest();
        let mut attributes: UnitAttributes =
            chemistry.get_unit_seed_attributes(self, coord, unit_entry);
        let mut resources: UnitResources =
            chemistry.get_unit_seed_stored_resource_amounts(self, coord, unit_entry);
        //println!("SEED_UNIT_AT resources: {:?}", resources);

        if _attributes.is_some() {
            merge_unit_attributes(&mut attributes, &_attributes.unwrap());
        }

        let entry = unit_entry.clone();
        if entry.default_attributes.is_some() {
            merge_unit_attributes(&mut attributes, entry.default_attributes.as_ref().unwrap());
        }

        if entry.default_resources.is_some() {
            resources = entry.default_resources.as_ref().unwrap().clone();
        }

        // println!(
        //     "[world.seed_unit_at] seeding unit at {:?} with resources {:?}",
        //     &coord, resources
        // );

        self.last_unit_id = self.last_unit_id + 1;

        let unit = Unit {
            resources,
            attributes,
            entry_id: unit_entry.id,
            id: self.last_unit_id,
            coord: coord.clone(),
            last_update_tick: 0,
        };

        self._unit_count += 1;
        self.set_unit_at(coord, Some(unit));
    }

    pub fn get_position_at(&self, coord: &Coord) -> Option<&Position> {
        assert_coords_valid_for_world!(coord, self);
        let maybe_pos = self.grid.get([coord.0, coord.1]).unwrap();
        maybe_pos.as_ref()
    }

    pub fn get_unit_at(&self, coord: &Coord) -> Option<&Unit> {
        let maybe_pos = self.grid.get([coord.0, coord.1]).unwrap();
        if let Some(pos) = maybe_pos {
            if let Some(unit) = &pos.unit {
                return Some(unit);
            }
        }

        return None;
    }

    pub fn has_unit_at(&self, coord: &Coord) -> bool {
        let maybe_pos = self.grid.get([coord.0, coord.1]).unwrap();

        if let Some(pos) = maybe_pos {
            return pos.unit.is_some();
        }

        return false;
    }

    /*
     *  Sets a unit in the position at that coordinate.
     */
    pub fn set_unit_at(&mut self, coord: &Coord, unit: Option<Unit>) {
        let pos = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(_pos) = pos {
            match &unit {
                Some(unit) => {
                    //self.unit_cache.insert(coord);
                }
                None => {
                    //self.unit_cache.remove(coord);
                }
            }

            _pos.set_unit(unit);
        }
    }

    pub fn get_unit_attribute_at(
        &self,
        coord: &Coord,

        attr_idx: UnitAttributeIndex,
    ) -> UnitAttributeValue {
        let mut item = self.grid.get([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            return pos.get_unit_attribute(attr_idx);
        }
        panic!["Position does not exist at {:?}", coord];
    }

    pub fn set_unit_attribute_at(
        &mut self,
        coord: &Coord,
        attr_idx: UnitAttributeIndex,
        value: UnitAttributeValue,
    ) {
        assert_coords_valid_for_world!(coord, self);

        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            pos.set_unit_attribute(attr_idx, value);
        }
    }

    pub fn get_unit_resource_at(
        &self,
        coord: &Coord,
        resource_idx: UnitResourceIndex,
    ) -> UnitResourceAmount {
        assert_coords_valid_for_world!(coord, self);

        let mut item = self.grid.get([coord.0, coord.1]).unwrap();
        if let Some(pos) = item {
            return pos.get_unit_resource(resource_idx);
        }
        panic!["Position does not exist at {:?}", coord];
    }

    pub fn set_unit_resource_at(
        &mut self,
        coord: &Coord,
        resource_idx: UnitResourceIndex,
        amount: UnitResourceAmount,
    ) {
        assert_coords_valid_for_world!(coord, self);

        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();
        if let Some(pos) = item {
            pos.set_unit_resource(resource_idx, amount);
        }
    }

    pub fn set_some_unit_resources_at(
        &mut self,
        coord: &Coord,
        unit_resources: &SomeUnitResources,
        chemistry: &ChemistryInstance,
    ) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            pos.set_some_unit_resources(unit_resources);
        }
    }

    pub fn set_unit_resources_at(&mut self, coord: &Coord, unit_resources: UnitResources) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();
        if let Some(pos) = item {
            pos.set_unit_resources(unit_resources);
        }
    }

    pub fn add_unit_resource_at(
        &mut self,
        coord: &Coord,
        resource_idx: UnitResourceIndex,
        amount: UnitResourceAmount,
    ) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();
        if let Some(pos) = item {
            pos.add_unit_resource(resource_idx, amount);
        }
    }

    pub fn set_pos_resource_tab_offset(
        &mut self,
        coord: &Coord,
        resource_idx: PositionResourceIndex,
        offset: i32,
    ) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();
        if let Some(pos) = item {
            pos.resources[resource_idx].offset_per_tick = offset;
        }
    }

    pub fn add_unit_resources_at(&mut self, coord: &Coord, unit_resources: &SomeUnitResources) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();
        if let Some(pos) = item {
            pos.add_unit_resources(unit_resources);
        }
    }

    pub fn set_unit_attributes_at(&mut self, coord: &Coord, attributes: UnitAttributes) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            println!("AOEU: {:?}", attributes);
            pos.set_unit_attributes(attributes);
        }
    }

    pub fn set_pos_resource_at(
        &mut self,
        coord: &Coord,
        resource_idx: PositionResourceIndex,
        value: PositionResourceAmount,
    ) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            return pos.set_resource(resource_idx, value, self.tick);
        }

        panic!["Position does not exist at {:?}", coord];
    }

    pub fn get_pos_resource_at(
        &self,
        coord: &Coord,
        resource_idx: PositionResourceIndex,
    ) -> PositionResourceAmount {
        let mut item = self.grid.get([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            return pos.get_resource(resource_idx, self.tick);
        }

        panic!["Position does not exist at {:?}", coord];
    }

    pub fn get_pos_attribute_at(
        &self,
        coord: &Coord,
        attr_idx: PositionAttributeIndex,
    ) -> PositionAttributeValue {
        let mut item = self.grid.get([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            return pos.get_attribute(attr_idx);
        }

        panic!["Position does not exist at {:?}", coord];
    }

    pub fn set_pos_attribute_at(
        &mut self,
        coord: &Coord,
        attr_idx: PositionAttributeIndex,
        val: PositionAttributeValue,
    ) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            return pos.set_attribute(attr_idx, val);
        }

        panic!["Position does not exist at {:?}", coord];
    }

    pub fn set_pos_attributes_at(&mut self, coord: &Coord, attributes: PositionAttributes) {
        let mut item = self.grid.get_mut([coord.0, coord.1]).unwrap();

        if let Some(pos) = item {
            pos.set_attributes(attributes);
        }
    }

    pub fn get_pos_at_dir(&self, coord: &Coord, dir: GridDirection) -> Option<&Position> {
        let c = coord_by_direction_offset(coord, &dir, self.size);

        match c {
            Some(coord) => self.get_position_at(&coord),
            None => None,
        }
    }

    pub fn get_pos_at_offset(&self, coord: &Coord, offset: CoordOffset) -> Option<&Position> {
        let maybe_coord = coord_by_coord_offset(coord, offset, self.size);

        if maybe_coord.is_some() {
            Some(self.get_position_at(&maybe_coord.unwrap()).unwrap())
        } else {
            None
        }
    }
}

// use ndarray::ArrayViewMut;
// pub struct WorldViewMut<'a, 'b, A, D> {
//     sim_attr: &'a mut SimulationAttributes,
//     unit_entry_attr: &'a mut UnitEntryAttributes,
//     grid: ArrayViewMut<'b, A, D>,
// }

// pub struct LocalWorldViewMut<'a, 'b, A, D> {
//     sim_attr: &'a mut SimulationAttributes,
//     unit_entry_attr: &'a mut UnitEntryAttributes,
//     coord: &'a Coord,
//     size: usize,
//     local_grid: ArrayViewMut<'b, A, D>,
// }

pub mod tests {
    use crate::simulation::common::{
        helpers::place_units::PlaceUnitsMethod, variants::CheeseChemistry,
    };

    use super::*;

    #[test]
    pub fn set_some_resources() {
        let specs = SimulationSpecs {
            chemistry_key: "cheese".to_string(),
            place_units_method: PlaceUnitsMethod::Skip,
            ..Default::default()
        };

        let chemistry: ChemistryInstance = specs.construct_chemistry();
        let mut world = World::new((5, 5), &chemistry);
        let unit_entry = UnitEntry::new("foo_unit", NullBehavior::construct());
        let coord = (2, 2);
        world.seed_unit_at(&coord, &unit_entry.info, None, &chemistry);

        world.set_some_unit_resources_at(&coord, &vec![Some(0), Some(0)], &chemistry);
        assert_eq!(world.get_unit_resource_at(&coord, 0), 0);
        assert_eq!(world.get_unit_resource_at(&coord, 1), 0);

        world.set_some_unit_resources_at(&coord, &vec![Some(1), None], &chemistry);
        assert_eq!(world.get_unit_resource_at(&coord, 0), 1);

        world.set_some_unit_resources_at(&coord, &vec![None, Some(5)], &chemistry);
        assert_eq!(world.get_unit_resource_at(&coord, 0), 1);
        assert_eq!(world.get_unit_resource_at(&coord, 1), 5);

        world.set_some_unit_resources_at(&coord, &vec![None, None], &chemistry);
        assert_eq!(world.get_unit_resource_at(&coord, 0), 1);
        assert_eq!(world.get_unit_resource_at(&coord, 1), 5);
    }
}

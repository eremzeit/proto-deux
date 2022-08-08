use crate::simulation::common::*;
use crate::util::{coord_by_coord_offset, Coord, CoordOffset, GridDirection, GridSize2D};
use ndarray::*;
use ndarray::{Array, Array2, Dim, Ix, Shape};

pub struct CoordOffsetIterator {
    iter: OffsetIterator,
    coord: Coord,
    grid_size: GridSize2D,
}

impl Iterator for CoordOffsetIterator {
    type Item = (Coord, GridDirection);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next = self.iter.next();
            if next.is_none() {
                return None;
            }

            let (coord_offset, dir) = next.unwrap();

            let result = coord_by_coord_offset(&self.coord, coord_offset, self.grid_size);
            if result.is_some() {
                return Some((result.unwrap(), dir));
            }
        }
    }
}

impl CoordOffsetIterator {
    pub fn new(coord: &Coord, grid_size: &GridSize2D) -> Self {
        Self {
            coord: coord.clone(),
            iter: OffsetIterator::new(),
            grid_size: grid_size.clone(),
        }
    }
}

pub struct OffsetIterator {
    direction: Option<GridDirection>,
}

impl OffsetIterator {
    pub fn new() -> Self {
        OffsetIterator {
            direction: Some(GridDirection::Up),
        }
    }
}

impl Iterator for OffsetIterator {
    type Item = (CoordOffset, GridDirection);

    fn next(&mut self) -> Option<Self::Item> {
        let current_dir = self.direction.clone();
        match self.direction {
            Some(GridDirection::Up) => {
                self.direction = Some(GridDirection::Right);
                Some(((0, 1), GridDirection::Up))
            }

            Some(GridDirection::Right) => {
                self.direction = Some(GridDirection::Down);
                Some(((1, 0), GridDirection::Right))
            }

            Some(GridDirection::Down) => {
                self.direction = Some(GridDirection::Left);
                Some(((0, -1), GridDirection::Down))
            }

            Some(GridDirection::Left) => {
                self.direction = None;
                Some(((-1, 0), GridDirection::Left))
            }

            None => None,
        }
    }
}

pub struct CoordIterator {
    x: usize,
    y: usize,
    size: GridSize2D,
}

impl CoordIterator {
    pub fn new(size: GridSize2D) -> CoordIterator {
        CoordIterator { x: 0, y: 0, size }
    }

    // pub fn only_units(self, world: &World) -> Box<dyn Iterator<Item=Unit>> {
    //   let a = self.filter(|coord| -> bool {
    //       world.has_unit_at(coord)
    //   });

    //   a
    // }
}

impl Iterator for CoordIterator {
    type Item = Coord;

    // traverse by rows
    fn next(&mut self) -> Option<Self::Item> {
        let x_size = self.size.0;
        let y_size = self.size.1;

        if self.x < x_size {
            let coord = (self.x, self.y);
            self.x += 1;
            return Some(coord);
        }

        self.x = 0;
        self.y += 1;

        if self.y < y_size {
            let coord = (self.x, self.y);
            self.x += 1;
            return Some(coord);
        }

        None
    }
}

// pub struct PositionIterator<'a> {
//     x: usize,
//     y: usize,
//     world: &'a World,
// }
//
// impl<'a> PositionIterator<'a> {
//     pub fn new(world: &'a World) -> PositionIterator<'a> {
//         PositionIterator {
//             x: 0,
//             y: 0,
//             world,
//         }
//     }
//
//     pub fn reset(&mut self) {
//         self.x = 0;
//         self.y = 0;
//     }
// }
//
// impl<'a> Iterator for PositionIterator<'a> {
//     type Item = &'a Position;
//
//     // traverse by rows
//     fn next(&mut self) -> Option<Self::Item> {
//         let x_size = self.world.size.0;
//         let y_size = self.world.size.1;
//
//         if self.x < x_size {
//             assert!(self.y < y_size);
//             let coord = (self.x, self.y);
//             let position = self.world.get_position_at(&coord);
//             self.x += 1;
//             return Some(position);
//         }
//
//         self.x = 0;
//         self.y += 1;
//
//         if self.y < y_size {
//             let coord = (self.x, self.y);
//             let position = self.world.get_position_at(&coord);
//             self.x += 1;
//             return Some(position);
//         }
//
//         None
//     }
// }

// pub struct UnitIterator<'a> {
//   position_iterator: PositionIterator<'a>,
//   world: &'a World,
// }
//
// impl<'a> UnitIterator<'a> {
//     pub fn new(world: &'a World) -> UnitIterator<'a> {
//         UnitIterator {
//             world,
//             position_iterator: PositionIterator::new(world),
//         }
//     }
// }
//
// impl<'a> Iterator for UnitIterator<'a> {
//     type Item = Coord;
//
//     // traverse by rows
//     fn next(&mut self) -> Option<Self::Item> {
//       loop {
//         let maybe_pos = self.position_iterator.next();
//
//         if maybe_pos.is_none() {
//           return None;
//         }
//
//         let coord = maybe_pos.unwrap();
//         if let Some(unit) = self.world.get_unit_at(&coord) {
//           return Some(coord.clone());
//         }
//
//       }
//     }
// }

mod test {
    mod coord_offset_iterator {
        use super::*;
        use crate::simulation::iterators::CoordOffsetIterator;
        use crate::util::{GridDirection, GridSize2D};

        fn test() {
            let mut _iter = CoordOffsetIterator::new(&(2, 2), &(5, 5));
            assert_eq!(_iter.next().unwrap(), ((2, 3), GridDirection::Up));
        }
    }
}

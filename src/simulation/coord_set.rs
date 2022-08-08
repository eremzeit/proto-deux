use util::{Coord, GridSize2D};
use std::convert::TryFrom;

// use roaring::{RoaringBitmap};
// use roaring::bitmap;
// 
// #[derive(Clone)]
// pub struct CoordSet {
//     pub size: GridSize2D,
//     pub bitmap: RoaringBitmap,
// }
// 
// pub struct CoordSetIter<'a> {
//     iter: bitmap::Iter<'a>,
//     size: GridSize2D,
// }
// 
// impl Iterator for CoordSetIter<'_> {
//     type Item = Coord;
// 
//     fn next(&mut self) -> Option<Coord> {
//         match self.iter.next() {
//             Some(num) => Some(idx_to_coord(num, &self.size)),
//             None => None
//         }
//     }
// }
// 
// 
// impl CoordSet {
//     pub fn new(size: GridSize2D) -> Self {
//        Self {
//            size,
//            bitmap: RoaringBitmap::new(),
//        }
//     }
// 
//     pub fn iter(&self) -> CoordSetIter {
//         CoordSetIter {
//             iter: self.bitmap.iter(),
//             size: self.size.clone(),
//         }
//     }
// 
//     pub fn insert(&mut self, coord: &Coord) {
//         self.bitmap.insert(coord_to_idx(coord, &self.size));
//     }
// 
//     pub fn exists(&self, coord: &Coord) -> bool {
//         self.bitmap.contains(coord_to_idx(coord, &self.size))
//     }
//     pub fn remove(&mut self, coord: &Coord) {
//         self.bitmap.remove(coord_to_idx(coord, &self.size));
//     }
// }
// 
// fn idx_to_coord(hash: u32, size: &GridSize2D) -> Coord {
//        let y = (hash as usize) / size.0;
//        let x = (hash as usize - y * (size.0));
//        (x as usize, y as usize)
// }
// 
// fn coord_to_idx(coord:&Coord, size: &GridSize2D) -> u32 {
//     let x = coord.1.checked_mul(size.0).unwrap().checked_add(coord.0).unwrap();
//     u32::try_from(x).ok().unwrap()
// }
// 
// pub mod test {
//     use super::*;
// 
//     #[test]
//     pub fn convert_idx() {
//         let size = (4, 5);
// 
//         let cases = vec![
//             ((0,0), 0),
//             ((1,0), 1),
//             ((2,0), 2),
//             ((0,1), 4),
//             ((0,2), 8),
// 
//             ((1,2), 9),
//             ((2,2), 10),
//         ];
// 
//         for (coord, idx) in &cases {
//             assert_eq!(coord_to_idx(coord, &size), *idx);
//             assert_eq!(idx_to_coord(*idx, &size), coord.clone());
//             assert_eq!(idx_to_coord(coord_to_idx(coord, &size), &size), coord.clone());
//         }
// 
//         let size = (5, 5);
// 
//         let cases = vec![
//             ((0,0), 0),
//             ((1,0), 1),
//             ((4,0), 4),
//             ((0,1), 5),
//             ((1,1), 6),
//         ];
// 
//         for (coord, idx) in &cases {
//             assert_eq!(coord_to_idx(coord, &size), *idx);
//             assert_eq!(idx_to_coord(*idx, &size), coord.clone());
//             assert_eq!(idx_to_coord(coord_to_idx(coord, &size), &size), coord.clone());
//         }
//     }
// 
//     #[test]
//     pub fn test_iter() {
//         let mut set = CoordSet::new((5,5));
//         set.insert(&(1,1));
//         let coords = set.iter().collect::<Vec<Coord>>();
//         assert_eq!(coords, vec![(1,1)])
//     }
// }

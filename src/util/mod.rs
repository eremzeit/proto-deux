pub mod cli;
pub mod text_grid;

#[macro_use]
pub mod macros;

use std::cmp::{max, min};
use std::time::{Duration, Instant};

pub type Coord = (usize, usize);
pub type CoordOffset = (i32, i32);
pub type GridSize2D = (usize, usize);

pub struct RateCounter {
    pub count: u128,
    pub last_update: Instant,
    pub last_rate: f64,
}

impl RateCounter {
    pub fn new() -> Self {
        Self {
            count: 0,
            last_update: Instant::now(),
            last_rate: 0.0,
        }
    }

    pub fn inc_and_update(&mut self) {
        //println!("inc");
        self.count = self.count + 1;

        let elapsed_ms = self.last_update.elapsed().as_millis();
        if (elapsed_ms > 5000) {
            self.last_rate = (self.count as f64 / elapsed_ms as f64) as f64;
            println!(
                "rate: {:?}",
                (self.last_rate * 1000.0, self.count, elapsed_ms)
            );

            self.count = 0;
            self.last_update = Instant::now();
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum GridDirection {
    Up,
    Right,
    Down,
    Left,
}

use std::fmt::{Debug, Formatter, Result};

impl Debug for GridDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            GridDirection::Up => write!(f, "(UP)"),
            GridDirection::Down => write!(f, "(DOWN)"),
            GridDirection::Right => write!(f, "(RIGHT)"),
            GridDirection::Left => write!(f, "(LEFT)"),
        }
    }
}

pub fn grid_direction_to_num(dir: GridDirection) -> u8 {
    match dir {
        GridDirection::Up => 0,
        GridDirection::Right => 1,
        GridDirection::Down => 2,
        GridDirection::Left => 3,
    }
}

pub fn grid_direction_from_num(num: u8) -> GridDirection {
    match num {
        0 => GridDirection::Up,
        1 => GridDirection::Right,
        2 => GridDirection::Down,
        3 => GridDirection::Left,
        _ => GridDirection::Up,
    }
}

pub fn grid_direction_from_string(key: &str) -> Option<GridDirection> {
    if key.to_lowercase() == "up" {
        return Some(GridDirection::Up);
    } else if key.to_lowercase() == "right" {
        return Some(GridDirection::Right);
    } else if key.to_lowercase() == "down" {
        return Some(GridDirection::Down);
    } else if key.to_lowercase() == "left" {
        return Some(GridDirection::Left);
    }

    return None;
}

pub fn coord_by_direction_offset(
    coord: &Coord,
    direction: &GridDirection,
    size: GridSize2D,
) -> Option<Coord> {
    let offset = match direction {
        GridDirection::Up => (0, 1),
        GridDirection::Right => (1, 0),
        GridDirection::Down => (0, -1),
        GridDirection::Left => (-1, 0),
    };

    //assert_coords_valid_for_size(
    coord_by_coord_offset(coord, offset, size)
}

pub fn coord_by_coord_offset(
    coord: &Coord,
    offset: CoordOffset,
    size: GridSize2D,
) -> Option<Coord> {
    let new_x = coord.0 as i32 + offset.0;
    let new_y = coord.1 as i32 + offset.1;
    if new_x >= 0 && new_y >= 0 && new_x < size.0 as i32 && new_y < size.1 as i32 {
        return Some((new_x as usize, new_y as usize));
    } else {
        return None;
    }
}

pub fn proportional_resize(
    rect_width: u32,
    rect_height: u32,
    target_width: u32,
    target_height: u32,
) -> (u32, u32) {
    let wr = rect_width as f32 / target_width as f32;
    let hr = rect_height as f32 / target_height as f32;

    return if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as u32;
            (target_width as u32, h as u32)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as u32;
            (w, target_height as u32)
        }
    } else {
        (rect_width as u32, rect_height as u32)
    };
}

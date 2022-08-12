use super::CellRenderer;
use crate::chemistry;
use crate::simulation::common::Coord;
use crate::simulation::position::Position;
use crate::simulation::simulation_data::{SimulationData, ThreadedSimulationReference};
use crate::ui::colors::to_color;
use opengl_graphics::GlGraphics;
use piston_window::clear;
use piston_window::ellipse;
use piston_window::rectangle;
use piston_window::types::Color;
use piston_window::Context;
use piston_window::Transformed;
use piston_window::Viewport;

use crate::chemistry::variants::cheese;

pub struct CheeseCellRenderer {}

pub fn get_cheese_pct(pos: &Position, current_tick: u64) -> f64 {
    let res_defs = cheese::defs::PositionResourcesLookup::new();
    let cheese = pos.get_resource(res_defs.cheese, current_tick);

    (cheese as f64 / 1000.0).min(1.0)
}

pub fn get_unit_cheese_size_ratio(pos: &Position) -> f64 {
    let res_defs = cheese::defs::UnitResourcesLookup::new();
    let cheese = pos.get_unit_resource(res_defs.cheese);
    println!("unit cheese: {}", cheese);
    (cheese as f64 / 100.0).min(1.0).max(0.2)
}

pub fn calc_resource_rect(full_rect: [f64; 4], pct: f64, coord: &Coord) -> [f64; 4] {
    let choice = (coord.0 + coord.1) % 4;

    if choice == 0 {
        [
            full_rect[0] as f64,
            full_rect[1] as f64,
            full_rect[2] as f64,
            (full_rect[3] as f64 * pct),
        ]
    } else if choice == 1 {
        [
            full_rect[0] as f64,
            full_rect[1] as f64,
            full_rect[2] as f64 * pct,
            full_rect[3] as f64,
        ]
    } else if choice == 2 {
        // right to left
        [
            full_rect[0] + (1.0 - pct) * full_rect[2],
            full_rect[1],
            full_rect[2] as f64 * pct,
            full_rect[3] as f64,
        ]
    } else {
        // bottom to top
        [
            full_rect[0],
            full_rect[1] + (1.0 - pct) * full_rect[3],
            full_rect[2],
            full_rect[3] * pct,
        ]
    }
}

impl CellRenderer for CheeseCellRenderer {
    fn draw_cell(
        &self,
        sim: &SimulationData,
        g: &mut GlGraphics,
        c: &mut Context,
        x: usize,
        y: usize,
        cell_rect: [f64; 4],
    ) {
        let pos = sim.grid[[x, y]].as_ref().unwrap();

        let attr_defs = chemistry::variants::cheese::defs::PositionAttributesLookup::new();
        let is_cheese_source = pos.get_attribute(attr_defs.is_cheese_source).unwrap_bool();

        // cell bg
        let bg_color = if is_cheese_source {
            [0.0, 200.0, 0.0, 100.0]
        } else {
            [0.0, 0.0, 0.0, 255.0]
        };
        rectangle(bg_color, cell_rect, c.transform.trans(0.0, 0.0), g);

        let cheese_pct = get_cheese_pct(pos, sim.tick);

        // cheese indicator
        if cheese_pct > 0.0 {
            let resource_rect = calc_resource_rect(cell_rect, cheese_pct, &(x, y));
            rectangle(
                to_color([0xA0, 0xC5, 0x5F, 0x44]),
                resource_rect,
                c.transform.trans(0.0, 0.0),
                g,
            );
        }

        if pos.has_unit() {
            let cheese_pct = get_unit_cheese_size_ratio(pos);
            let cell_width = cell_rect[3];
            let width = cell_width * cheese_pct;
            let offset = (cell_width - width) / 2.0;
            let rect = [cell_rect[0] + offset, cell_rect[1] + offset, width, width];

            ellipse(
                [255.0, 0.0, 0.0, 255.0],
                rect,
                c.transform.trans(0.0, 0.0),
                g,
            );
        }
    }

    fn bg_color(&self) -> Color {
        [0.5; 4]
    }
}

// pub fn draw(sim: &SimulationData, g: &mut GlGraphics, c: &mut Context, viewport: Viewport) {
//     let tick = sim.tick;
//     clear([0.5; 4], g);

//     let x = (tick % 10) * 20;
//     let y = (tick / 10) * 20;
//     let rect = [0.0, 0.0, 20.0, 20.0];

//     println!("rendering tick: {}, {:?}", tick, (x, y));
//     rectangle(
//         [0.0, 255.0, 0.0, 255.0],
//         rect,
//         c.transform.trans(x as f64, y as f64),
//         g,
//     );
// }

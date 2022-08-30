pub mod cheese;
pub mod draw_world;
use crate::simulation::simulation_data::{SimulationData, ThreadedSimulationReference};
use opengl_graphics::GlGraphics;
use piston_window::types::Color;
use piston_window::{clear, rectangle, Context, Viewport};

use self::cheese::CheeseCellRenderer;

pub trait CellRenderer {
    fn draw_cell(
        &self,
        sim: &SimulationData,
        g: &mut GlGraphics,
        c: &mut Context,
        x: usize,
        y: usize,
        cell_rect: [f64; 4],
    );
    fn bg_color(&self) -> Color;
}

pub fn draw_world(
    sim: &SimulationData,
    gl: &mut GlGraphics,
    viewport: Viewport,
    cell_renderer: &Box<dyn CellRenderer>,
    cell_size: f64,
) {
    gl.draw(viewport, |mut c, g| {
        let tick = sim.tick;
        clear(cell_renderer.bg_color(), g);

        for x in 0..sim.config.size.0 {
            for y in 0..sim.config.size.1 {
                let x_pos = x as f64 * cell_size;
                let y_pos = y as f64 * cell_size;
                let rect = [x_pos, y_pos, cell_size, cell_size];
                cell_renderer.draw_cell(sim, g, &mut c, x, y, rect);
            }
        }
    });
}

pub fn get_cell_renderer(chemistry_key: &str) -> Box<dyn CellRenderer> {
    match chemistry_key {
        "cheese" => Box::new(CheeseCellRenderer::new()),
        _ => unreachable!(),
    }
}

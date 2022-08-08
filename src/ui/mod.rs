pub mod board;
pub mod event_loop;
pub mod execute;

use fps_counter::FPSCounter;
use piston_window::Transformed;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::piston_window::EventLoop;
use crate::piston_window::RenderEvent;
use crate::piston_window::UpdateEvent;
use piston_window::clear;
use piston_window::rectangle;
use piston_window::{OpenGL, PistonWindow, WindowSettings};

use opengl_graphics::GlGraphics;

const BOARD_WIDTH: usize = 100;
const BOARD_HEIGHT: usize = 100;
const BOARD_SIZE: usize = BOARD_WIDTH * BOARD_HEIGHT;
// pub type Board = [u8; BOARD_SIZE];

pub struct Board {
    grid: [(u8, bool); BOARD_SIZE],
}

impl Board {
    pub fn new() -> Board {
        let mut b = Board {
            grid: [(0, true); BOARD_SIZE],
        };
        for i in 0..BOARD_SIZE {
            b.grid[i] = (rand::random::<u8>(), rand::random::<bool>());
        }
        for i in 0..BOARD_SIZE {
            let mut dir = b.grid[i].1;
            let mut val = b.grid[i].0;
            if i != 0 && i != BOARD_SIZE - 1 {
                let side = rand::random::<u8>() % 3;
                dir = if side == 0 {
                    b.grid[i - 1].1
                } else if side == 1 {
                    b.grid[i].1
                } else {
                    b.grid[i + 1].1
                };

                val = if side == 0 {
                    (b.grid[i - 1].0 / 2 + b.grid[i].0 / 2)
                } else if side == 1 {
                    b.grid[i].0
                } else {
                    (b.grid[i + 1].0 / 2 + b.grid[i].0 / 2)
                };
            }

            b.grid[i] = (val, dir);
        }
        b
    }

    pub fn update_board(&mut self) {
        for i in 0..BOARD_SIZE {
            if self.grid[i].1 {
                //increasing
                self.grid[i] = if self.grid[i].0 == 255 {
                    (255, false)
                } else {
                    (self.grid[i].0 + 1, true)
                };
            } else {
                //decreasing
                self.grid[i] = if self.grid[i].0 == 0 {
                    (0, true)
                } else {
                    (self.grid[i].0 - 1, false)
                };
            }
        }
    }
}

fn main() {
    let graphics_api = piston_window::Api::opengl(4, 2);
    let mut window: PistonWindow = WindowSettings::new("piston", [2048; 2])
        .graphics_api(graphics_api)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(OpenGL::V4_2);
    // let start_time = Instant::now();

    let mut fps = FPSCounter::new();
    // let mut frame_count = 0;
    let mut last_fps = 0;
    let mut fps_printed = false;

    let start_time = Instant::now();

    let frame_time_ms = 5.0;

    let mut board = Board::new();
    // let mut last_board_update = Instant::now();
    window.set_bench_mode(true);

    let max_tile_size = 10.0;
    // let red_color = [255.0, 0.0, 0.0, 255.0];

    while let Some(e) = window.next() {
        let current_time = Instant::now();
        let time_since = current_time.duration_since(start_time).as_millis();
        let current_i = (time_since / 100) % BOARD_SIZE as u128;

        // let tile_size = max_tile_size - ((time_since % 5000) as f64 / 5000.0 * 1.0);
        let tile_size = max_tile_size;

        if let Some(r) = e.render_args() {
            gl.draw(r.viewport(), |c, g| {
                clear([0.5; 4], g);

                for i in 0..BOARD_SIZE {
                    let y = i / BOARD_WIDTH;
                    let x = i % BOARD_WIDTH;

                    let x_pos = (x as f64 * tile_size) as f64;
                    let y_pos = (y as f64 * tile_size) as f64;

                    // let color = if board.grid[i].1 {
                    //     [
                    //         board.grid[i].0 as f32 / 255.0,
                    //         board.grid[i].0 as f32 / 600.0,
                    //         board.grid[i].0 as f32 / 600.0,
                    //         255.0,
                    //     ]
                    // } else {
                    //     [
                    //         board.grid[i].0 as f32 / 600.0,
                    //         board.grid[i].0 as f32 / 600.0,
                    //         board.grid[i].0 as f32 / 255.0,
                    //         255.0,
                    //     ]
                    // };
                    let mut color = [
                        board.grid[i].0 as f32 / 1000.0 + 0.1,
                        board.grid[i].0 as f32 / 255.0,
                        board.grid[i].0 as f32 / 1000.0 + 0.2,
                        255.0,
                    ];

                    if i == current_i as usize {
                        color = [
                            board.grid[i].0 as f32 / 255.0,
                            board.grid[i].0 as f32 / 255.0,
                            board.grid[i].0 as f32 / 255.0,
                            255.0,
                        ];
                    }
                    // let color = [100.0, 100.0, 100.0, 255.0];
                    // let rect = [x_pos, y_pos, x_pos + tile_size, y_pos + tile_size];
                    let rect = [0.0, 0.0, tile_size, tile_size];
                    rectangle(color, rect, c.transform.trans(x_pos, y_pos), g);
                }

                last_fps = fps.tick();
            });
        }

        if let Some(u) = e.update_args() {
            board.update_board();

            // update game state
            // framerate independence
            if u.dt < frame_time_ms {
                thread::sleep(Duration::from_millis((frame_time_ms - u.dt + 2.0) as u64));
            }
        }

        // if last_board_update.elapsed().as_millis() > frame {
        //     update_board(&mut board);
        //     last_board_update = Instant::now();
        // }

        if (time_since / 1000) % 5 == 0 {
            if !fps_printed {
                println!("fps: {}", last_fps);
                fps_printed = true;
            }
        } else {
            fps_printed = false;
        }

        // if let Some(b) = e.press_args() {
        //     match b {
        //         Button::Keyboard(key) => match key {
        //             Key::Q => {
        //                 break;
        //             }
        //             _ => {}
        //         },
        //         _ => {}
        //     }
        // }
    }
}

pub mod colors;
pub mod event_loop;
pub mod execute;
pub mod fake_board;
pub mod world;

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

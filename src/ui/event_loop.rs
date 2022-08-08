use crate::chemistry::{BaseChemistry, CheeseChemistry};
use crate::simulation::common::{
    Position, Simulation, SimulationConfig, SimulationControlEvent, SimulationControlEventSender,
    SimulationData, SimulationEvent,
};
use crate::simulation::simulation_data::ThreadedSimulationReference;

use piston_window::G2dTextureContext;

use crate::util::RateCounter;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fps_counter::FPSCounter;
use piston_window::Transformed;
use std::thread;

use crate::piston_window::EventLoop;
use crate::piston_window::RenderEvent;
use crate::piston_window::UpdateEvent;
use piston_window::clear;
use piston_window::rectangle;
use piston_window::{OpenGL, PistonWindow, WindowSettings};

use opengl_graphics::GlGraphics;

pub fn start_ui_loop(
    _sim: ThreadedSimulationReference,
    mut sim_events: Receiver<SimulationEvent>,
    sender_from_ui: SimulationControlEventSender,
) {
    let mut counter = RateCounter::new();
    let mut started_sim = false;

    let graphics_api = piston_window::Api::opengl(4, 2);
    let mut window: PistonWindow = WindowSettings::new("piston", [2048; 2])
        .graphics_api(graphics_api)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(OpenGL::V4_2);

    let mut fps = FPSCounter::new();
    let mut last_fps = 0;
    let mut fps_printed = false;

    let start_time = Instant::now();

    let frame_time_ms = 5.0;

    window.set_bench_mode(true);

    while let Some(e) = window.next() {
        let current_time = Instant::now();
        let time_since = current_time.duration_since(start_time).as_millis();

        // let tile_size = max_tile_size - ((time_since % 5000) as f64 / 5000.0 * 1.0);
        let tile_size = 10.0;

        if let Some(r) = e.render_args() {
            gl.draw(r.viewport(), |c, g| {
                clear([0.5; 4], g);

                let rect = [0.0, 0.0, tile_size, tile_size];
                rectangle(
                    [0.0, 255.0, 0.0, 255.0],
                    rect,
                    c.transform.trans(100.0, 200.0),
                    g,
                );

                last_fps = fps.tick();
            });
        }

        if let Some(u) = e.update_args() {
            // update game state
            // framerate independence
            if u.dt < frame_time_ms {
                thread::sleep(Duration::from_millis((frame_time_ms - u.dt + 2.0) as u64));
            }
        }

        if (time_since / 1000) % 5 == 0 {
            if !fps_printed {
                println!("fps: {}", last_fps);
                fps_printed = true;
            }
        } else {
            fps_printed = false;
        }

        // if !started_sim {
        //     sender_from_ui.send(SimulationControlEvent::Resume);
        //     started_sim = true;
        // }
    }
}

// pub fn start_ui_loop(_sim: ThreadedSimulationReference, mut sim_events: Receiver<SimulationEvent>, sender_from_ui: SimulationControlEventSender) {

//     //let mut app_view = init_ui();

//     // Construct the window.
//     let mut window: PistonWindow =
//         WindowSettings::new("Protorust", [WIDTH, HEIGHT])
//             .graphics_api(OpenGL::V3_2) // If not working, try `OpenGL::V2_1`.
//             .samples(4)
//             .exit_on_esc(true)
//             .vsync(true)
//             .build()
//             .unwrap();

//     window.set_max_fps(8);

//     // construct the CONROD Ui
//     let mut conrod_ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
//         .theme(simulation_ui::theme())
//         .build();

//     // Add a `Font` to the `Ui`'s `font::Map` from file.
//     //conrod_ui.fonts.insert_from_file("/Library/Fonts/DMMono-Regular.ttf").unwrap();

//     // Create a texture for *conrod* to use for efficiently caching text on the GPU.
//     let mut conrod_glyph_cache = conrod_core::text::GlyphCache::builder()
//         .dimensions(WIDTH, HEIGHT)
//         .scale_tolerance(0.1)
//         .position_tolerance(0.1)
//         .build();

//     let buffer_len = WIDTH as usize * HEIGHT as usize;
//     let init = vec![128; buffer_len];
//     let settings = TextureSettings::new();
//     let mut texture_context = window.create_texture_context();
//     let mut text_texture_cache = G2dTexture::from_memory_alpha(&mut texture_context, &init, WIDTH, HEIGHT, &settings).unwrap();

//     // Load the rust logo from file to a piston_window texture.
//     let rust_logo: G2dTexture = {
//         let path = "/Users/zeit/Downloads/download.png";
//         let settings = TextureSettings::new();
//         Texture::from_path(&mut texture_context, &path, Flip::None, &settings).unwrap()
//     };

//     // Create our `conrod_core::image::Map` which describes each of our widget->image mappings.
//     let mut image_map = conrod_core::image::Map::new();
//     let rust_logo = image_map.insert(rust_logo);
//     //let factory: gfx_device_gl::Factory = window.factory.clone();

//     let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
//     let ref font = assets.join("DMMono-Light.ttf");
//     let texture_settings = TextureSettings::new();
//     let mut texture_context2 = window.create_texture_context();
//     let mut glyphs = Glyphs::new(font, texture_context2, texture_settings).unwrap();

//     // Instantiate the generated list of widget identifiers.
//     let ids = simulation_ui::Ids::new(conrod_ui.widget_id_generator());

//     let cell_renderer = get_cell_renderer(&_sim);
//     let mut app = simulation_ui::SimulationApp::new(_sim.clone(), ids, UiSettings {
//         world_background_color: cell_renderer.world_background_color(),
//         side_panel_background_color: cell_renderer.side_panel_background_color(),
//     });

//     let mut counter = RateCounter::new();
//     let mut text_vertex_data = Vec::new();

//     let mut started_sim = false;
//     let mut render_counter: u32 = 0;

//     // Poll events from the window.
//     while let Some(event) = window.next() {
//         let cell_renderer = get_cell_renderer(&_sim);

//         // Convert the src event to a conrod event.
//         let size = window.size();
//         let (win_w, win_h) = (size.width as conrod_core::Scalar, size.height as conrod_core::Scalar);

//         let evt = event.clone();

//         if let Some(e) = gui::conrod::ui_events::convert(evt, win_w, win_h) {
//             conrod_ui.handle_event(e);
//         }

//         // runs the closure if the event is an update event
//         event.update(|_| {
//             app.layout_simulation_ui(&mut conrod_ui.set_widgets(), &_sim.clone(), [win_w, win_h]);
//         });

//         /*
//          * window.draw_2d is a convenience function from piston_window that sets up a
//          * gfx backend, via the piston graphics library
//          *
//          */
//         window.draw_2d(&event, |context: Context, graphics: &mut G2d, device| {
//             // An assembly of states that affect regular draw calls
//             let draw_state = DrawState::new_alpha();

//             if let Some(primitives) = conrod_ui.draw_if_changed() {

//                 // A function used for caching glyphs to the texture cache.
//                 let cache_queued_glyphs = |_graphics: &mut G2d,
//                                            cache: &mut G2dTexture,
//                                            rect: conrod_core::text::rt::Rect<u32>,
//                                            data: &[u8]|
//                     {
//                         let offset = [rect.min.x, rect.min.y];
//                         let size = [rect.width(), rect.height()];
//                         let format = piston_window::texture::Format::Rgba8;
//                         text_vertex_data.clear();
//                         text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
//                         UpdateTexture::update(cache, &mut texture_context, format, &text_vertex_data[..], offset, size)
//                             .expect("failed to update texture")
//                     };

//                 // Specify how to get the drawable texture from the image. In this case, the image
//                 // *is* the texture.
//                 fn texture_from_image<T>(img: &T) -> &T { img }

//                 // Draw the conrod `render::Primitives`.
//                 gui::conrod::draw::primitives(primitives,
//                                                 context,
//                                                 graphics,
//                                                 &mut text_texture_cache,
//                                                 &mut conrod_glyph_cache,
//                                                 &image_map,
//                                                 cache_queued_glyphs,
//                                                 texture_from_image);

//                 texture_context.encoder.flush(device);
//             }

//             // // if let Some(rect) = conrod_ui.rect_of(app.ids.side_panel) {
//             // //     //println!("side panel: {:?}", rect);
//             // // }
//             //
//             if let Some(rect) = conrod_ui.rect_of(app.ids.world_panel) {
//                 //println!("world panel: {:?}", rect);
//                 let x = 30.0;
//                 let y = 30.0;

//                 //
//                 let height = HEIGHT as f64 - 500.0;
//                 let width = WIDTH as f64 - 500.0;

//                 let dim = rect.dim();
//                 let world_target_rect = [x, y, dim[0], dim[1]];
//                 //println!("world target rect: {:?}", &world_target_rect);
//                 //let world_target_rect = [x, y, width, height];

//                 render_world(world_target_rect, &mut sim_events, &_sim.clone(), graphics, &draw_state, context, &cell_renderer, &mut glyphs);

//                 glyphs.factory.encoder.flush(device);
//             }
//         });

//         if !started_sim {
//             sender_from_ui.send(SimulationControlEvent::Resume);
//             started_sim = true;
//         }
//     }
// }

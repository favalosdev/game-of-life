use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::EventPump;
use sdl2::mouse::MouseState;

use crate::camera::Camera;
use crate::feedback::{Feedback, MouseCoords};
use crate::config::*;
use crate::quad_tree::QuadTree;

pub struct InputState {
    pub is_paused: bool,
    pub show_grid: bool
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            is_paused: true,
            show_grid: false,
        }
    }
}

pub fn handle_input(
    event_pump: &mut EventPump,
    camera: &mut Camera,
    quad_tree: &mut QuadTree,
    feedback: &mut Feedback,
    input_state: &mut InputState,
) -> bool {
    let mouse_state: MouseState = event_pump.mouse_state();
    let (x_w, y_w) = camera.from_screen_coords(mouse_state.x() - OFFSET_X, mouse_state.y() - OFFSET_Y);
    feedback.mouse_coords = MouseCoords { x: x_w, y: -y_w };
    feedback.cell_count = quad_tree.cell_count();

    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return true; // Signal to quit
            },
            Event::KeyDown { scancode: Some(Scancode::W), .. } => {
                camera.y -= CAMERA_DELTA;
            },
            Event::KeyDown { scancode: Some(Scancode::A), .. } => {
                camera.x -= CAMERA_DELTA;
            },
            Event::KeyDown { scancode: Some(Scancode::S), .. } => {
                camera.y += CAMERA_DELTA;
            },
            Event::KeyDown { scancode: Some(Scancode::D), .. } => {
                camera.x += CAMERA_DELTA;
            },
            // Zoom in
            Event::KeyDown { scancode: Some(Scancode::I), .. } => {
                camera.zoom += 1;
            },
            // Zoom out
            Event::KeyDown { scancode: Some(Scancode::O), .. } => {
                if camera.zoom > 1 {
                    camera.zoom -= 1;
                }
            },
            Event::KeyDown { scancode: Some(Scancode::P), .. } => {
                input_state.is_paused = !input_state.is_paused;
            },
            Event::KeyDown { scancode: Some(Scancode::E), .. } => {
                if input_state.is_paused {
                    quad_tree.advance();
                }
            },
            Event::KeyDown { scancode: Some(Scancode::G), .. } => {
                input_state.show_grid = !input_state.show_grid;
            },
            /*
            Event::MouseButtonDown { x, y, .. } => {
                if input_state.is_paused {
                    let (x_w, y_w) = camera.from_screen_coords(x - OFFSET_X, y - OFFSET_Y);
                    let coords = (x_w.floor() as isize, -(y_w.floor()) as isize);

                    if grid.is_alive(coords) {
                        grid.cells.remove(&coords);
                    } else {
                        grid.cells.insert(coords);
                    }
                }
            }
            */
            _ => {}
        }
    }

    false // Don't quit
}

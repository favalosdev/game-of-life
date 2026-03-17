extern crate sdl2;

use std::time::{Duration, Instant};
use std::fs::File;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::{MouseState, MouseButton};

use clap::Parser;
use ca_formats::rle::Rle;

mod config;
mod feedback;
mod renderer;
mod input;
mod camera;
mod quad_tree;

use quad_tree::QuadTree;
use camera::Camera;
use config::*;
use feedback::{Feedback, MouseCoords};
use renderer::draw_all;
use input::{InputState, save_pattern};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // File-path of the pattern (in .rle format) to load
    #[arg(short = 'i', long)]
    input: Option<String>,
    // Path where the new pattern is saved to
    #[arg(short = 'o', long)]
    output: Option<String>,
    // Whether the code should run with the HashLife optimization or not
    #[arg(long, default_value_t=false)]
    hash_life: bool 
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Game of Life", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();

    let args = Args::parse();

    let mut quad_tree= QuadTree::new();
    quad_tree.init();

    let mut camera = Camera::new(DEFAULT_ZOOM, 0, 0);

    let file = match args.input {
        Some(path) => File::open(path).unwrap(),
        // Default to opening the Gosper Glider Gun pattern
        None => File::open("assets/patterns/default.rle").unwrap(),
    };

    quad_tree.load_pattern(Rle::new_from_file(file).unwrap());

    let mut feedback = Feedback::new();
    let mut input_state = InputState::new();

    let mut last_game_tick = Instant::now();
    let game_interval = Duration::from_nanos(1_000_000_000 / GAME_FREQ);

    let mut last_qt = quad_tree.get_id();
    let mut last_cells = quad_tree.qt_to_world();

    // Initial render
    draw_all(&mut canvas, &last_cells, &camera, &feedback, input_state.show_grid);

    'running: loop {
        let now = Instant::now();

        if now.duration_since(last_game_tick) >= game_interval {
            last_game_tick = now;

            let current_qt = quad_tree.get_id();

            if last_qt != quad_tree.get_id() {
                last_cells = quad_tree.qt_to_world();
                last_qt = current_qt;
            }

            draw_all(&mut canvas, &last_cells, &camera, &feedback, input_state.show_grid);

            if !input_state.is_paused {
                quad_tree.advance(STEP);
            }
        }

        let mouse_state: MouseState = event_pump.mouse_state();
        let (mx_s, my_s) = (mouse_state.x() - OFFSET_X, OFFSET_Y - mouse_state.y());
        let (mx_w, my_w) = camera.from_screen_coords((mx_s, my_s));
        let zoom = camera.zoom;

        feedback.mouse_coords = MouseCoords { x: mx_w, y: my_w };
        feedback.cell_count = quad_tree.cell_count();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { scancode: Some(Scancode::W), .. } => {
                    camera.y += CAMERA_DELTA / zoom;
                },
                Event::KeyDown { scancode: Some(Scancode::A), .. } => {
                    camera.x -= CAMERA_DELTA / zoom;
                },
                Event::KeyDown { scancode: Some(Scancode::S), .. } => {
                    camera.y -= CAMERA_DELTA / zoom;
                },
                Event::KeyDown { scancode: Some(Scancode::D), .. } => {
                    camera.x += CAMERA_DELTA / zoom;
                },
                Event::KeyDown { scancode: Some(Scancode::I), .. } => {
                    camera.zoom += 1;
                },
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
                        quad_tree.advance(STEP);
                    }
                },
                Event::KeyDown { scancode: Some(Scancode::G), .. } => {
                    input_state.show_grid = !input_state.show_grid;
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                    if input_state.is_paused {
                        quad_tree.toggle((mx_w, my_w));
                    }
                },
                Event::KeyDown { scancode: Some(Scancode::V), .. } => {
                    // Just ignore the Result type for now
                    if input_state.is_paused {
                        save_pattern(&last_cells, args.output.as_ref(), &(quad_tree.b), &(quad_tree.s)).unwrap();
                    }
                }
                _ => {}
            }
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }
}

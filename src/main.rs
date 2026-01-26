extern crate sdl2;
extern crate memoize;

use sdl2::render::Canvas;
use sdl2::video::Window;

use std::time::{Duration, Instant};
use std::fs::File;

use clap::Parser;
use ca_formats::rle::Rle;

mod config;
mod feedback;
mod renderer;
mod input;
mod grid;
mod camera;
mod quad_tree;

use grid::Grid;
use quad_tree::Arena;
use camera::Camera;
use config::*;
use feedback::Feedback;
use renderer::draw_all;
use input::{handle_input, InputState};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// File-path of the pattern (in .rle format) to load
    #[arg(short = 'p', long)]
    pattern_path: Option<String>,
}
    
fn main() {
    // SDL-2 stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Game of Life", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas: Canvas<Window> = window.into_canvas().build().unwrap();

    // Game of life specific stuff
    let args = Args::parse();
    let mut grid = Grid::new();
    let mut arena = Arena::new();
    let mut camera = Camera::new(ZOOM, 0, 0);

    let file = match args.pattern_path {
        Some(path) => File::open(path).unwrap(),
        None => File::open("assets/patterns/gosperglidergun.rle").unwrap(),
    };

    arena.load_pattern(Rle::new_from_file(file).unwrap());

    let mut feedback = Feedback::new();
    let mut input_state = InputState::new();

    let mut last_game_tick = Instant::now();
    let game_interval = Duration::from_nanos(1_000_000_000 / GAME_FREQ);

    // Initial render
    draw_all(&mut canvas, arena.to_world(), &camera, &feedback, input_state.show_grid);

    'running: loop {
        let now = Instant::now();
        if now.duration_since(last_game_tick) >= game_interval {
            last_game_tick = now;
            draw_all(&mut canvas, arena.to_world(), &camera, &feedback, input_state.show_grid);
            if !input_state.is_paused {
                arena.next_gen(arena.root);
                feedback.generation += 1;
            }
        }

        if handle_input(&mut event_pump, &mut camera, &mut grid, &mut feedback, &mut input_state) {
            break 'running;
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
    }
}

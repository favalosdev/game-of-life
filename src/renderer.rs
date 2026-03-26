use std::path::Path;
use std::collections::LinkedList;
use std::{cmp, usize};
use literal::list;
use sdl2::ttf::Sdl2TtfContext;

use std::time::{Duration, Instant};

use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::{MouseState, MouseButton};

use golback::universe::{Universe, WCoord, NodeId};

use crate::config::*;
use crate::save_pattern;
use crate::camera::Camera;

// Stolen macros to handle annoying Rects
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);
pub struct Renderer {
    universe: Universe,
    is_hash_life: bool,
    step: usize,
    // SDL-2 variables
    event_pump: EventPump,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    ttf_context: Sdl2TtfContext,
    // UI/UX variables
    camera: Camera,
    frac_render: bool,
    history: LinkedList<NodeId>,
    is_paused: bool,
    show_grid: bool,
    mouse_coords: WCoord
}

impl Renderer {
    pub fn new(universe: Universe, is_hash_life: bool, step: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Game of Life", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| format!("Failed to build window: {}", e))?;

        let event_pump = sdl_context.event_pump().map_err(|e| format!("Failed to create event pump: {}", e))?;
        let canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| format!("Failed to create canvas: {}", e))?;
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let texture_creator = canvas.texture_creator();

        let instance = Self {
            universe,
            is_hash_life,
            step,
            // SDL-2 variables
            event_pump,
            canvas,
            ttf_context,
            texture_creator,
            // UI/UX variables
            camera: Camera::new(),
            frac_render: false,
            history: list![],
            is_paused: true,
            show_grid: false,
            mouse_coords: (0, 0)
        };

        Ok(instance)
    }

    fn get_rect(&self, point: WCoord) -> Rect {
        let (xo_s, yo_s) = self.camera.from_world_coords(point, self.frac_render);
        let (xf_s, yf_s) = self.camera.from_world_coords((point.0 + 1, point.1 + 1), self.frac_render);

        let r_width = xf_s - xo_s;
        let r_height = yf_s - yo_s;
        
        rect!(xo_s + OFFSET_X, (OFFSET_Y - yo_s) - r_height, r_width, r_height)
    }

    fn draw_grid(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(GRID_COLOR);

        let pivot = self.get_rect((0, 0));
        let square_width = pivot.width() as i32;
        let square_height = pivot.height() as i32;

        let start_x = pivot.x() % square_width;
        let start_y = pivot.y() % square_height;

        let mut x = start_x;

        let w_w = WINDOW_WIDTH as i32;
        let w_h = WINDOW_HEIGHT as i32;

        while x <= w_w {
            self.canvas.draw_line((x, 0), (x, w_h))?;
            x += square_width;
        }

        let mut y = start_y;

        while y <= w_h {
            self.canvas.draw_line((0, y), (w_w, y))?;
            y += square_height;
        }

        Ok(())
    }

    fn draw_squares(&mut self, cells: &LinkedList<WCoord>) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(CELL_COLOR);

        for (x, y) in cells.iter() {
            let to_fill = self.get_rect((*x, *y));
            self.canvas.fill_rect(to_fill)?;
        }

        Ok(())
    }

    fn draw_sim_info(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Load a font
        let mut font = self.ttf_context.load_font(Path::new("assets/IBM_Plex_Mono/IBMPlexMono-Regular.ttf"), 20)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        let mut text = String::new();
        text.push_str(&format!("gen: {}", self.universe.epochs()));
        text.push_str(&format!(" cells: {}", self.universe.population()));

        if !self.frac_render {
            let (mx, my) = self.mouse_coords;
            text.push_str(&format!(" x: {:.2}, y: {:.2}", mx, my));
        } else {
            text.push_str(&" x: --, y: --");
        }

        let surface = font
            .render(&text)
            .blended(TEXT_COLOR)?;
        let texture  = self.texture_creator.create_texture_from_surface(&surface)?;
        let TextureQuery { width, height, .. } = texture.query();
        let padding = 10;
        let target = rect!((WINDOW_WIDTH - padding) - width, (WINDOW_HEIGHT - padding) - height, width, height);
        self.canvas.copy(&texture, None, Some(target))?;

        // Dirty-ass solution

        let text = if self.is_paused { "--PAUSED--" } else { "  LIVE  " };
        let surface = font
            .render(&text)
            .blended(TEXT_COLOR)?;
        let texture  = self.texture_creator.create_texture_from_surface(&surface)?;
        let TextureQuery { width, height, .. } = texture.query();
        let padding = 10;
        let target = rect!(padding, (WINDOW_HEIGHT - padding) - height, width, height);
        self.canvas.copy(&texture, None, Some(target))?;

        Ok(())
    }

    fn draw_sim_state(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut font = self.ttf_context.load_font(Path::new("assets/IBM_Plex_Mono/IBMPlexMono-Regular.ttf"), 20)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);
        Ok(())
    }

    fn draw_all(&mut self, cells: &LinkedList<WCoord>) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.draw_squares(cells)?;
        self.draw_sim_info()?;
        self.draw_sim_state()?;

        if self.show_grid {
            self.draw_grid();
        }

        self.canvas.present();
        Ok(())
    }

    pub fn r#loop(&mut self, output_path: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_game_tick = Instant::now();
        let game_interval = Duration::from_nanos(1_000_000_000 / GAME_FREQ);

        let mut last = self.universe.state();
        let mut coords = self.universe.to_coords();

        // Initial render
        self.draw_all(&coords)?;

        'running: loop {
            let now = Instant::now();

            if now.duration_since(last_game_tick) >= game_interval {
                last_game_tick = now;

                let curr = self.universe.state();

                if last != self.universe.state() {
                    coords = self.universe.to_coords();
                    last = curr;
                }

                if let Err(e) = self.draw_all(&coords) {
                    eprintln!("Drawing error: {}", e);
                    continue;
                }

                if !self.is_paused {
                    if self.is_hash_life {
                        self.universe.hash_life();
                    } else {
                        self.universe.advance(self.step);
                    }
                }
            }

            let mouse_state: MouseState = self.event_pump.mouse_state();
            self.mouse_coords = self.camera.from_screen_coords((mouse_state.x() - OFFSET_X, OFFSET_Y - mouse_state.y()));

            let zoom_factor = if !self.frac_render { self.camera.zoom } else { 1 };

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    },
                    Event::KeyDown { scancode: Some(Scancode::W), .. } => {
                        self.camera.pos.1 += CAMERA_DELTA / zoom_factor;
                    },
                    Event::KeyDown { scancode: Some(Scancode::A), .. } => {
                        self.camera.pos.0 -= CAMERA_DELTA / zoom_factor;
                    },
                    Event::KeyDown { scancode: Some(Scancode::S), .. } => {
                        self.camera.pos.1 -= CAMERA_DELTA / zoom_factor;
                    },
                    Event::KeyDown { scancode: Some(Scancode::D), .. } => {
                        self.camera.pos.0 += CAMERA_DELTA / zoom_factor;
                    },
                    Event::KeyDown { scancode: Some(Scancode::I), .. } => {
                        if !self.frac_render {
                            self.camera.zoom += 1;
                        } else {
                            self.camera.frac_zoom += 0.1 * self.camera.frac_zoom;

                            if self.camera.frac_zoom > 0.99 {
                                self.frac_render = false;
                            }
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::O), .. } => {
                        if !self.frac_render {
                            self.camera.zoom -= 1;

                            if self.camera.zoom == 1 {
                                self.frac_render = true;
                            }
                        } else {
                            self.camera.frac_zoom -= 0.1 * self.camera.frac_zoom;
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::P), .. } => {
                        self.is_paused = !self.is_paused;
                    },
                    Event::KeyDown { scancode: Some(Scancode::E), .. } => {
                        if self.is_paused {
                            if self.is_hash_life {
                                self.universe.hash_life();
                            } else {
                                self.universe.advance(self.step);
                            }
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::G), .. } => {
                        self.show_grid = !self.show_grid;
                    },
                    Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                        if self.is_paused && !self.frac_render {
                            self.universe.toggle(self.mouse_coords);
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::V), .. } => {
                        if self.is_paused && !coords.is_empty() {
                            if let Err(e) = save_pattern(
                                &coords,
                                output_path,
                                &self.universe.b(),
                                &self.universe.s()
                            ) {
                                eprintln!("{}", e);
                            }
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::J), .. } => {
                    },
                    Event::KeyDown { scancode: Some(Scancode::K), .. } => {
                    },
                    _ => {}
                }
            }

            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
        }

        Ok(())
    }
}


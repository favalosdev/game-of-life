use std::path::Path;
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
use sdl2::ttf::Sdl2TtfContext;

use crate::backend::{Coordinates, Universe};
use crate::config::*;
use crate::history::History;
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
    step: u64,
    // SDL-2 variables
    event_pump: EventPump,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    ttf_context: Sdl2TtfContext,
    // UI/UX variables
    camera: Camera,
    frac_render: bool,
    is_running: bool,
    show_grid: bool,
    mouse_coords: Coordinates
}

// Helper function
fn advance(u: &mut Universe, is_hash_life: bool, step: u64) {
    if is_hash_life {
        u.hash_life();
    } else {
        u.advance(step);
    }
}

impl Renderer {
    pub fn new(universe: Universe, is_hash_life: bool, step: u64) -> Result<Self, Box<dyn std::error::Error>> {
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
            is_running: true,
            show_grid: false,
            mouse_coords: (0, 0)
        };

        Ok(instance)
    }

    fn get_rect(&self, point: Coordinates) -> Rect {
        let (xo_s, yo_s) = self.camera.from_world_coords(point);
        let (xf_s, yf_s) = self.camera.from_world_coords((point.0 + 1, point.1 + 1));

        let r_width = xf_s - xo_s;
        let r_height = yf_s - yo_s;
        
        rect!(xo_s + OFFSET_X, (OFFSET_Y - yo_s) - r_height, r_width, r_height)
    }

    fn draw_grid(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(GRID_COLOR);

        let pivot = self.get_rect((0, 0));
        let square_width = pivot.width() as i32;
        let square_height = pivot.height() as i32;
        let mut x = pivot.x().rem_euclid(square_width);
        let mut y = pivot.y().rem_euclid(square_height);

        let w_w = WINDOW_WIDTH as i32;
        let w_h = WINDOW_HEIGHT as i32;

        while x <= w_w {
            self.canvas.draw_line((x, 0), (x, w_h))?;
            x += square_width;
        }

        while y <= w_h {
            self.canvas.draw_line((0, y), (w_w, y))?;
            y += square_height;
        }

        Ok(())
    }

    fn draw_squares(&mut self, cells: &Vec<Coordinates>) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(CELL_COLOR);

        for &p in cells.into_iter() {
            let to_draw = self.get_rect(p);
            self.canvas.fill_rect(to_draw)?;
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

        let surface = font.render(&text).blended(TEXT_COLOR)?;
        let texture  = self.texture_creator.create_texture_from_surface(&surface)?;
        let TextureQuery { width, height, .. } = texture.query();
        let padding = 10;
        let target = rect!((WINDOW_WIDTH - padding) - width, (WINDOW_HEIGHT - padding) - height, width, height);
        self.canvas.copy(&texture, None, Some(target))?;

        // Dirty-ass solution
        let text = if self.is_running { "  LIVE  " } else { "--PAUSED--" };
        let surface = font.render(&text).blended(TEXT_COLOR)?;
        let texture  = self.texture_creator.create_texture_from_surface(&surface)?;
        let TextureQuery { width, height, .. } = texture.query();
        let padding = 10;
        let target = rect!(padding, (WINDOW_HEIGHT - padding) - height, width, height);
        self.canvas.copy(&texture, None, Some(target))?;

        Ok(())
    }

    fn draw_all(&mut self, cells: &Vec<Coordinates>) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.draw_squares(cells)?;
        if self.show_grid { self.draw_grid()?; }
        self.draw_sim_info()?;
        self.canvas.present();
        Ok(())
    }

    pub fn r#loop(&mut self, output_path: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_game_tick = Instant::now();
        let game_interval = Duration::from_nanos(1_000_000_000 / GAME_FREQ);

        let mut last = self.universe.state();
        let mut curr = last;
        let mut coords = self.universe.to_coords().into_iter().collect();
        let mut history = History::new(1000, curr);

        'running: loop {
            let now = Instant::now();

            if now.duration_since(last_game_tick) >= game_interval {
                last_game_tick = now;
                
                if curr != last {
                    last = curr;
                    coords = self.universe.to_coords().into_iter().collect();
                }

                assert_eq!(history.state(), self.universe.state());

                if let Err(e) = self.draw_all(&coords) {
                    eprintln!("Drawing error: {}", e);
                    continue;
                }

                if self.is_running {
                    advance(&mut self.universe, self.is_hash_life, self.step);
                    curr = self.universe.state();
                    history.enqueue(curr);
                }
            }

            let mouse_state: MouseState = self.event_pump.mouse_state();
            self.mouse_coords = self.camera.from_screen_coords((mouse_state.x() - OFFSET_X, OFFSET_Y - mouse_state.y()));
            let zoom_factor = self.camera.zoom.max(1.0) as i32;

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
                            self.camera.zoom += 1.0;
                        } else {
                            self.camera.zoom *= 1.1;

                            if self.camera.zoom > 0.99 {
                                self.frac_render = false;
                                self.camera.zoom = 1.0;
                            }
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::O), .. } => {
                        if !self.frac_render {
                            self.camera.zoom -= 1.0;

                            if self.camera.zoom == 1.0 {
                                self.frac_render = true;
                            }
                        } else {
                            self.camera.zoom *= 0.9;
                            // Safe clipping
                            self.camera.zoom = self.camera.zoom.max(0.0001);
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::P), .. } => {
                        self.is_running = !self.is_running;
                    },
                    Event::KeyDown { scancode: Some(Scancode::E), .. } => {
                        if !self.is_running {
                            advance(&mut self.universe, self.is_hash_life, self.step);
                            curr = self.universe.state();
                            history.enqueue(curr);
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::G), .. } => {
                        self.show_grid = !self.show_grid;
                    },
                    Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                        if !self.is_running && !self.frac_render {
                            self.universe.toggle(self.mouse_coords);
                            curr = self.universe.state();
                            history.enqueue(curr);
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::V), .. } => {
                        if !self.is_running && !coords.is_empty() {
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
                        if !self.is_running {
                            history.unwind();
                            self.universe.set_state(history.state());
                            curr = self.universe.state();
                        }
                    },
                    _ => {}
                }
            }

            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
        }

        Ok(())
    }
}


use std::path::Path;
use std::collections::LinkedList;
use std::{cmp, usize};

use std::time::{Duration, Instant};

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::mouse::{MouseState, MouseButton};

use crate::config::*;
use crate::feedback::{Feedback, MouseCoords};
use crate::quad_tree::{QuadTree, WCoord};
use crate::input::InputState;
use crate::save_pattern;
use crate::camera::Camera;

// Stolen macros to handle annoying Rects
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub struct Renderer {
    camera: Camera,
    feedback: Feedback,
    input_state: InputState,
    event_pump: EventPump,
    canvas: Canvas<Window>,
    frac_render: bool
}

impl Renderer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window("Game of Life", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .map_err(|e| format!("Failed to build window: {}", e))?;

        let event_pump = sdl_context.event_pump().map_err(|e| format!("Failed to create event pump: {}", e))?;
        let canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| format!("Failed to create canvas: {}", e))?;

        let result = Self {
            camera: Camera::new(),
            feedback: Feedback::new(),
            input_state: InputState::new(),
            event_pump,
            canvas,
            frac_render: false
        };

        Ok(result)
    }

    fn get_rect(&self, point: WCoord) -> Rect {
        let (xo_s, yo_s) = self.camera.from_world_coords(point, self.frac_render);
        let (xf_s, yf_s) = self.camera.from_world_coords((point.0 + 1, point.1 + 1), self.frac_render);

        let r_width = xf_s - xo_s;
        let r_height = yf_s - yo_s;
        
        rect!(xo_s + OFFSET_X, (OFFSET_Y - yo_s) - r_height, r_width, r_height)
    }

    fn draw_grid(&mut self, min_x_s: u32, min_y_s: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(GRID_COLOR);

        let square= self.get_rect((0, 0));
        let square_width = square.width();
        let square_height = square.height();

        let start_x = min_x_s % square_width;
        let start_y = min_y_s % square_height;

        let mut x = start_x;

        while x <= WINDOW_WIDTH {
            self.canvas.draw_line((x as i32, 0), (x as i32, WINDOW_HEIGHT as i32))?;
            x += square_width;
        }

        let mut y = start_y;

        while y <= WINDOW_HEIGHT {
            self.canvas.draw_line((0, y as i32), (WINDOW_WIDTH as i32, y as i32))?;
            y += square_height;
        }

        Ok(())
    }

    fn draw_squares(&mut self, cells: &LinkedList<WCoord>) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(CELL_COLOR);

        let mut min_x_s = WINDOW_WIDTH;
        let mut min_y_s = WINDOW_HEIGHT;

        for (x, y) in cells.iter() {
            let to_fill = self.get_rect((*x, *y));
            self.canvas.fill_rect(to_fill)?;

            if to_fill.x >= 0 {
                min_x_s = cmp::min(min_x_s, to_fill.x as u32);
            }

            if to_fill.y >= 0 {
                min_y_s = cmp::min(min_y_s, to_fill.y as u32);
            }
        }

        if self.input_state.show_grid && self.camera.zoom > 6 {
            self.draw_grid(min_x_s, min_y_s)?;
        }

        Ok(())
    }

    fn draw_feedback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let texture_creator = self.canvas.texture_creator();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let padding = 10;

        // Load a font
        let mut font = ttf_context.load_font(Path::new("assets/IBM_Plex_Mono/IBMPlexMono-Regular.ttf"), 20)?;
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        let mx = self.feedback.mouse_coords.x;
        let my = self.feedback.mouse_coords.y;

        let mut text = String::new();

        if self.feedback.epochs < usize::MAX - 1 {
            text.push_str(&format!("gen: {}", self.feedback.epochs));
        }

        text.push_str(&format!(" cells: {}", self.feedback.cell_count));

        if !self.frac_render {
            text.push_str(&format!(" x: {:.2}, y: {:.2}", mx, my));
        }

        // Render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(&text)
            .blended(FEEDBACK_COLOR)?;

        let texture = texture_creator
            .create_texture_from_surface(&surface)?;

        let TextureQuery { width: t_width, height: t_height, .. } = texture.query();

        let target = rect!(WINDOW_WIDTH - t_width - padding, WINDOW_HEIGHT - t_height - padding, t_width, t_height);

        self.canvas.copy(&texture, None, Some(target))?;

        Ok(())
    }

    fn draw_all(&mut self, cells: &LinkedList<WCoord>) -> Result<(), Box<dyn std::error::Error>> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.draw_squares(cells)?;
        self.draw_feedback()?;
        self.canvas.present();
        Ok(())
    }

    pub fn r#loop(&mut self, quad_tree: &mut QuadTree, output_path: Option<&String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut last_game_tick = Instant::now();
        let game_interval = Duration::from_nanos(1_000_000_000 / GAME_FREQ);

        let mut last_qt = quad_tree.get_id();
        let mut last_cells = quad_tree.to_world();

        // Initial render
        self.draw_all(&last_cells)?;

        'running: loop {
            let now = Instant::now();

            if now.duration_since(last_game_tick) >= game_interval {
                last_game_tick = now;

                let current_qt = quad_tree.get_id();

                if last_qt != quad_tree.get_id() {
                    last_cells = quad_tree.to_world();
                    last_qt = current_qt;
                }

                if let Err(e) = self.draw_all(&last_cells) {
                    eprintln!("Drawing error: {}", e);
                    continue;
                }

                if !self.input_state.is_paused {
                    quad_tree.advance();
                }
            }

            let mouse_state: MouseState = self.event_pump.mouse_state();
            let (mx_s, my_s) = (mouse_state.x() - OFFSET_X, OFFSET_Y - mouse_state.y());
            let (mx_w, my_w) = self.camera.from_screen_coords((mx_s, my_s));

            self.feedback.mouse_coords = MouseCoords { x: mx_w, y: my_w };
            self.feedback.cell_count = quad_tree.cell_count();
            self.feedback.epochs = cmp::min(quad_tree.epochs, usize::MAX - 1);
            let aux_zoom = if !self.frac_render { self.camera.zoom } else { 1 };

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    },
                    Event::KeyDown { scancode: Some(Scancode::W), .. } => {
                        self.camera.pos.1 += CAMERA_DELTA / aux_zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::A), .. } => {
                        self.camera.pos.0 -= CAMERA_DELTA / aux_zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::S), .. } => {
                        self.camera.pos.1 -= CAMERA_DELTA / aux_zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::D), .. } => {
                        self.camera.pos.0 += CAMERA_DELTA / aux_zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::I), .. } => {
                        if !self.frac_render {
                            self.camera.zoom += 1;

                        } else {
                            self.camera.frac_zoom += 0.1 * self.camera.frac_zoom;

                            if self.camera.frac_zoom > 0.95 {
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
                        self.input_state.is_paused = !self.input_state.is_paused;
                    },
                    Event::KeyDown { scancode: Some(Scancode::E), .. } => {
                        if self.input_state.is_paused {
                            quad_tree.advance();
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::G), .. } => {
                        self.input_state.show_grid = !self.input_state.show_grid;
                    },
                    Event::MouseButtonDown { mouse_btn: MouseButton::Left, .. } => {
                        if self.input_state.is_paused {
                            quad_tree.toggle((mx_w, my_w));
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::V), .. } => {
                        if self.input_state.is_paused && !last_cells.is_empty() {
                            if let Err(e) = save_pattern(
                                &last_cells,
                                output_path,
                                &quad_tree.b,
                                &quad_tree.s
                            ) {
                                eprintln!("{}", e);
                            }
                        }
                    }
                    _ => {}
                }
            }

            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));
        }

        Ok(())
    }
}


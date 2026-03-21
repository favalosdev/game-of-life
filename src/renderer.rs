use std::path::Path;
use std::collections::LinkedList;
use std::cmp;

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
use crate::quad_tree::QuadTree;
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
    canvas: Canvas<Window>
}

impl Renderer {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Game of Life", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .expect("Failed to build window");

        let event_pump = sdl_context.event_pump().expect("Failed to create event pump");
        let canvas: Canvas<Window> = window.into_canvas().build().expect("Failed to create canvas");

        Self {
            camera: Camera::new(DEFAULT_ZOOM, 0, 0),
            feedback: Feedback::new(),
            input_state: InputState::new(),
            event_pump,
            canvas
        }
    }

    fn get_rect(&self, x_raw: isize, y_raw: isize) -> Rect {
        let (xo_w, yo_w) = (x_raw, y_raw);

        let (xo_s, yo_s) = self.camera.from_world_coords((xo_w, yo_w));
        let (xf_s, yf_s) = self.camera.from_world_coords((xo_w + 1, yo_w + 1));

        let r_width = xf_s - xo_s;
        let r_height = yf_s - yo_s;
        
        rect!(xo_s + OFFSET_X, (OFFSET_Y - yo_s) - r_height, r_width, r_height)
    }

    fn draw_grid(&mut self, min_x_s: u32, min_y_s: u32) {
        self.canvas.set_draw_color(GRID_COLOR);

        let square= self.get_rect(0, 0);
        let square_width = square.width();
        let square_height = square.height();

        let start_x = min_x_s % square_width;
        let start_y = min_y_s % square_height;

        let mut x = start_x;

        while x <= WINDOW_WIDTH {
            let _ = self.canvas.draw_line((x as i32, 0), (x as i32, WINDOW_HEIGHT as i32));
            x += square_width;
        }

        let mut y = start_y;

        while y <= WINDOW_HEIGHT {
            let _ = self.canvas.draw_line((0, y as i32), (WINDOW_WIDTH as i32, y as i32));
            y += square_height;
        }
    }

    fn draw_squares(&mut self, cells: &LinkedList<(isize, isize)>) {
        self.canvas.set_draw_color(CELL_COLOR);

        let mut min_x_s = WINDOW_WIDTH;
        let mut min_y_s = WINDOW_HEIGHT;

        for (x,y) in cells.iter() {
            let to_fill = self.get_rect(*x, *y);
            let _ = self.canvas.fill_rect(to_fill);

            if to_fill.x >= 0 {
                min_x_s = cmp::min(min_x_s, to_fill.x as u32);
            }

            if to_fill.y >= 0 {
                min_y_s = cmp::min(min_y_s, to_fill.y as u32);
            }
        }

        if self.input_state.show_grid {
            self.draw_grid(min_x_s, min_y_s)
        }
    }

    fn draw_feedback(&mut self) {
        let texture_creator = self.canvas.texture_creator();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
        let padding = 10;

        // Load a font
        let mut font = ttf_context.load_font(Path::new("assets/IBM_Plex_Mono/IBMPlexMono-Regular.ttf"), 20).unwrap();
        font.set_style(sdl2::ttf::FontStyle::BOLD);

        let mx = self.feedback.mouse_coords.x;
        let my = self.feedback.mouse_coords.y;
        let cell_count = self.feedback.cell_count;

        let text = format!("cells: {cell_count}, x: {mx:.2}, y: {my:.2}");

        // Render a surface, and convert it to a texture bound to the canvas
        let surface = font
            .render(&text)
            .blended(FEEDBACK_COLOR)
            .unwrap();

        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

        let TextureQuery { width: t_width, height: t_height, .. } = texture.query();

        let target = rect!(WINDOW_WIDTH - t_width - padding, WINDOW_HEIGHT - t_height - padding, t_width, t_height);

        self.canvas.copy(&texture, None, Some(target)).unwrap();
    }

    fn draw_all(&mut self, cells: &LinkedList<(isize, isize)>) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.draw_squares(cells);
        self.draw_feedback();
        self.canvas.present();
    }

    pub fn r#loop(&mut self, quad_tree: &mut QuadTree, output_path: Option<&String>) {
        let mut last_game_tick = Instant::now();
        let game_interval = Duration::from_nanos(1_000_000_000 / GAME_FREQ);

        let mut last_qt = quad_tree.get_id();
        let mut last_cells = quad_tree.to_world();

        // Initial render
        self.draw_all(&last_cells);

        'running: loop {
            let now = Instant::now();

            if now.duration_since(last_game_tick) >= game_interval {
                last_game_tick = now;

                let current_qt = quad_tree.get_id();

                if last_qt != quad_tree.get_id() {
                    last_cells = quad_tree.to_world();
                    last_qt = current_qt;
                }

                self.draw_all(&last_cells);

                if !self.input_state.is_paused {
                    quad_tree.advance(STEP);
                }
            }

            let mouse_state: MouseState = self.event_pump.mouse_state();
            let (mx_s, my_s) = (mouse_state.x() - OFFSET_X, OFFSET_Y - mouse_state.y());
            let (mx_w, my_w) = self.camera.from_screen_coords((mx_s, my_s));
            let zoom = self.camera.zoom;

            self.feedback.mouse_coords = MouseCoords { x: mx_w, y: my_w };
            self.feedback.cell_count = quad_tree.cell_count();

            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    },
                    Event::KeyDown { scancode: Some(Scancode::W), .. } => {
                        self.camera.y += CAMERA_DELTA / zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::A), .. } => {
                        self.camera.x -= CAMERA_DELTA / zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::S), .. } => {
                        self.camera.y -= CAMERA_DELTA / zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::D), .. } => {
                        self.camera.x += CAMERA_DELTA / zoom;
                    },
                    Event::KeyDown { scancode: Some(Scancode::I), .. } => {
                        self.camera.zoom += 1;
                    },
                    Event::KeyDown { scancode: Some(Scancode::O), .. } => {
                        if self.camera.zoom > 1 {
                            self.camera.zoom -= 1;
                        }
                    },
                    Event::KeyDown { scancode: Some(Scancode::P), .. } => {
                        self.input_state.is_paused = !self.input_state.is_paused;
                    },
                    Event::KeyDown { scancode: Some(Scancode::E), .. } => {
                        if self.input_state.is_paused {
                            quad_tree.advance(STEP);
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
    }
}


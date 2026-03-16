use sdl2::pixels::Color;

pub const WINDOW_HEIGHT: u32 = 768;
pub const WINDOW_WIDTH: u32 = 1024;
pub const GAME_FREQ: u64 = 20;
pub const FPS: u32 = 200;
pub const DEFAULT_ZOOM: i32 = 25;
pub const OFFSET_X: i32 = (WINDOW_WIDTH / 2) as i32;
pub const OFFSET_Y: i32 = (WINDOW_HEIGHT / 2) as i32;
pub const CAMERA_DELTA: i32 = 100;
pub const GRID_COLOR: Color = Color::RGB(64, 64, 64);
pub const CELL_COLOR: Color = Color::RGB(0, 255, 0);
pub const FEEDBACK_COLOR: Color = Color::RGB(255, 255, 255);
pub const STEP: usize = 1;

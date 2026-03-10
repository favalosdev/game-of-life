use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use std::path::Path;
use std::collections::LinkedList;
use std::cmp;

use crate::camera::Camera;
use crate::config::*;
use crate::feedback::Feedback;

// Stolen macros to handle annoying Rects
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

fn get_rect(camera: &Camera, x_raw: isize, y_raw: isize) -> Rect {
    let (xo_w, yo_w) = (x_raw, y_raw);
    let (xf_w, yf_w) = (xo_w + 1, yo_w + 1);

    let (xo_s, yo_s) = camera.from_world_coords(xo_w, yo_w);
    let (xf_s, yf_s) = camera.from_world_coords(xf_w, yf_w);

    let r_width = xf_s - xo_s;
    let r_height = yf_s - yo_s;
    
    rect!(xo_s + OFFSET_X, (OFFSET_Y - yo_s) - r_height, r_width, r_height)
}

fn draw_squares(canvas: &mut Canvas<Window>, cells: &LinkedList<(isize, isize)>, camera: &Camera, show_grid: bool) {
    canvas.set_draw_color(CELL_COLOR);

    let mut min_x_s = WINDOW_WIDTH;
    let mut min_y_s = WINDOW_HEIGHT;

    for (x,y) in cells.iter() {
        let to_fill = get_rect(camera, *x, *y);
        let _ = canvas.fill_rect(to_fill);

        if to_fill.x >= 0 {
            min_x_s = cmp::min(min_x_s, to_fill.x as u32);
        }

        if to_fill.y >= 0 {
            min_y_s = cmp::min(min_y_s, to_fill.y as u32);
        }
    }

    if show_grid {
        draw_grid(canvas, camera, min_x_s, min_y_s)
    }
}

fn draw_grid(canvas: &mut Canvas<Window>, camera: &Camera, min_x_s: u32, min_y_s: u32) {
    canvas.set_draw_color(GRID_COLOR);

    let square= get_rect(camera, 0, 0);
    let square_width = square.width();
    let square_height = square.height();

    let start_x = min_x_s % square_width;
    let start_y = min_y_s % square_height;

    let mut x = start_x;

    while x <= WINDOW_WIDTH {
        let _ = canvas.draw_line((x as i32, 0), (x as i32, WINDOW_HEIGHT as i32));
        x += square_width;
    }

    let mut y = start_y;

    while y <= WINDOW_HEIGHT {
        let _ = canvas.draw_line((0, y as i32), (WINDOW_WIDTH as i32, y as i32));
        y += square_height;
    }
}

fn draw_feedback(canvas: &mut Canvas<Window>, feedback: &Feedback) {
    let texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
    let padding = 10;

    // Load a font
    let mut font = ttf_context.load_font(Path::new("assets/IBM_Plex_Mono/IBMPlexMono-Regular.ttf"), 20).unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);

    let mx = feedback.mouse_coords.x;
    let my = feedback.mouse_coords.y;
    let cell_count = feedback.cell_count;

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

    canvas.copy(&texture, None, Some(target)).unwrap();
}

pub fn draw_all(canvas: &mut Canvas<Window>, cells: &LinkedList<(isize, isize)>, camera: &Camera, feedback: &Feedback, show_grid: bool) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_squares(canvas, cells, camera, show_grid);
    draw_feedback(canvas, feedback);
    canvas.present();
}

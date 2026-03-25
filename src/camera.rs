use crate::config::DEFAULT_ZOOM;
use golback::universe::WCoord;

pub struct Camera {
    pub zoom: i32,
    pub frac_zoom: f32,
    pub pos: (i32, i32)
}

impl Camera {
    pub fn new() -> Self {
        Self {
            zoom: DEFAULT_ZOOM,
            frac_zoom: 1.0,
            pos: (0, 0)
        }
    }

    pub fn from_world_coords(&self, (x_w, y_w): WCoord, frac: bool) -> (i32, i32) {
        let zoom = if frac { self.frac_zoom } else { self.zoom as f32 };
        let x_s = ((x_w as i32) - self.pos.0) as f32 * zoom;
        let y_s = ((y_w as i32) - self.pos.1) as f32 * zoom; 
        (x_s.floor() as i32, y_s.floor() as i32)
    } 

    pub fn from_screen_coords(&self, (x_s, y_s): (i32, i32)) -> WCoord {
        let x_w = x_s.div_euclid(self.zoom) + self.pos.0;
        let y_w = y_s.div_euclid(self.zoom) + self.pos.1;
        (x_w as isize, y_w as isize)
    }
}

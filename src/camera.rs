use crate::config::DEFAULT_ZOOM;
use crate::backend::WCoord;

pub struct Camera {
    pub zoom: f32,
    pub pos: (i32, i32)
}

impl Camera {
    pub fn new() -> Self {
        Self {
            zoom: DEFAULT_ZOOM,
            pos: (0, 0)
        }
    }

    pub fn from_world_coords(&self, (x_w, y_w): WCoord) -> (i32, i32) {
        let x_s = ((x_w as f32) - self.pos.0 as f32) * self.zoom;
        let y_s = ((y_w as f32) - self.pos.1 as f32) * self.zoom;
        (x_s.floor() as i32, y_s.floor() as i32)
    } 

    pub fn from_screen_coords(&self, (x_s, y_s): (i32, i32)) -> WCoord {
        let clipped = self.zoom.max(1.0) as i32;
        let x_w = x_s.div_euclid(clipped) + self.pos.0;
        let y_w = y_s.div_euclid(clipped) + self.pos.1;
        (x_w as i64, y_w as i64)
    }
}

use crate::quad_tree::WCoord;

pub struct Camera {
    pub zoom: i32,
    pub x: i32,
    pub y: i32
}

impl Camera {
    pub fn new(zoom: i32, x: i32, y: i32) -> Self {
        Self { zoom, x, y }
    }

    pub fn from_world_coords(&self, (x_w, y_w): WCoord) -> (i32, i32) {
        let x_s = ((x_w as i32) - self.x) * self.zoom;
        let y_s = ((y_w as i32) - self.y) * self.zoom;
        (x_s, y_s)
    }

    pub fn from_screen_coords(&self, (x_s, y_s): (i32, i32)) -> WCoord {
        let x_w = x_s.div_euclid(self.zoom) + self.x;
        let y_w = y_s.div_euclid(self.zoom) + self.y;
        (x_w as isize, y_w as isize)
    }
}

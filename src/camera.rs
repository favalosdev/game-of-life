pub struct Camera {
    pub zoom: f32,
    pub x: f32,
    pub y: f32
}

impl Camera {
    pub fn new(zoom: f32, x: f32, y: f32) -> Self {
        Self { zoom, x, y }
    }

    pub fn from_world_coords(&self, x_w: isize, y_w: isize) -> (i32, i32) {
        let x_s = ((x_w as f32) - self.x) * self.zoom;
        let y_s = ((y_w as f32) - self.y) * self.zoom;
        (x_s.floor() as i32, y_s.floor() as i32)
    }

    pub fn from_screen_coords(&self, x_s: i32, y_s: i32) -> (f32, f32) {
        let x_w = x_s as f32 / self.zoom + self.x;
        let y_w = y_s as f32 / self.zoom + self.y;
        (x_w, y_w)
    }
}

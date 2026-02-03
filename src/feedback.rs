#[derive(Clone, PartialEq)]
pub struct MouseCoords {
    pub x: f32,
    pub y: f32
}

#[derive(Clone, PartialEq)]
pub struct Feedback {
    pub cell_count: usize,
    pub mouse_coords: MouseCoords,
}

impl Feedback {
    pub fn new() -> Self {
        Feedback {
            cell_count: 0,
            mouse_coords: MouseCoords {
                x: 0.0,
                y: 0.0
            }
        }
    }
}

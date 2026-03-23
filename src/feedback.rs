#[derive(Clone, PartialEq)]
pub struct MouseCoords {
    pub x: isize,
    pub y: isize 
}

#[derive(Clone, PartialEq)]
pub struct Feedback {
    pub cell_count: usize,
    pub mouse_coords: MouseCoords,
    pub epochs: usize
}

impl Feedback {
    pub fn new() -> Self {
        Feedback {
            cell_count: 0,
            mouse_coords: MouseCoords {
                x: 0,
                y: 0
            },
            epochs: 0
        }
    }
}

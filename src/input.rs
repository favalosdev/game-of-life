use std::collections::LinkedList;
use crate::quad_tree:: WCoord;

pub struct InputState {
    pub is_paused: bool,
    pub show_grid: bool
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            is_paused: true,
            show_grid: false,
        }
    }
}

pub fn save_pattern(cells: &LinkedList<WCoord>, path: String) {
    let mut init = String::new();
    init += &get_rle_string(cells);
}

fn get_rle_string(cells: &LinkedList<WCoord>) -> String {
    String::from("dummy")
}

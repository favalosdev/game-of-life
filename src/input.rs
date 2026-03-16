use std::collections::LinkedList;
use std::fs::File;
use std::io::prelude::*;

use crate::quad_tree:: WCoord;

pub struct InputState {
    pub is_paused: bool,
    pub show_grid: bool
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            is_paused: true,
            show_grid: false
        }
    }
}

fn format_output_path(path: Option<&String>) -> String {
    // TODO: add more sophisticated formatting in here
    path.map_or(String::from("default.rle"), |v| (*v).clone())
}

pub fn save_pattern(cells: &LinkedList<WCoord>, arg: Option<&String>) -> std::io::Result<()> {
    let init = String::from("dummy_test");
    let path = format_output_path(arg);
    let mut file = File::create(path)?;
    file.write_all(init.as_bytes())?;
    Ok(())
}

fn get_rle_string(cells: &LinkedList<WCoord>) -> String {
    String::from("dummy")
}

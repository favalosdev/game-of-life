extern crate sdl2;

use std::fs::File;

use ca_formats::rle::Rle;
use clap::Parser;

mod config;
mod feedback;
mod renderer;
mod input;
mod camera;
mod quad_tree;

use quad_tree::QuadTree;
use config::*;
use renderer::Renderer;
use input::save_pattern;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // File-path of the pattern (in .rle format) to load
    #[arg(short = 'i', long)]
    input: Option<String>,
    // Path where the new pattern is saved to
    #[arg(short = 'o', long)]
    output: Option<String>,
    // Overrides the "interactive" & "gens" parameters
    #[arg(long, default_value_t=false)]
    hash_life: bool,
    #[arg(short = 't', long, default_value_t=false)]
    interactive: bool,
    #[arg(short='g', long)]
    gens: Option<usize>,
    // Only available in interactive mode
    #[arg(short='s', long, default_value_t=1)]
    step: usize
}

fn main() {
    let args = Args::parse();

    let mut quad_tree= QuadTree::new(
        if !args.interactive { args.gens.expect("\"gens\" parameter should not be missing when not in interactive mode!") } else { args.step },
        args.hash_life
    );

    quad_tree.init();

    let input_path  = args.input.unwrap_or(String::from(DEFAULT_PATTERN_PATH));
    let file = File::open(input_path).expect("Unable to open file");

    quad_tree.load_pattern(Rle::new_from_file(file).unwrap());

    // TODO: define in which cases the application should not execute
    if args.interactive {
        let mut renderer = Renderer::new();
        renderer.r#loop(&mut quad_tree, args.output.as_ref());
    } else {
        quad_tree.advance();

        assert!(args.output.is_some(), "Output filename path arg missing!");

        if let Err(e) = save_pattern(
            &quad_tree.to_world(),
            args.output.as_ref(),
            &quad_tree.b,
            &quad_tree.s
        ) {
            eprintln!("{}", e); 
        }
    }
}

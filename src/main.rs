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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let gens = if !args.interactive {
        args.gens.ok_or("gens parameter required in non-interactive mode")?
    } else {
        args.step
    };

    let mut quad_tree = QuadTree::new(gens, args.hash_life);

    quad_tree.init();

    let input_path = args.input.unwrap_or(String::from(DEFAULT_PATTERN_PATH));
    let file = File::open(&input_path)?;

    quad_tree.load_pattern(Rle::new_from_file(file)?);

    if args.interactive {
        let mut renderer = Renderer::new()?;
        renderer.r#loop(&mut quad_tree, args.output.as_ref())?;
    } else {
        quad_tree.advance();

        let output = args.output.ok_or("output parameter required in non-interactive mode")?;

        save_pattern(
            &quad_tree.to_world(),
            Some(&output),
            &quad_tree.b,
            &quad_tree.s
        )?;
    }

    Ok(())
}

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
use renderer::Renderer;
use input::save_pattern;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // File-path of the pattern (in .rle format) to load
    #[arg(short = 'i', long)]
    input: String,
    // Path where the new pattern is saved to
    #[arg(short = 'o', long)]
    output: Option<String>,
    // Overrides the "interactive" & "gens" parameters
    #[arg(long, default_value_t=false)]
    hash_life: bool,
    // Amount of times hash-life should be executed in non-interactive mode
    #[arg(short = 'r', long, default_value_t=1)]
    repeat: usize,
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

    let gens = if !args.interactive && !args.hash_life {
        args.gens.ok_or("gens parameter required in non-interactive mode")?
    } else {
        // Just ignore whatever falls in here
        args.step
    };

    let mut quad_tree = QuadTree::new(gens, args.hash_life);
    quad_tree.init();

    let file = File::open(&args.input)?;

    quad_tree.load_pattern(Rle::new_from_file(file)?);

    if args.interactive {
        let mut renderer = Renderer::new()?;
        renderer.r#loop(&mut quad_tree, args.output.as_ref())?;
    } else {
        for _ in 1..=args.repeat {
            quad_tree.advance();
        };

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

extern crate sdl2;

use clap::Parser;

mod config;
mod renderer;
mod rle;
mod camera;
mod backend;

use backend::Universe;
use renderer::Renderer;
use rle::save_pattern;

use crate::config::UNIVERSE_DIM;

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
    #[arg(short = 'r', long)]
    repeat: Option<u32>,
    #[arg(short = 't', long, default_value_t=false)]
    interactive: bool,
    #[arg(short='g', long)]
    gens: Option<u64>,
    // Only available in interactive mode
    #[arg(short='s', long, default_value_t=1)]
    step: u64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut universe = Universe::new();
    universe.init(UNIVERSE_DIM);
    universe.load(args.input)?;

    if args.interactive {
        let mut renderer = Renderer::new(universe, args.hash_life, args.step)?;
        renderer.r#loop(args.output.as_ref())?;
    } else {
        let output = args.output.ok_or("output parameter required in non-interactive mode")?;

        if args.hash_life {
            let repeat = args.repeat.ok_or("repeat parameter required when running hashlife in interactive mode")?;
            for _ in 1..=repeat {
                universe.hash_life();
            }
        } else {
            let gens = args.gens.ok_or("gens parameter required in non-interactive mode")?;
            universe.advance(gens);
        }

        save_pattern(
            &universe.to_coords(),
            Some(&output),
            &universe.b(),
            &universe.s()
        )?;
    }

    Ok(())
}

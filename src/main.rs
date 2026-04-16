extern crate sdl2;

mod config;
mod renderer;
mod rle;
mod camera;
mod history;

use golback::universe::Universe;
use renderer::Renderer;
use rle::save_pattern;
use config::UNIVERSE_DIM;

use clap::Parser;

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

    if args.interactive {
        if let Some(input) = args.input {
            universe.load(input)?;
        }

        let ttf_context = sdl2::ttf::init()?; 
        let mut renderer = Renderer::new(universe, args.hash_life, args.step, &ttf_context)?;
        renderer.r#loop(args.output)?;
    } else {
        let output = args.output.ok_or("output parameter required in non-interactive mode")?;
        universe.load(args.input.ok_or("Input parameter required in non-interactive mode")?)?;

        if args.hash_life {
            let repeat = args.repeat.ok_or("repeat parameter required when running hashlife in interactive mode")?;
            for _ in 1..=repeat {
                universe.hash_life();
            }
        } else {
            let gens = args.gens.ok_or("gens parameter required in non-interactive mode")?;
            universe.advance(gens);
        }

        let cells = universe.to_coords().into_iter().collect();

        save_pattern(
            &cells,
            Some(&output),
            &universe.b(),
            &universe.s()
        )?;
    }

    Ok(())
}

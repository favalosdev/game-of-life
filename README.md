# game-of-life

This is a Rust implementation of Conway's Game of Life using SDL2 for graphics, adapted and extended from [this source](https://johnhw.github.io/hashlife/index.md.html). The original hashlife algorithm implementation has been ported to Rust and enhanced with interactive visualization capabilities.

## Running Instructions

1. Clone the repository and navigate to the root folder
2. Generate a build release with `cargo build --release`
3. Execute the program with `./target/release/game-of-life`

## Flags

- `-i, --input <path>`: File-path of the pattern (in .rle format) to load (required)
- `-o, --output <path>`: Path where the new pattern is saved to
- `--hash-life`: Run simulation at full speed by executing Bill Gosper's hashlife algorithm
- `-t, --interactive`: Run in interactive mode with graphical interface
- `-g, --gens <number>`: Number of generations to advance (non-interactive mode only when not using hashlife)
- `-s, --step <number>`: Number of generations to advance per step in interactive mode
- `--repeat <number>`: Number of hashlife iterations to run in non-interactive mode (default 1)

## Examples

- Run non-interactive simulation for 100 generations and save result:

  `./target/release/game-of-life --input assets/patterns/house.rle --output results/house_out.rle --gens 100`

- Run interactive mode (render window, with step 5 by default):

  `./target/release/game-of-life -t --input assets/patterns/gosperglidergun.rle --output results/interactive_out.rle --step 5`

- Run Hashlife mode (ignores `--gens`):

  `./target/release/game-of-life --hash-life --input assets/patterns/koksgalaxy.rle --output results/hashlife_out.rle --repeat 3`

- Run non-interactive simulation with input and gens:

  `./target/release/game-of-life --input assets/patterns/house.rle --output results/house_out.rle --gens 50`

- Run Hashlife without `--interactive`/`--gens`:

  `./target/release/game-of-life --input assets/patterns/koksgalaxy.rle --hash-life --output results/hashlife_out.rle`

## Controls

- **W**: Move the camera upwards
- **A**: Move the camera to the left
- **S**: Move the camera downwards
- **D**: Move the camera to the right
- **I**: Zoom-in
- **O**: Zoom-out
- **P**: Pause/unpause the simulation
- **E**: Advance the game by the specified step size (default 1) generations ahead (only when paused)
- **G**: Toggle grid display

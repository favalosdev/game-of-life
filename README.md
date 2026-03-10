# game-of-life

Adapted and extended from this [source](https://johnhw.github.io/hashlife/index.md.html).

## Running instructions

1. Clone the repository and navigate to the root folder
2. Generate a build release with `cargo build --release`
3. Excecute the program with `./target/release/game-of-life`

## Flags

- `--pattern-path <path>`: Specify the pattern to load. The path must point to a `.rle` file.
- `--hash-life`: Run simulation at full speed by executing Bill Gosper's hashlife algorithm.

## Controls

- **W**: Move the camera upwards
- **A**: Move the camera to the left
- **S**: Move the camera downwards
- **D**: Move the camera to the right
- **I**: Zoom-in
- **O**: Zoom-out
- **P**: Pause
- **E**: Advance the game `step` generations ahead. Can only be done when the game is paused
- **G**: Display the grid in which the squares are rendered

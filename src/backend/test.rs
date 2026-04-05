//! # golback
//!
//! A fast, no-fuss Game of Life backend. Drop it in, feed it cells, and let it run.
//!
//! `golback` exposes a clean API for simulating Conway's Game of Life (and beyond).
//! Under the hood it uses HashLife, a memoized quadtree algorithm that makes large
//! or long-running simulations orders of magnitude faster than naive approaches.
//!
//! ## Features
//!
//! - Clean API: create a universe, load a pattern, advance, inspect
//! - HashLife engine for exponential speedup on large patterns and long runs
//! - Load from RLE files or pass in raw coordinates
//! - Any birth/survival ruleset, not just B3/S23
//! - Toggle, add, or delete individual cells at any point
//!
//! ## Quick Start
//!
//! ```rust
//! use golback::universe::Universe;
//!
//! let mut universe = Universe::new();
//! universe.init(3);
//!
//! universe.load("glider.rle".to_string()).unwrap();
//! universe.advance(100);
//!
//! println!("Population: {}", universe.population());
//! ```

// Tests created by Claude

mod tests {
    use std::collections::LinkedList;
    use crate::backend::{Coordinates, Universe};

    const TEST_DIM: u32 = 5;

    fn make_universe(cells: &[(i64, i64)]) -> Universe {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        let mut list = LinkedList::new();
        for &c in cells {
            list.push_back(c);
        }
        u.from_coords(&list);
        u
    }

    // --- Initialization ---

    #[test]
    fn new_has_zero_population_and_epochs() {
        let u = Universe::new();
        assert_eq!(u.population(), 0);
        assert_eq!(u.epochs(), 0);
    }

    #[test]
    fn init_produces_empty_universe() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        assert_eq!(u.population(), 0);
        let cells = u.to_coords().into_iter().collect::<Vec<_>>();
        assert!(cells.is_empty());
    }

    #[test]
    fn default_rules_are_b3_s23() {
        let u = Universe::new();
        assert_eq!(u.b(), vec![3]);
        assert_eq!(u.s(), vec![2, 3]);
    }

    // --- Cell manipulation ---

    #[test]
    fn add_sets_cell_alive() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        u.add((0, 0));
        assert!(u.is_alive((0, 0)));
        assert_eq!(u.population(), 1);
    }

    #[test]
    fn add_same_cell_twice_does_not_double_count() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        u.add((0, 0));
        u.add((0, 0));
        assert_eq!(u.population(), 1);
    }

    #[test]
    fn delete_removes_alive_cell() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        u.add((0, 0));
        u.delete((0, 0));
        assert!(!u.is_alive((0, 0)));
        assert_eq!(u.population(), 0);
    }

    #[test]
    fn delete_on_dead_cell_is_a_noop() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        u.delete((0, 0));
        assert_eq!(u.population(), 0);
    }

    #[test]
    fn toggle_flips_dead_to_alive() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        u.toggle((1, 1));
        assert!(u.is_alive((1, 1)));
    }

    #[test]
    fn toggle_flips_alive_to_dead() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        u.add((1, 1));
        u.toggle((1, 1));
        assert!(!u.is_alive((1, 1)));
        assert_eq!(u.population(), 0);
    }

    #[test]
    fn is_alive_returns_false_for_dead_cell() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        assert!(!u.is_alive((3, 3)));
    }

    #[test]
    fn is_alive_returns_false_outside_bounds() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        assert!(!u.is_alive((9999, 9999)));
    }

    // --- Coordinate I/O ---

    #[test]
    fn from_coords_to_coords_roundtrip() {
        let input = [(0, 0), (1, 0), (0, 1), (1, 1)];
        let u = make_universe(&input);
        let output = u.to_coords().into_iter().collect::<Vec<_>>();
        assert_eq!(output.len(), input.len());
        for c in &input {
            assert!(output.contains(c));
        }
    }

    #[test]
    fn from_coords_sets_correct_population() {
        let u = make_universe(&[(0, 0), (1, 0), (2, 0)]);
        assert_eq!(u.population(), 3);
    }

    #[test]
    fn to_coords_is_empty_for_dead_universe() {
        let mut u = Universe::new();
        u.init(TEST_DIM);
        let output = u.to_coords().into_iter().collect::<Vec<_>>();
        assert!(output.is_empty());
    }

    // --- Known patterns ---

    // A blinker oscillates between horizontal and vertical with period 2.
    #[test]
    fn blinker_oscillates() {
        let mut u = make_universe(&[(-1, 0), (0, 0), (1, 0)]);
        u.advance(1);
        assert_eq!(u.population(), 3);
        assert!(u.is_alive((0, -1)));
        assert!(u.is_alive((0,  0)));
        assert!(u.is_alive((0,  1)));
        assert!(!u.is_alive((-1, 0)));
        assert!(!u.is_alive(( 1, 0)));
        // One more step should restore the original orientation
        u.advance(1);
        assert!(u.is_alive((-1, 0)));
        assert!(u.is_alive(( 0, 0)));
        assert!(u.is_alive(( 1, 0)));
    }

    // A block is a still life — it should never change.
    #[test]
    fn block_is_still_life() {
        let cells = [(0, 0), (1, 0), (0, 1), (1, 1)];
        let mut u = make_universe(&cells);
        let state_before = u.state();
        u.advance(10);
        assert_eq!(u.state(), state_before);
        assert_eq!(u.population(), 4);
    }

    // A glider returns to its original shape every 4 generations (translated).
    #[test]
    fn glider_preserves_population() {
        let mut u = make_universe(&[(0, 2), (1, 0), (1, 2), (2, 1), (2, 2)]);
        let initial_pop = u.population();
        u.advance(4);
        assert_eq!(u.population(), initial_pop);
    }

    // A single live cell with no neighbors should die.
    #[test]
    fn lone_cell_dies() {
        let mut u = make_universe(&[(0, 0)]);
        u.advance(1);
        assert_eq!(u.population(), 0);
    }

    // Two cells adjacent — both die (underpopulation).
    #[test]
    fn two_adjacent_cells_die() {
        let mut u = make_universe(&[(0, 0), (1, 0)]);
        u.advance(1);
        assert_eq!(u.population(), 0);
    }

    // --- Epoch tracking ---

    #[test]
    fn advance_increments_epochs_correctly() {
        let mut u = make_universe(&[(-1, 0), (0, 0), (1, 0)]);
        u.advance(7);
        assert_eq!(u.epochs(), 7);
    }

    #[test]
    fn multiple_advances_accumulate_epochs() {
        let mut u = make_universe(&[(-1, 0), (0, 0), (1, 0)]);
        u.advance(3);
        u.advance(5);
        assert_eq!(u.epochs(), 8);
    }
}

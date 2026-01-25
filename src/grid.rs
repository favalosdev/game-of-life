use std::collections::{HashSet, LinkedList};
use std::cmp;
use ca_formats::rle::Rle;
use ca_formats::Input;
use crate::quad_tree::Arena;

const RANGE: usize = 2000;

pub struct Grid {
    pub cells: HashSet<(isize, isize)>, // Important: this in (x,y) format
    b: Vec<usize>,
    s: Vec<usize>,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            cells: HashSet::new(),
            // B3/S23
            b: vec![3],
            s: vec![2,3]
        }
    }

    pub fn load_pattern<T : Input>(&mut self, pattern: Rle<T>) {
        let header_data = pattern.header_data().unwrap();
        let width = header_data.x;
        let height = header_data.y;
        let rule = &header_data.rule;

        match rule {
            Some(content) => {
                let parts: Vec<&str> = content.split("/").collect();
                self.b = parts[0][1..].chars().map(|c| c.to_digit(10).unwrap() as usize).collect();
                self.s = parts[1][1..].chars().map(|c| c.to_digit(10).unwrap() as usize).collect();
            },
            _ => {}
        }

        let parsed = pattern
            .map(|cell| cell.unwrap())
            .filter(|data | data.state == 1)
            .map(|data| ((data.position.0 - (width as i64) / 2) as isize, -(data.position.1 - (height as i64) / 2) as isize))
            .collect::<HashSet<_>>();
    
        let mut arena_cells = parsed.clone().into_iter().collect::<LinkedList<_>>(); 
        self.cells = parsed;
        
        let mut arena = Arena::new();

        let top = arena_cells.pop_back();

        match top {
            Some((t_x, t_y)) => {
                let mut max_span = cmp::max(t_x.abs(), t_y.abs());

                for (x, y) in arena_cells.iter() {
                    max_span = cmp::max(x.abs(), y.abs())
                }

                // arena_cells.push_back((t_x, t_y));
                arena.from_world(arena_cells, 0, 0, 4 * (max_span / 4) + (4 as isize));
                println!("Cells correctly loaded into the QuadTree!")
            },
            None => {}
        }
    }

    pub fn is_alive(&self, coords: (isize, isize)) -> bool {
        self.cells.get(&coords).is_some()
    }

    pub fn evolve(&mut self) {
        let mut to_traverse: LinkedList<(isize, isize)> = LinkedList::new();

        for (x,y) in self.cells.iter() {
            let (x, y) = (*x, *y);

            to_traverse.push_back((x,y));
            let mut neighbors = self.get_neighbor_coords((x,y));
            to_traverse.append(&mut neighbors)
        }

        let mut copy: HashSet<(isize, isize)> = HashSet::new();

        for (x, y) in to_traverse.iter() {
            let (x, y) = (*x, *y);
            let will_be_alive = self.transition((x,y));

            if will_be_alive {
                copy.insert((x, y));
            }
        }

        self.cells = copy;
    }

    pub fn get_neighbor_coords(&self, pos: (isize, isize)) -> LinkedList<(isize, isize)> {
        let offsets = [
            (-1, -1), (0, -1), (1, -1),
            (-1,  0),          (1,  0),
            (-1,  1), (0,  1), (1,  1),
        ];

        let (x, y) = pos;

        let coords: LinkedList<(isize, isize)> = offsets.iter().map(|&(dx, dy)| {
            let mut x_f= x + dx;
            let mut y_f = y + dy;

            if x_f.abs() as usize == RANGE {
                x_f = x.abs() * (-1) * dx.signum();
            }

            if y_f.abs() as usize == RANGE {
                y_f = y.abs() * (-1) * dy.signum();
            }

            (x_f, y_f)
        }).collect();

        return coords;
    }

    fn count_alive_neighbors(&self, coords: (isize, isize)) -> usize {
        self.get_neighbor_coords(coords).iter().map(|&(x,y)| self.is_alive((x,y)) as usize).sum()
    }

    pub fn transition(&self, coords: (isize, isize)) -> bool {
        let n = self.count_alive_neighbors(coords);
        if self.is_alive(coords) { self.s.contains(&n) } else { self.b.contains(&n) }
    }
}

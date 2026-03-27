use std::collections::{LinkedList, HashSet};
use std::cmp;
use std::fs::File;

use rustc_hash::FxHashMap;

use literal::list;

use ca_formats::rle::Rle;

const ARENA_SIZE: usize = 500_000;

#[derive(Debug)]
struct Node {
    n: u64, // Number of alive cells within the node
    k: u32, // Size of the quadtree (2**k x 2**k)
    a: NodeId,
    b: NodeId,
    c: NodeId,
    d: NodeId
}

/// Unique identifier type for a node in the Game of Life universe.
///
/// Node IDs are unsigned integers used to index and reference cells efficiently
/// within internal data structures. Each `NodeId` corresponds to a distinct cell or node.
pub type NodeId = usize;

#[derive(Debug, Clone, Copy)]
enum Quadrant {
    NW,
    NE,
    SW,
    SE
}

type Path = Vec<(NodeId, Quadrant)>;

/// World coordinate type representing a cell position in the Game of Life grid.
/// 
/// Coordinates are signed integers allowing for negative positions.
/// The origin (0, 0) is at the center of the universe.
pub type WCoord = (i64, i64);

const VOID: NodeId = 0;
const DEAD: NodeId = 1;
const ALIVE: NodeId = 2;

fn offset(k: u32) -> i64 {
    if k >= 2 {
        2_i64.pow(k - 2)
    } else {
        1
    }
}

fn transform(x: i64) -> i64 {
    x.abs() - (x < 0) as i64
}

fn in_limit(target: WCoord, k: u32) -> bool {
    let limit = 2_i64.pow(k - 1);
    transform(target.0) < limit && transform(target.1) < limit
}

impl Node {
    fn new(
        n: u64,
        k: u32,
        a: NodeId,
        b: NodeId,
        c: NodeId,
        d: NodeId
    ) -> Self {
        Node { n, k, a, b, c, d }
    }
}

struct Caches {
    join: FxHashMap<(NodeId, NodeId, NodeId, NodeId), NodeId>,
    zero: FxHashMap<u32, NodeId>,
    successor: FxHashMap<(NodeId, Option<u32>), NodeId>
}

impl Caches {
    fn new() -> Self {
        Caches {
            join: FxHashMap::default(),
            zero: FxHashMap::default(),
            successor: FxHashMap::default()
        }
    }
}

/// A high-performance Game of Life universe using HashLife algorithm with quadtree compression.
/// 
/// This implementation uses memoization and recursive quadtree structures to efficiently
/// simulate Conway's Game of Life, capable of computing multiple generations per second
/// for many patterns.
/// 
/// # Examples
/// 
/// ```rust
/// use golback::universe::Universe;
/// 
/// // Create a new universe with default B3/S23 rules
/// let mut universe = Universe::new();
/// universe.init();
/// 
/// // Load a pattern from RLE file
/// universe.load("glider.rle".to_string()).unwrap();
/// 
/// // Advance 100 generations
/// universe.advance(100);
/// 
/// // Get current population
/// println!("Population: {}", universe.population());
/// ```
pub struct Universe {
    nodes: Vec<Node>,
    root: NodeId,
    caches: Caches,
    b: Vec<u8>,
    s: Vec<u8>,
    epochs: u64
}

impl Universe {
    /// Creates a new empty universe with default Conway's Game of Life rules (B3/S23).
    /// 
    /// The universe starts uninitialized. Call `init()` to create an empty grid,
    /// or `load()`/`from_coords()` to populate it with a pattern.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use golback::universe::Universe;
    /// 
    /// let mut universe = Universe::new();
    /// universe.init(); // Creates empty grid
    /// ```
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(ARENA_SIZE);

        let void = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let dead = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let alive = Node::new(1, 0, VOID, VOID, VOID, VOID);
        let dummy = Node::new(0, 1, DEAD, DEAD, DEAD, DEAD);
        
        nodes.push(void);
        nodes.push(dead);
        nodes.push(alive);
        nodes.push(dummy);

        let root = 3;

        Self {
            nodes,
            root,
            caches: Caches::new(),
            b: vec![3],
            s: vec![2, 3],
            epochs: 0
        }
    }

    pub fn init(&mut self, dim: u32) {
        self.root = self.zero(dim)
    }

    /// Returns the total number of generations that have elapsed since universe creation.
    /// 
    /// This counter is updated by `advance()` and `hash_life()` operations.
    pub fn epochs(&self) -> u64 {
        self.epochs
    }

    pub fn dim(&self) -> u32 {
        self.nodes[self.root].k
    }

    /// Returns the birth rule set.
    /// 
    /// A cell is born if it has exactly this many alive neighbors.
    /// Default Conway's Game of Life uses B3 (birth with 3 neighbors).
    /// Returns a cloned vector of birth rules.
    pub fn b(&self) -> Vec<u8> {
        self.b.clone()
    }

    /// An alive cell survives if it has exactly this many alive neighbors.
    /// 
    /// Default Conway's Game of Life uses S23 (survive with 2 or 3 neighbors
    /// Returns a cloned vector of survival rules (how many neighbors required for an alive cell to survive).
    pub fn s(&self) -> Vec<u8> {
        self.s.clone()
    }
    /// 
    /// The RLE format can include custom birth/survival rules in the header.
    /// If custom rules are found, they will override the default B3/S23 rules.
    /// The pattern is centered at the origin (0, 0).
    /// 
    /// # Arguments
    /// * `input` - Path to the RLE file
    /// 
    /// # Errors
    /// Returns an error if the file cannot be read or parsed as valid RLE format.
    /// 
    /// # Examples
    /// ```rust
    /// # use golback::universe::Universe;
    /// # let mut universe = Universe::new();
    /// universe.load("patterns/glider.rle".to_string())?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    /// 
    /// # Errors
    /// Returns an error if the file cannot be opened or parsed as valid RLE format.
    pub fn load(&mut self, input: String) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&input)?;
        let pattern = Rle::new_from_file(file)?;

        let header_data = pattern.header_data().unwrap();
        let width = header_data.x;
        let height = header_data.y;
        let rule = &header_data.rule;

        match rule {
            Some(content) => {
                let parts: Vec<&str> = content.split("/").collect();
                self.b = parts[0][1..].chars().map(|c| c.to_digit(10).unwrap() as u8).collect();
                self.s = parts[1][1..].chars().map(|c| c.to_digit(10).unwrap() as u8).collect();
            },
            _ => {}
        }

        let coords  =  pattern
            .map(|cell| cell.unwrap())
            .filter(|data | data.state == 1)
            .map(|data| (data.position.0 - (width as i64) / 2, (height as i64) / 2 - data.position.1))
            .collect::<LinkedList<_>>();

        self.from_coords(&coords);
        Ok(())
    }

    /// Creates a universe from a list of alive cell coordinates.
    /// 
    /// All coordinates not in the list are considered dead.
    /// The universe will automatically expand to accommodate all coordinates.
    /// 
    /// # Arguments
    /// * `cells` - List of (x, y) coordinates representing alive cells
    /// 
    /// # Examples
    /// ```rust
    /// use golback::universe::Universe;
    /// use std::collections::LinkedList;
    /// 
    /// let mut universe = Universe::new();
    /// let mut cells = LinkedList::new();
    /// 
    /// cells.push_back((0, 0));
    /// cells.push_back((1, 0));
    /// cells.push_back((0, 1));
    /// 
    /// universe.from_coords(cells); // Creates a block pattern
    /// ```
    pub fn from_coords(&mut self, cells: &LinkedList<WCoord>) {
        if !cells.is_empty() {
            self.root = self.from_coords_aux(&cells, (0,0), self.dim(), offset(self.dim()))
        }
    }

    /// Returns a unique identifier for the current universe state.
    /// 
    /// This can be used to detect when the universe reaches a previous state
    /// (cycle detection) or to compare universe configurations.
    pub fn state(&self) -> NodeId {
        self.root
    }

    /// Returns the current population (number of alive cells).
    /// 
    /// This count is efficiently maintained and doesn't require scanning the entire universe.
    pub fn population(&self) -> u64 {
        self.nodes[self.root].n
    }

    /// Performs one HashLife iteration, advancing by 2^(k-2) generations.
    /// 
    /// This is the core high-performance operation. The number of generations
    /// advanced depends on the current universe size (larger universes advance more generations).
    /// Use this for maximum performance when you don't need exact generation control.
    /// 
    /// # Examples
    /// ```rust
    /// # use golback::universe::Universe;
    /// # let mut universe = Universe::new();
    /// # universe.load("patterns/gosperglidergun.rle".to_string())?;
    /// universe.hash_life(); // Advance many generations at once
    /// ```
    pub fn hash_life(&mut self) {
        let nested = self.centre(self.root);
        self.root = self.successor(nested, None);
        self.epochs += 2_u64.pow(self.dim() - 2);
    }

    /// Advances the universe by exactly the specified number of generations.
    /// 
    /// This method provides precise control over generation advancement.
    /// For maximum performance with large generation counts, consider using `hash_life()` instead.
    /// 
    /// # Arguments
    /// * `gens` - Number of generations to advance (must be >= 1)
    /// 
    /// # Examples
    /// ```rust
    /// # use golback::universe::Universe;
    /// # let mut universe = Universe::new();
    /// # universe.load("patterns/gosperglidergun.rle".to_string())?;
    /// universe.advance(100); // Advance exactly 100 generations
    /// ```
    pub fn advance(&mut self, gens: u64) {
        self.root = self.advance_aux(self.root, gens);
        self.epochs += gens;
    }

    /// Deletes a cell at the specified coordinates if it exists and is alive.
    /// 
    /// If the target coordinates are within the current universe bounds and the cell
    /// is alive, it will be set to dead. If the coordinates are outside bounds or
    /// the cell is already dead, no action is taken.
    /// 
    /// # Arguments
    /// * `target` - The (x, y) coordinates of the cell to delete

    /*
    pub fn delete(&mut self, target: WCoord) {
        if in_limit(target, self.dim()) {
            let path = self.get_path(target);

            match path.first() {
                Some((ALIVE, _)) => self.backprop(path, DEAD),
                _ => {}
            } 
        }
    }
    */

    /// Adds a cell at the specified coordinates, expanding the universe if necessary.
    /// 
    /// If the target coordinates are within the current universe bounds and the cell
    /// is dead, it will be set to alive. If the coordinates are outside bounds,
    /// the universe will be expanded to accommodate the new cell.
    /// 
    /// # Arguments
    /// * `target` - The (x, y) coordinates of the cell to add
    
    /*
    pub fn add(&mut self, target: WCoord) {
        if in_limit(target, self.dim()) {
            let path = self.get_path(target);

            match path.first() {
                Some((DEAD, _)) => self.backprop(path, ALIVE),
                _ => {}
            } 
        } else {
            let mut k = self.dim();
            let bound = cmp::max(transform(target.0), transform(target.1));

            while 2_i64.pow(k-1) <= bound {
                self.root = self.centre(self.root);
                k += 1;
            }

            self.add(target);
        }
    }
    */

    /// Toggles the state of a cell at the specified coordinates.
    /// 
    /// If the target coordinates are within bounds, the cell state will be flipped
    /// (alive becomes dead, dead becomes alive). If outside bounds, the cell will
    /// be added as alive.
    /// 
    /// # Arguments
    /// * `target` - The (x, y) coordinates of the cell to toggle
    pub fn toggle(&mut self, target: WCoord) {
        if in_limit(target, self.dim()) {
            let path = self.get_path(target);

            match path.first() {
                Some((ALIVE, _)) => self.backprop(path, DEAD),
                Some((DEAD, _)) => self.backprop(path, ALIVE),
                _ => {}
            } 
        }
    }

    /// Checks if a cell at the specified coordinates is alive.
    /// 
    /// Returns true if the coordinates are within bounds and the cell is alive,
    /// false otherwise (including if coordinates are outside bounds).
    /// 
    /// # Arguments
    /// * `target` - The (x, y) coordinates to check
    /// 
    /// # Returns
    /// `true` if the cell is alive, `false` otherwise
    
    /*
    pub fn is_alive(&mut self, target: WCoord) -> bool {
        if in_limit(target, self.dim()) {
            let path = self.get_path(target);
            let (leaf, _) = path[0];
            leaf == ALIVE
        } else {
            false
        }
    }
    */

    /// Returns a list of coordinates of all currently alive cells.
    /// 
    /// This can be used to export the current universe state or for visualization.
    /// The returned coordinates are in no particular order.
    /// 
    /// # Examples
    /// ```rust
    /// # use golback::universe::Universe;
    /// # let mut universe = Universe::new();
    /// # universe.load("patterns/gosperglidergun.rle".to_string())?;
    /// let alive_cells = universe.to_coords();
    /// println!("Found {} alive cells", alive_cells.len());
    /// ```
    pub fn to_coords(&self) -> LinkedList<WCoord> {
        let mut points = list![];
        self.to_coords_aux(self.root, (0, 0), &mut points, offset(self.dim()));
        points
    }

    // Private methods

    fn from_coords_aux(
        &mut self,
        cells: &LinkedList<WCoord>,
        (c_x, c_y): WCoord,
        level: u32,
        prev_offset: i64
    ) -> NodeId {
        if cells.is_empty() {
            self.zero(level)
        } else if level == 1 {
            let lookup: HashSet<&WCoord> = cells.iter().collect();

            let a_coords = (c_x - 1, c_y);
            let b_coords = (c_x, c_y);
            let c_coords = (c_x - 1, c_y - 1);
            let d_coords = (c_x, c_y - 1);

            let a = if lookup.contains(&a_coords) { ALIVE } else { DEAD };
            let b = if lookup.contains(&b_coords) { ALIVE } else { DEAD };
            let c = if lookup.contains(&c_coords) { ALIVE } else { DEAD };
            let d = if lookup.contains(&d_coords) { ALIVE } else { DEAD };

            self.join(a, b, c, d)
        } else {
            let mut ne_cells = list![];
            let mut nw_cells = list![];
            let mut se_cells = list![];
            let mut sw_cells = list![];

            for (x, y) in cells.iter() {
                let p = (*x, *y);

                match (p.0 >= c_x, p.1 >= c_y) {
                    (true, true)   => ne_cells.push_back(p),
                    (true, false)  => se_cells.push_back(p),
                    (false, true)  => nw_cells.push_back(p),
                    (false, false) => sw_cells.push_back(p)
                }
            }

            let offset = prev_offset / 2;
            let nw = self.from_coords_aux(&nw_cells, (c_x - offset, c_y + offset), level - 1, offset);
            let ne = self.from_coords_aux(&ne_cells, (c_x + offset, c_y + offset), level - 1, offset);
            let sw = self.from_coords_aux(&sw_cells, (c_x - offset, c_y - offset), level - 1, offset);
            let se = self.from_coords_aux(&se_cells, (c_x + offset, c_y - offset), level - 1, offset);

            self.join(nw, ne, sw, se)
        }
    }

    fn new_node(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    fn join(&mut self, a: NodeId, b: NodeId, c: NodeId, d: NodeId) -> NodeId {
        if let Some(id) = self.caches.join.get(&(a, b, c, d)) {
            return *id;
        }

        let n = &self.nodes[a].n + &self.nodes[b].n + &self.nodes[c].n + &self.nodes[d].n;
        let to_add = Node::new(n, &self.nodes[a].k + 1, a, b, c, d);
        let result = self.new_node(to_add);
        self.caches.join.insert((a, b, c, d), result);
        result
    }

    fn zero(&mut self, k: u32) -> NodeId {
        if let Some(id) = self.caches.zero.get(&k) {
            return *id;
        }

        let result = if k == 0 {
            DEAD
        } else {
            let z = self.zero(k-1);
            self.join(z, z, z, z)
        };

        self.caches.zero.insert(k, result);
        result
    }

    fn centre(&mut self, m: NodeId) -> NodeId {
        let m_node = &self.nodes[m];
        let (ma, mb, mc, md) = (m_node.a, m_node.b, m_node.c, m_node.d);
        let z = self.zero(m_node.k - 1);
        let ja = self.join(z, z, z, ma);
        let jb = self.join(z, z, mb, z);
        let jc = self.join(z, mc, z, z);
        let jd = self.join(md, z, z, z);
        self.join(ja, jb, jc, jd)
    }

    fn life(&self, a: NodeId, b: NodeId, c: NodeId, d: NodeId, e: NodeId, f: NodeId, g: NodeId, h: NodeId, i: NodeId) -> NodeId {
        let mut outer = 0;

        for id in vec![a, b, c, d, f, g, h, i] {
            outer += self.nodes[id].n as u8;
        }

        if (self.nodes[e].n == 1 && self.s.contains(&(outer))) || self.b.contains(&(outer)) {
            ALIVE
        } else {
            DEAD
        }
    }

    fn life_4x4(&mut self, m: NodeId) -> NodeId {
        let m_node = &self.nodes[m];
        let a = &self.nodes[m_node.a];
        let b = &self.nodes[m_node.b];
        let c = &self.nodes[m_node.c];
        let d = &self.nodes[m_node.d];

        let ad = self.life(a.a, a.b, b.a, a.c, a.d, b.c, c.a, c.b, d.a);
        let bc = self.life(a.b, b.a, b.b, a.d, b.c, b.d, c.b, d.a, d.b);
        let cb = self.life(a.c, a.d, b.c, c.a, c.b, d.a, c.c, c.d, d.c);
        let da = self.life(a.d, b.c, b.d, c.b, d.a, d.b, c.d, d.c, d.d);

        self.join(ad, bc, cb, da)
    }

    fn advance_aux(&mut self, root: NodeId, mut gens: u64) -> NodeId {
        let mut nested = root;
        let mut counter = 0;

        while gens > 0 {
            if (gens & 1) == 1 {
                nested = self.centre(nested);
                nested = self.successor(nested, Some(counter));
            }

            gens = gens >> 1;
            counter += 1;
        }

        nested
    }

    fn successor(&mut self, m: NodeId, j: Option<u32>) -> NodeId {
        if let Some(id) = self.caches.successor.get(&(m, j)) {
            return *id;
        }

        let m_node = &self.nodes[m];
        let level = m_node.k;

        let next = if m_node.n == 0 {
            m_node.a
        } else if level == 2 {
            // Base case. It doesn't need to be memoized
            self.life_4x4(m)
        } else {
            let step = Some(j.map_or(level - 2, |j| cmp::min(j, level - 2)));
            
            let (ma, mb, mc, md) = (m_node.a, m_node.b, m_node.c, m_node.d);
            
            let a = &self.nodes[ma];
            let (aa, ab, ac, ad) = (a.a, a.b, a.c, a.d);
            
            let b = &self.nodes[mb];
            let (ba, bb, bc, bd) = (b.a, b.b, b.c, b.d);
            
            let c = &self.nodes[mc];
            let (ca, cb, cc, cd) = (c.a, c.b, c.c, c.d);
            
            let d = &self.nodes[md];
            let (da, db, dc, dd) = (d.a, d.b, d.c, d.d);
            
            let j1 = self.join(aa, ab, ac, ad);
            let j2 = self.join(ab, ba, ad, bc);
            let j3 = self.join(ba, bb, bc, bd);
            let j4 = self.join(ac, ad, ca, cb);
            let j5 = self.join(ad, bc, cb, da);
            let j6 = self.join(bc, bd, da, db);
            let j7 = self.join(ca, cb, cc, cd);
            let j8 = self.join(cb, da, cd, dc);
            let j9 = self.join(da, db, dc, dd);

            let c1 = self.successor(j1, step);
            let c2 = self.successor(j2, step);
            let c3 = self.successor(j3, step);
            let c4 = self.successor(j4, step);
            let c5 = self.successor(j5, step);
            let c6 = self.successor(j6, step);
            let c7 = self.successor(j7, step);
            let c8 = self.successor(j8, step);
            let c9 = self.successor(j9, step);

            if step.unwrap() < level - 2 {
                let s1 = self.join(self.nodes[c1].d, self.nodes[c2].c, self.nodes[c4].b, self.nodes[c5].a);
                let s2 = self.join(self.nodes[c2].d, self.nodes[c3].c, self.nodes[c5].b, self.nodes[c6].a);
                let s3 = self.join(self.nodes[c4].d, self.nodes[c5].c, self.nodes[c7].b, self.nodes[c8].a);
                let s4 = self.join(self.nodes[c5].d, self.nodes[c6].c, self.nodes[c8].b, self.nodes[c9].a);
                self.join(s1, s2, s3, s4)
            } else {
                let s1 = self.join(c1, c2, c4, c5);
                let s2 = self.join(c2, c3, c5, c6);
                let s3 = self.join(c4, c5, c7, c8);
                let s4 = self.join(c5, c6, c8, c9);

                let ss1 = self.successor(s1, step);
                let ss2 = self.successor(s2, step);
                let ss3 = self.successor(s3, step);
                let ss4 = self.successor(s4, step);

                self.join(ss1, ss2, ss3, ss4)
            }
        };
        self.caches.successor.insert((m, j), next);
        next
    }

    fn backprop(&mut self, path: Path, target: NodeId) {
        let (_, q) = path[0];
        let (parent, _) = path[1];
        let mut updated = self.set_child(parent, q, target);

        for i in 2..path.len() {
            let (_, q) = path[i-1];
            let (curr, _) = path[i];
            updated = self.set_child(curr, q, updated);
        }

        self.root = updated;
    }

    fn set_child(&mut self, parent: NodeId, q: Quadrant, target: NodeId) -> NodeId {
        let Node { a, b, c, d, .. } = self.nodes[parent];

        match q {
            Quadrant::NW => self.join(target, b, c, d),
            Quadrant::NE => self.join(a, target, c, d),
            Quadrant::SW => self.join(a, b, target, d),
            Quadrant::SE => self.join(a, b, c, target)
        }
    } 

    fn get_path(&self, target: WCoord) -> Path {
        let dim = self.dim();
        let mut path = Vec::with_capacity((dim + 1) as usize);
        self.get_path_aux(self.root, Quadrant::SW, target, (0, 0), offset(dim), &mut path);
        path
    }

    fn get_path_aux(
        &self,
        current: NodeId,
        quadrant: Quadrant,
        target: WCoord,
        (c_x, c_y): WCoord,
        prev_offset: i64,
        path: &mut Path
    ) {
        let c_node = &self.nodes[current];
        let level = c_node.k;

        if level > 0 {
            let offset = prev_offset / 2;
            let (x, y) = target;

            match (x >= c_x, y >= c_y) {
                (true, true)   => self.get_path_aux(c_node.b, Quadrant::NE, target, (c_x + offset, c_y + offset), offset, path),
                (true, false)  => self.get_path_aux(c_node.d, Quadrant::SE, target, (c_x + offset, c_y - offset), offset, path),
                (false, true)  => self.get_path_aux(c_node.a, Quadrant::NW, target, (c_x - offset, c_y + offset), offset, path),
                (false, false) => self.get_path_aux(c_node.c, Quadrant::SW, target, (c_x - offset, c_y - offset), offset, path)
            }
        }

        path.push((current, quadrant));
    }

    fn to_coords_aux(
        &self,
        root: NodeId,
        (c_x, c_y): WCoord,
        points: &mut LinkedList<WCoord>,
        prev_offset: i64,
    ) {
        let r_node= &self.nodes[root];
        let level = r_node.k;

        if r_node.n > 0 {
            if level == 1 {
                if r_node.a == ALIVE {
                    points.push_back((c_x - 1, c_y));
                }

                if r_node.b == ALIVE {
                    points.push_back((c_x, c_y));
                }

                if r_node.c == ALIVE {
                    points.push_back((c_x - 1, c_y - 1));
                }

                if r_node.d == ALIVE {
                    points.push_back((c_x, c_y - 1));
                }
            } else {
                let offset = prev_offset / 2;
                self.to_coords_aux(r_node.a, (c_x - offset, c_y + offset), points, offset);
                self.to_coords_aux(r_node.b, (c_x + offset, c_y + offset), points, offset);
                self.to_coords_aux(r_node.c, (c_x - offset, c_y - offset), points, offset);
                self.to_coords_aux(r_node.d, (c_x + offset, c_y - offset), points, offset);
            }
        }
    }
}

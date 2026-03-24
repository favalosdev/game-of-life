use std::collections::{LinkedList, HashSet};
use std::cmp;
use std::fs::File;

use rustc_hash::FxHashMap;

use literal::list;

use ca_formats::rle::Rle;

const DIM: usize = 15;
const ARENA_SIZE: usize = 500_000;

type NodeId = usize;

#[derive(Debug)]
struct Node {
    n: usize, // Number of alive cells within the node
    k: usize, // Size of the quadtree (2**k x 2**k)
    a: NodeId,
    b: NodeId,
    c: NodeId,
    d: NodeId
}

#[derive(Debug, Clone, Copy)]
enum Quadrant {
    NW,
    NE,
    SW,
    SE
}

type Path = Vec<(NodeId, Quadrant)>;
pub type WCoord = (isize, isize);

const VOID: NodeId = 0;
const DEAD: NodeId = 1;
const ALIVE: NodeId = 2;

impl Node {
    fn new(
        n: usize,
        k: usize,
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
    zero: FxHashMap<usize, NodeId>,
    successor: FxHashMap<(NodeId, Option<usize>), NodeId>
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

pub struct Universe {
    nodes: Vec<Node>,
    root: NodeId,
    caches: Caches,
    b: Vec<usize>,
    s: Vec<usize>,
    epochs: usize
}

impl Universe {
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(ARENA_SIZE);

        let void = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let dead = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let alive = Node::new(1, 0, VOID, VOID, VOID, VOID);
        
        nodes.push(void);
        nodes.push(dead);
        nodes.push(alive);

        let caches = Caches::new();

        Self {
            nodes,
            root: VOID,
            caches,
            b: vec![3],
            s: vec![2, 3],
            epochs: 0
        }
    }

    pub fn init(&mut self) {
        self.root = self.zero(cmp::max(DIM, 1_usize));
    }

    pub fn epochs(&self) -> usize {
        self.epochs
    }

    pub fn b(&self) -> Vec<usize> {
        self.b.clone()
    }

    pub fn s(&self) -> Vec<usize> {
        self.s.clone()
    }

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
                self.b = parts[0][1..].chars().map(|c| c.to_digit(10).unwrap() as usize).collect();
                self.s = parts[1][1..].chars().map(|c| c.to_digit(10).unwrap() as usize).collect();
            },
            _ => {}
        }

        let coords  =  pattern
            .map(|cell| cell.unwrap())
            .filter(|data | data.state == 1)
            .map(|data| ((data.position.0 - (width as i64) / 2) as isize, ((height as i64) / 2 - data.position.1) as isize))
            .collect::<LinkedList<_>>();

        self.from_coords(coords);
        Ok(())
    }

    pub fn from_coords(&mut self, cells: LinkedList<(isize, isize)>) {
        self.root = self.from_coords_aux(&cells, (0,0), cmp::max(DIM, 1_usize))
    }

    // Every universe is uniquely represented by an ID
    pub fn state(&self) -> NodeId {
        self.root
    }

    // Convert (x,y) to QuadTree
    fn from_coords_aux(
        &mut self,
        cells: &LinkedList<WCoord>,
        (c_x, c_y): WCoord,
        level: usize 
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

            let offset = 2_isize.pow((level - 2) as u32);
            let nw = self.from_coords_aux(&nw_cells, (c_x - offset, c_y + offset), level - 1);
            let ne = self.from_coords_aux(&ne_cells, (c_x + offset, c_y + offset), level - 1);
            let sw = self.from_coords_aux(&sw_cells, (c_x - offset, c_y - offset), level - 1);
            let se = self.from_coords_aux(&se_cells, (c_x + offset, c_y - offset), level - 1);
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

    fn zero(&mut self, k: usize) -> NodeId {
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
            outer += &self.nodes[id].n;
        }

        if (self.nodes[e].n == 1 && self.s.contains(&outer)) || self.b.contains(&outer) {
            ALIVE
        } else {
            DEAD
        }
    }

    pub fn population(&self) -> usize {
        self.nodes[self.root].n
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

    pub fn hash_life(&mut self) {
        let nested = self.centre(self.root);
        self.root = self.successor(nested, None);
        self.epochs += 2_usize.pow((self.nodes[self.root].k as u32) - 2);
    }

    pub fn advance(&mut self, gens: usize) {
        self.root = self.advance_aux(self.root, gens);
        self.epochs += gens;
    }

    fn advance_aux(&mut self, root: NodeId, mut gens: usize) -> NodeId {
        let mut nested = root;
        let mut index = 0;

        while gens > 0 {
            if (gens & 1) == 1 {
                nested = self.centre(nested);
                nested = self.successor(nested, Some(index));
            }

            gens = gens >> 1;
            index += 1;
        }

        nested
    }

    // Forward's m 2**j generations forward and returns a 2**(k-1) x 2**(k-1) successor.
    // The default value of j is k-2.

    fn successor(&mut self, m: NodeId, j: Option<usize>) -> NodeId {
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

    // Toggle the cell in coordinate (x, y)
    pub fn toggle(&mut self, target: WCoord) {
        let path = self.search(target);
        
        let (leaf, q) = path[0];
        let (parent, _) = path[1];
        let mut updated = self.set_child(parent, q, if leaf == ALIVE { DEAD } else { ALIVE });

        for i in 2..path.len() {
            let (_, q) = path[i-1];
            let (curr, _) = path[i];
            updated = self.set_child(curr, q, updated);
        }

        self.root = updated;
    }

    // Aux function to toggle
    fn set_child(&mut self, parent: NodeId, q: Quadrant, target: NodeId) -> NodeId {
        let Node { a, b, c, d, .. } = self.nodes[parent];

        match q {
            Quadrant::NW => self.join(target, b, c, d),
            Quadrant::NE => self.join(a, target, c, d),
            Quadrant::SW => self.join(a, b, target, d),
            Quadrant::SE => self.join(a, b, c, target)
        }
    } 

    // Returns the path from the root node to the target
    fn search(&self, target: WCoord) -> Path {
        let mut path = Vec::with_capacity(cmp::max(DIM, 1) + 1);
        self.search_aux(self.root, Quadrant::SW, target, (0, 0), &mut path);
        path
    }

    fn search_aux(
        &self,
        current: NodeId,
        quadrant: Quadrant,
        target: WCoord,
        (c_x, c_y): WCoord,
        path: &mut Path
    ) {
        let c_node = &self.nodes[current];
        let level = c_node.k;

        if level > 0 {
            let offset = if level > 1 { 2_isize.pow((level - 2) as u32) } else { 1 };
            let (x, y) = target;

            match (x >= c_x, y >= c_y) {
                (true, true)   => self.search_aux(c_node.b, Quadrant::NE, target, (c_x + offset, c_y + offset), path),
                (true, false)  => self.search_aux(c_node.d, Quadrant::SE, target, (c_x + offset, c_y - offset), path),
                (false, true)  => self.search_aux(c_node.a, Quadrant::NW, target, (c_x - offset, c_y + offset), path),
                (false, false) => self.search_aux(c_node.c, Quadrant::SW, target, (c_x - offset, c_y - offset), path)
            }
        }

        path.push((current, quadrant));
    }

    pub fn to_coords(&self) -> LinkedList<WCoord> {
        let mut points = list![];
        self.to_coords_aux(self.root, (0, 0), &mut points);
        points
    }

    fn to_coords_aux(
        &self,
        root: NodeId,
        (c_x, c_y): WCoord,
        points: &mut LinkedList<WCoord>
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
                let offset = 2_isize.pow((level - 2) as u32);
                self.to_coords_aux(r_node.a, (c_x - offset, c_y + offset), points);
                self.to_coords_aux(r_node.b, (c_x + offset, c_y + offset), points);
                self.to_coords_aux(r_node.c, (c_x - offset, c_y - offset), points);
                self.to_coords_aux(r_node.d, (c_x + offset, c_y - offset), points);
            }
        }
    }
}

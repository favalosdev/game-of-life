use rustc_hash::FxHashMap;
use std::collections::{LinkedList, HashSet};
use literal::list;
use ca_formats::rle::Rle;
use ca_formats::Input;
use std::cmp;
use crate::config::QT_DIM;

type NodeId = usize;

#[derive(Debug)]
struct Node {
    n: usize,
    k: usize,
    a: NodeId,
    b: NodeId,
    c: NodeId,
    d: NodeId
}

enum Quadrant {
    NW,
    NE,
    SW,
    SE
}

type Path = Vec<(NodeId, Quadrant)>;

const DEAD: usize = 0;
const ALIVE: usize = 1;
const VOID: usize = 2;

impl Node {
    fn new(
        n: usize, // Number of alive cells within the node
        k: usize, // Size of the quadtree (2**k x 2**k)
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

pub struct QuadTree {
    nodes: Vec<Node>,
    root: NodeId,
    b: Vec<usize>,
    s: Vec<usize>,
    caches: Caches
}

impl QuadTree {
    pub fn new() -> Self {
        let mut nodes = Vec::with_capacity(5_000_000);

        let void = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let dead = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let alive = Node::new(1, 0, VOID, VOID, VOID, VOID);

        nodes.push(dead);
        nodes.push(alive);
        nodes.push(void);

        let caches = Caches::new();

        QuadTree {
            nodes,
            root: DEAD,
            b: vec![3],
            s: vec![2],
            caches
        }
    }

    pub fn load_pattern<T: Input>(&mut self, pattern: Rle<T>) {
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

        let cells =  pattern
            .map(|cell| cell.unwrap())
            .filter(|data | data.state == 1)
            .map(|data| ((data.position.0 - (width as i64) / 2) as isize, ((height as i64) / 2 - data.position.1) as isize))
            .collect::<LinkedList<_>>();

        self.world_to_qt(cells);
    }

    pub fn world_to_qt(&mut self, cells: LinkedList<(isize, isize)>) {
        self.root = self.world_to_qt_aux(cells, (0,0), QT_DIM)
    }

    pub fn get_id(&self) -> NodeId {
        self.root
    }

    // Convert (x,y) to QuadTree
    fn world_to_qt_aux(
        &mut self,
        cells: LinkedList<(isize, isize)>,
        (c_x, c_y): (isize, isize),
        level: usize 
    ) -> NodeId {
        if cells.is_empty() {
            self.zero(level)
        } else if level == 1 {
            let lookup = cells.iter().collect::<HashSet<_>>();

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

                if p.0 >= c_x {
                    if p.1 >= c_y {
                        ne_cells.push_back(p);
                    } else {
                        se_cells.push_back(p);
                    }
                } else {
                    if p.1 >= c_y {
                        nw_cells.push_back(p);
                    } else {
                        sw_cells.push_back(p);
                    }
                }
            }

            let offset = 2_isize.pow((level - 2) as u32);
            let nw = self.world_to_qt_aux(nw_cells, (c_x - offset, c_y + offset), level - 1);
            let ne = self.world_to_qt_aux(ne_cells, (c_x + offset, c_y + offset), level - 1);
            let sw = self.world_to_qt_aux(sw_cells, (c_x - offset, c_y - offset), level - 1);
            let se = self.world_to_qt_aux(se_cells, (c_x + offset, c_y - offset), level - 1);
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

    pub fn cell_count(&self) -> usize {
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

    pub fn advance(&mut self, n: usize) {
        self.root = self.advance_aux(self.root, n);
    }

    fn advance_aux(&mut self, root: NodeId, mut n: usize) -> NodeId {
        if n == 0 {
            return root;
        }

        let mut nested = root;
        let mut index = 0;

        while n > 0 {
            if (n & 1) == 1 {
                nested = self.centre(nested);
                nested = self.successor(nested, Some(index));
            }

            n = n >> 1;
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

    pub fn toggle(&mut self, (x, y): (isize, isize)) {
    }

    // Returns a path from the parent node to the target
    fn search(&self, (x, y): (isize, isize)) -> Path {
        let mut path= Vec::with_capacity(QT_DIM + 1);
        // TODO: add some guardrails in here
        self.search_aux(self.root, None, (x, y), (0, 0), &mut path);
        path
    }

    // Returns parent node and relative position within it
    fn search_aux(
        &self,
        current: NodeId,
        quadrant: Option<Quadrant>,
        (x, y): (isize, isize),
        (c_x, c_y): (isize, isize),
        path: &mut Path
    ) {
        let c_node = &self.nodes[current];
        let level = c_node.k;

        if level > 0 {
            let offset = 2_isize.pow((level - 2) as u32);

            if x >= c_x && y >= c_y {
                self.search_aux(c_node.b, Some(Quadrant::NE), (x, y), (c_x + offset, c_y + offset), path)
            } else if x >= c_x && y < c_y {
                self.search_aux(c_node.d, Some(Quadrant::SE), (x, y), (c_x + offset, c_y - offset), path)
            } else if x < c_x && y >= c_y {
                self.search_aux(c_node.a, Some(Quadrant::NW), (x, y), (c_x - offset, c_y + offset), path)
            } else {
                self.search_aux(c_node.c, Some(Quadrant::SW), (x, y), (c_x - offset, c_y - offset), path)
            }

            if quadrant.is_some() {
                path.push((current, quadrant.unwrap()))
            }
        }
    }

    pub fn qt_to_world(&self) -> LinkedList<(isize, isize)> {
        let mut points = list![];
        self.qt_to_world_aux(self.root, (0, 0), &mut points);
        points
    }

    fn qt_to_world_aux(
        &self,
        root: NodeId,
        (c_x, c_y): (isize, isize),
        points: &mut LinkedList<(isize, isize)>
    ) {
        let r_node= &self.nodes[root];

        if r_node.n > 0 {
            if r_node.k == 0 {
                points.push_back((c_x, c_y));
            } else if r_node.k == 1 {
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
                let offset = 2_isize.pow((r_node.k - 2) as u32);
                self.qt_to_world_aux(r_node.a, (c_x - offset, c_y + offset), points);
                self.qt_to_world_aux(r_node.b, (c_x + offset, c_y + offset), points);
                self.qt_to_world_aux(r_node.c, (c_x - offset, c_y - offset), points);
                self.qt_to_world_aux(r_node.d, (c_x + offset, c_y - offset), points);
            }
        }
    }
}

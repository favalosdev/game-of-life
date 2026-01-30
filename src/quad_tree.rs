use std::collections::{LinkedList, HashSet, HashMap};
use literal::list;
use ca_formats::rle::Rle;
use ca_formats::Input;

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

const VOID: usize = 0;
const DEAD: usize = 1;
const ALIVE: usize = 2;

const SIZE: isize = (2 as i32).pow(10) as isize;

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
    join: HashMap<(NodeId, NodeId, NodeId, NodeId), NodeId>,
    life_4x4: HashMap<NodeId, NodeId>,
    next_gen: HashMap<NodeId, NodeId>,
    successor: HashMap<(NodeId, Option<usize>), NodeId>
}

impl Caches {
    fn new() -> Self {
        Caches {
            join: HashMap::new(),
            life_4x4: HashMap::new(),
            next_gen: HashMap::new(),
            successor: HashMap::new()
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
        let mut nodes = vec![];

        let void = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let dead = Node::new(0, 0, VOID, VOID, VOID, VOID);
        let alive = Node::new(1, 0, VOID, VOID, VOID, VOID);

        nodes.push(void);
        nodes.push(dead);
        nodes.push(alive);

        let caches = Caches::new();

        QuadTree { nodes, root: DEAD, b: vec![3], s: vec![2], caches }
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
            .map(|data| ((data.position.0 - (width as i64) / 2) as isize, -(data.position.1 - (height as i64) / 2) as isize))
            .collect::<LinkedList<_>>();

        self.world_to_qt(cells);
    }

    pub fn world_to_qt(&mut self, cells: LinkedList<(isize, isize)>) {
        self.root = self.world_to_qt_aux(cells, (0, 0), SIZE / 2);
    }

    pub fn get_id(&self) -> NodeId {
        self.root
    }

    // Convert (x,y) to QuadTree
    fn world_to_qt_aux(
        &mut self,
        cells: LinkedList<(isize, isize)>,
        centre: (isize, isize),
        span: isize
    ) -> NodeId {
        let (c_x, c_y) = centre;

        if span == 1 {
            let lookup = cells.iter().collect::<HashSet<_>>();

            let a_coords = (c_x-1, c_y);
            let b_coords = (c_x, c_y);
            let c_coords = (c_x-1, c_y-1);
            let d_coords = (c_x, c_y-1);

            let a = if lookup.contains(&a_coords) { ALIVE } else { DEAD };
            let b = if lookup.contains(&b_coords) { ALIVE } else { DEAD };
            let c = if lookup.contains(&c_coords) { ALIVE } else { DEAD };
            let d = if lookup.contains(&d_coords) { ALIVE } else { DEAD };

            return self.join(a, b, c, d);
        }

        let mut ne_cells = list![];
        let mut nw_cells = list![];
        let mut se_cells = list![];
        let mut sw_cells = list![];

        for (x, y) in cells.iter() {
            let (x,y) = (*x, *y);

            if x >= c_x {
                if y >= c_y {
                    ne_cells.push_back((x, y));
                } else {
                    se_cells.push_back((x, y));
                }
            } else {
                if y >= c_y {
                    nw_cells.push_back((x, y));
                } else {
                    sw_cells.push_back((x, y));
                }
            }
        }

        let new_span = span / 2;

        let nw = self.world_to_qt_aux(
            nw_cells,
            (c_x - new_span, c_y + new_span),
            new_span
        );

        let ne = self.world_to_qt_aux(
            ne_cells,
            (c_x + new_span, c_y + new_span),
            new_span
        );

        let sw = self.world_to_qt_aux(
            sw_cells,
            (c_x - new_span, c_y - new_span),
            new_span
        );

        let se = self.world_to_qt_aux(
            se_cells,
            (c_x + new_span, c_y - new_span),
            new_span
        );

        return self.join(nw, ne, sw, se);
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
        let to_add = Node::new(n, &self.nodes[d].k + 1, a, b, c, d);
        let result = self.new_node(to_add);
        self.caches.join.insert((a, b, c, d), result);
        result
    }

    fn get_zero(&mut self, k: usize) -> NodeId {
        if k == 0 {
            DEAD
        } else {
            let z= self.get_zero(k-1);
            self.join(z, z, z, z)
        }
    }

    fn centre(&mut self, m: NodeId) -> NodeId {
        let m_node = &self.nodes[m];
        let (ma, mb, mc, md) = (m_node.a, m_node.b, m_node.c, m_node.d);
        let z = self.get_zero(m_node.k - 1);
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

        if self.nodes[e].n == 1 && self.s.contains(&outer) || self.b.contains(&outer) {
            ALIVE
        } else {
            DEAD
        }
    }

    pub fn cell_count(&self) -> usize {
        self.nodes[self.root].n
    }

    fn life_4x4(&mut self, m: NodeId) -> NodeId {
        if let Some(id) = self.caches.life_4x4.get(&m) {
            return *id;
        }

        let m_node = &self.nodes[m];
        let a = &self.nodes[m_node.a];
        let b = &self.nodes[m_node.b];
        let c = &self.nodes[m_node.c];
        let d = &self.nodes[m_node.d];

        let ad = self.life(a.a, a.b, b.a, a.c, a.d, b.c, c.a, c.b, d.a);
        let bc = self.life(a.b, b.a, b.b, a.d, b.c, b.d, c.b, d.a, d.b);
        let cb = self.life(a.c, a.d, b.c, c.a, c.b, d.a, c.c, c.d, d.c);
        let da = self.life(a.d, b.c, b.d, c.b, d.a, d.b, c.d, d.c, d.d);

        let result = self.join(ad, bc, cb, da);

        self.caches.life_4x4.insert(m, result);
        result
    }

    pub fn advance(&mut self) {
        let nested = self.centre(self.root);
        self.root = self.next_gen(nested);
    }

    // No expontential speed-ups
    fn successor(&mut self, m: NodeId, j: Option<usize>) -> NodeId {
        if let Some(id) = self.caches.successor.get(&(m, j)) {
            return *id;
        }

        let m_node = &self.nodes[m];

        let next = if m_node.n == 0 {
            // empty
            m_node.a
        } else if m_node.k == 2 {
            // base case
            self.life_4x4(m)
        } else {
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

            let c1 = self.successor(j1, j);
            let c2 = self.successor(j2, j);
            let c3 = self.successor(j3, j);
            let c4 = self.successor(j4, j);
            let c5 = self.successor(j5, j);
            let c6 = self.successor(j6, j);
            let c7 = self.successor(j7, j);
            let c8 = self.successor(j8, j);
            let c9 = self.successor(j9, j);
            
            let s1 = self.join(self.nodes[c1].d, self.nodes[c2].c, self.nodes[c4].b, self.nodes[c5].a);
            let s2 = self.join(self.nodes[c2].d, self.nodes[c3].c, self.nodes[c5].b, self.nodes[c6].a);
            let s3 = self.join(self.nodes[c4].d, self.nodes[c5].c, self.nodes[c7].b, self.nodes[c8].a);
            let s4 = self.join(self.nodes[c5].d, self.nodes[c6].c, self.nodes[c8].b, self.nodes[c9].a);

            let ss1 = self.successor(s1, j);
            let ss2 = self.successor(s2, j);
            let ss3 = self.successor(s3, j);
            let ss4 = self.successor(s4, j);

            self.join(ss1, ss2, ss3, ss4)
        };
        self.caches.successor.insert((m, j), next);
        next
    }

    // No expontential speed-ups
    fn next_gen(&mut self, m: NodeId) -> NodeId {
        if let Some(id) = self.caches.next_gen.get(&m) {
            return *id;
        }

        let m_node = &self.nodes[m];

        let next = if m_node.n == 0 {
            // empty
            m_node.a
        } else if m_node.k == 2 {
            // base case
            self.life_4x4(m)
        } else {
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

            let c1 = self.next_gen(j1);
            let c2 = self.next_gen(j2);
            let c3 = self.next_gen(j3);
            let c4 = self.next_gen(j4);
            let c5 = self.next_gen(j5);
            let c6 = self.next_gen(j6);
            let c7 = self.next_gen(j7);
            let c8 = self.next_gen(j8);
            let c9 = self.next_gen(j9);
            
            let s1 = self.join(self.nodes[c1].d, self.nodes[c2].c, self.nodes[c4].b, self.nodes[c5].a);
            let s2 = self.join(self.nodes[c2].d, self.nodes[c3].c, self.nodes[c5].b, self.nodes[c6].a);
            let s3 = self.join(self.nodes[c4].d, self.nodes[c5].c, self.nodes[c7].b, self.nodes[c8].a);
            let s4 = self.join(self.nodes[c5].d, self.nodes[c6].c, self.nodes[c8].b, self.nodes[c9].a);

            self.join(s1, s2, s3, s4)
        };
        self.caches.next_gen.insert(m, next);
        next
    }

    pub fn qt_to_world(&self) -> LinkedList<(isize, isize)> {
        let span = ((2 as u32).pow(self.nodes[self.root].k as u32) / 2) as isize;
        self.qt_to_world_aux(self.root, (0, 0), span)
    }

    fn qt_to_world_aux(
        &self,
        root: NodeId,
        centre: (isize, isize),
        span: isize
    ) -> LinkedList<(isize, isize)> {
        let top = &self.nodes[root];
        let (c_x, c_y) = centre;

        if top.k == 0 || top.n == 0 {
            list![]
        } else if top.k == 1 {
            let mut points = list![];

            let a_coords = (c_x-1, c_y);
            let b_coords = (c_x, c_y);
            let c_coords = (c_x-1, c_y-1);
            let d_coords = (c_x, c_y-1);

            if top.a == ALIVE {
                points.push_back(a_coords);
            }

            if top.b == ALIVE {
                points.push_back(b_coords);
            } 

            if top.c == ALIVE {
                points.push_back(c_coords);
            }

            if top.d == ALIVE {
                points.push_back(d_coords);
            }

            points
        } else {
            let mut points = list![];
            let new_span = span / 2;

            let nw_centre = (c_x - new_span, c_y + new_span);
            let ne_centre = (c_x + new_span, c_y + new_span);
            let sw_centre = (c_x - new_span, c_y - new_span);
            let se_centre = (c_x + new_span, c_y - new_span);

            if self.nodes[top.a].n > 0 {
                let mut nw = self.qt_to_world_aux(top.a, nw_centre, new_span);
                points.append(&mut nw);
            }

            if self.nodes[top.b].n > 0 {
                let mut ne = self.qt_to_world_aux(top.b, ne_centre, new_span);
                points.append(&mut ne);
            }

            if self.nodes[top.c].n > 0 {
                let mut sw = self.qt_to_world_aux(top.c, sw_centre, new_span);
                points.append(&mut sw);
            }

            if self.nodes[top.d].n > 0 {
                let mut se = self.qt_to_world_aux(top.d, se_centre, new_span);
                points.append(&mut se);
            }
         
            points
        }
    }
}

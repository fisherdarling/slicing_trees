use std::{fmt, hint::unreachable_unchecked};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Rect {
    width: usize,
    height: usize,
}

impl Rect {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn rotate(&mut self) {
        std::mem::swap(&mut self.width, &mut self.height);
    }

    pub fn cut(self, cut: Cut) -> (Self, Self) {
        match cut {
            Cut::Horizontal => {
                let r = Rect::new(self.width, self.height / 2);
                (r, r)
            }
            Cut::Vertical => {
                let r = Rect::new(self.width / 2, self.height);
                (r, r)
            }
        }
    }

    pub fn aabb(left: Rect, right: Rect, cut: Cut) -> Rect {
        use std::cmp::max;

        match cut {
            Cut::Vertical => Rect::new(left.width + right.width, max(left.height, right.height)),
            Cut::Horizontal => Rect::new(max(left.width, right.width), left.height + right.height),
        }
    }

    pub fn cost(&self) -> f32 {
        (self.width * self.height) as f32
    }
}

impl fmt::Debug for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.width, self.height)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Cut {
    Horizontal,
    Vertical,
}

impl Cut {
    pub fn opposite(&self) -> Cut {
        match self {
            Cut::Horizontal => Cut::Vertical,
            Cut::Vertical => Cut::Horizontal,
        }
    }
}

impl fmt::Debug for Cut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Cut::Vertical => "V",
                Cut::Horizontal => "H",
            }
        )
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Node {
    cut: Option<Cut>,
    rect: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}

impl Node {
    pub fn new(
        cut: Option<Cut>,
        rect: Option<usize>,
        left: Option<usize>,
        right: Option<usize>,
        parent: Option<usize>,
    ) -> Self {
        Self {
            cut,
            rect,
            left,
            right,
            parent,
        }
    }

    pub fn root() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn left_child_idx(&self) -> Option<usize> {
        self.left
    }

    pub fn right_child_idx(&self) -> Option<usize> {
        self.right
    }

    pub fn goto_left(&self, tree: &SlicingTree) -> Option<Node> {
        let idx = self.left?;
        tree.nodes.get(idx).copied()
    }

    pub fn goto_right(&self, tree: &SlicingTree) -> Option<Node> {
        let idx = self.right?;
        tree.nodes.get(idx).copied()
    }
}

pub struct SlicingTree {
    pub data: Vec<Rect>,
    pub nodes: Vec<Node>,
}

impl SlicingTree {
    pub fn new(starting_width: usize, starting_height: usize) -> Self {
        let start_rect = Rect::new(starting_width, starting_height);
        let data = vec![start_rect];

        let mut root = Node::root();
        root.rect = Some(0);

        Self {
            data,
            nodes: vec![root],
        }
    }

    pub fn random_tree(width: usize, height: usize, num_cuts: usize) -> SlicingTree {
        // Perform a random traversal num cuts number of times.
        // When the traversal hits a leaf rect, stop and randomly
        // cut the rect.

        let mut tree = SlicingTree::new(width, height);

        for _ in 0..num_cuts {
            let mut current = 0;

            while tree.nodes[current].rect.is_none() {
                if rand::random() {
                    current = tree.nodes[current].left.unwrap();
                } else {
                    current = tree.nodes[current].right.unwrap();
                }
            }

            // Get the cut, always opposite to the parent (guarantee skewed trees)
            let cut = if let Some(parent) = tree.nodes[current].parent {
                tree.nodes[parent].cut.unwrap().opposite()
            } else {
                if rand::random() {
                    Cut::Vertical
                } else {
                    Cut::Horizontal
                }
            };

            let rect_idx = tree.nodes[current].rect.unwrap();
            let rect = tree.data[rect_idx];

            let (left, right) = rect.cut(cut);
            // let left_rect = tree.push_rect(left);
            tree.data[rect_idx] = left;
            let left_rect = rect_idx;
            let right_rect = tree.push_rect(right);

            let new_left = Node::new(None, Some(left_rect), None, None, Some(current));
            let left_idx = tree.push_node(new_left);

            let new_right = Node::new(None, Some(right_rect), None, None, Some(current));
            let right_idx = tree.push_node(new_right);

            let mut new_parent = &mut tree.nodes[current];
            new_parent.rect = None;
            new_parent.cut = Some(cut);
            new_parent.left = Some(left_idx);
            new_parent.right = Some(right_idx);
        }

        tree
    }

    pub fn push_rect(&mut self, rect: Rect) -> usize {
        self.data.push(rect);
        self.data.len() - 1
    }

    pub fn push_node(&mut self, node: Node) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn postorder(&self) -> NPE {
        let mut data = Vec::new();
        self.postorder_rec(0, &mut data);
        NPE::new(data)
    }

    fn postorder_rec(&self, root: usize, data: &mut Vec<TreeItem>) {
        let root_node = self.nodes[root];
        if let Some(rect) = root_node.rect {
            data.push(TreeItem::Rect(rect));
            return;
        }

        self.postorder_rec(root_node.left.unwrap(), data);
        self.postorder_rec(root_node.right.unwrap(), data);

        if let Some(cut) = root_node.cut {
            data.push(TreeItem::Cut(cut));
        }
    }

    pub fn aabb(&self, root: usize) -> Rect {
        let node = self.nodes[root];
        if let Some(rect) = node.rect {
            return self.data[rect];
        }

        let left = self.aabb(node.left.unwrap());
        let right = self.aabb(node.right.unwrap());

        let cut = node.cut.unwrap();

        Rect::aabb(left, right, cut)
    }

    pub fn print_as_problem(&self) {
        println!("{}", self.data.len());
        for rect in &self.data {
            println!("{} {}", rect.width, rect.height);
        }

        let postorder = self.postorder();
        // println!("Problem Postorder: {:?}", postorder);

        for item in 0..postorder.expr.len() - 1 {
            print!("{:?} ", postorder.expr[item]);
        }

        println!("{:?}", postorder.expr[postorder.expr.len() - 1]);
    }
}

#[derive(Default, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct NPE {
    pub expr: Vec<TreeItem>,
    pub ballot: Vec<(usize, usize)>,
}

impl NPE {
    pub fn new(expr: Vec<TreeItem>) -> Self {
        let mut new = NPE::default();
        new.ballot = vec![(0, 0); expr.len()];
        new.expr = expr;
        new.calculate_ballot();

        new
    }

    pub fn calculate_ballot(&mut self) {
        if self.expr[0].is_cut() {
            self.ballot[0] = (0, 1);
        } else {
            self.ballot[0] = (1, 0);
        }

        for i in 1..self.expr.len() {
            self.ballot[i] = self.ballot[i - 1];

            if self.expr[i].is_rect() {
                self.ballot[i].0 += 1;
            } else {
                self.ballot[i].1 += 1;
            }
        }
    }

    pub fn create_problem(&mut self, iterations: usize, tree: &SlicingTree) {
        println!("{}", tree.data.len());
        for rect in &tree.data {
            println!("{} {}", rect.width, rect.height);
        }
        // println!("Problem Postorder: {:?}", postorder);

        self.perturb(iterations);

        for item in 0..self.expr.len() - 1 {
            print!("{:?} ", self.expr[item]);
        }

        println!("{:?}", self.expr[self.expr.len() - 1]);
    }
    
    pub fn aabb(&self, rects: &[Rect]) -> Rect {
        let mut operands = Vec::with_capacity(rects.len());
        // let mut operators = Vec::with_capacity(rects.len() / 2);

        for e in &self.expr {
            match e {
                TreeItem::Rect(i) => operands.push(rects[*i]),
                TreeItem::Cut(cut) => {
                    let right = operands.pop().unwrap();
                    let left = operands.pop().unwrap();

                    operands.push(Rect::aabb(left, right, *cut))
                }
            }
        }
        // while let Some(e) in sel

        operands[0]
    }

    pub fn perturb(&mut self, iterations: usize) {
        use rand::{seq::SliceRandom, thread_rng, Rng};
        static CHOICES: &[u8] = &[1, 2, 3];
        let mut rng = thread_rng();

        let mut num_chains = self.chains().count();
        // let num_operands = self.count_operands();
        let num_operators = self.count_operators();

        for _ in 0..iterations {
            match *CHOICES.choose(&mut rng).unwrap() {
                // M1
                1 => {
                    // println!("1");
                    let a: usize = rng.gen_range(0, num_operators - 1);
                    self.m1(a);
                }
                // M2
                2 => {
                    // println!("2");
                    let n = rng.gen_range(0, num_chains);
                    // println!("{} chain", n);
                    self.m2(n);
                    // let (a, b) = self.chains().nth(n).unwrap();
                }
                // M3
                3 => {
                    // println!("3");
                    self.m3();
                    num_chains = self.chains().count();
                }
                _ => unreachable!(),
            }
        }
    }

    pub fn m1(&mut self, a: usize) {
        let mut iter =
            self.expr
                .iter()
                .enumerate()
                .filter_map(|(i, b)| if b.is_rect() { Some(i) } else { None });

        let a_idx = iter.nth(a).unwrap();
        let b_idx = iter.next().unwrap();

        // println!("M1: {} <-> {}", a_idx, b_idx);

        self.swap(a_idx, b_idx);
    }

    pub fn m2(&mut self, n: usize) {
        let (a, b) = self.chains().nth(n).unwrap();
        
        let chain = &mut self.expr[a..b];
        
        // println!("M2: ({}, {}) {:?}", a, b, chain);

        for e in chain {
            match e {
                TreeItem::Cut(cut) => {
                    *cut = cut.opposite();
                }
                _ => {}
            }
        }
    }

    // pub fn print_npe(&self) {
    //     for item in 0..self.expr.len() - 1 {
    //         print!("{:?} ", self.expr[item]);
    //     }

    //     println!("{:?}", self.expr[self.expr.len() - 1]);
    // }

    pub fn m3(&mut self) {
        use rand::{seq::SliceRandom, thread_rng};

        let mut windows: Vec<_> = self
            .expr
            .windows(2)
            .enumerate()
            .filter_map(|(i, s)| {
                match s {
                    &[a, b] => if a.is_rect() && b.is_cut() || a.is_cut() && b.is_rect() {
                        Some(i)
                    } else {
                        None
                    }, // TODO: Figure out: 
                    _ => None,
                }
            })
            .collect();
        
        windows.shuffle(&mut thread_rng());

        for i in windows {
            if self.is_swap_normalized(i, i + 1) {
                // println!("M3: ({}, {}) {:?} <-> {:?}", i, i+1, self.expr[i], self.expr[i + 1]);
                self.swap(i, i + 1);

                if !self.is_normalized(i.saturating_sub(1), i + 2) {
                    self.swap(i, i + 1);
                } else {
                    self.calculate_ballot();
                    break;
                }
            }
        }
    }

    pub fn chains(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let mut start = 0;
        let mut stop = 0;

        let mut iter = self.expr.iter().enumerate();

        std::iter::from_fn(move || {
            while let Some((i, item)) = iter.next() {
                if item.is_cut() {
                    stop += 1;
                } else if stop - start > 0 {
                    let a = start;
                    let b = stop;

                    start = i;
                    stop = i;

                    return Some((a + 1, b + 1));
                } else {
                    start = i;
                    stop = i;
                }
            }

            if stop - start > 0 {
                let a = start;
                let b = stop;

                start = 0;
                stop = 0;

                Some((a + 1, b + 1))
            } else {
                None
            }
        })
    }

    pub fn nth_chain(&mut self, n: usize) -> &mut [TreeItem] {
        let mut start = 0;
        let mut stop = 0;
        let mut count = 0;

        for (i, item) in self.expr.iter().enumerate() {
            if item.is_cut() {
                stop += 1;
            } else if stop - start > 0 {
                count += 1;

                if count == n {
                    return &mut self.expr[start..stop];
                }

                start = i;
                stop = i;
            }
        }

        todo!()
    }

    pub fn number_chains(&self) -> usize {
        let mut start = 0;
        let mut stop = 0;
        let mut count = 0;

        for (i, item) in self.expr.iter().enumerate() {
            if item.is_cut() {
                stop += 1;
            } else if stop - start > 0 {
                count += 1;

                start = i;
                stop = i;
            }
        }

        if stop - start > 0 {
            count += 1;
        }

        count
    }

    fn satisfies_ballot(&self, a: usize, b: usize) -> bool {
        2 * self.ballot[b].1 < a
    }

    pub fn is_normalized(&self, a: usize, b: usize) -> bool {
        self.expr[a..=b].windows(2).all(|w| match w {
            &[a, b] => {
                a != b
            }
            _ => true,
        })
    }

    pub fn is_swap_normalized(&self, a: usize, b: usize) -> bool {
        self.satisfies_ballot(a, b) && self.is_normalized(a.saturating_sub(1), b.saturating_add(1))

        // println!("{} < {}", 2 * self.number_operators(b), a);
        // if !(2 * self.number_operators(b) < a) {
        //     return false;
        // }

        // 2 * self.ballot[b].1 < a

        // let (operand, operator) = if self.expr[a].is_cut() {
        //     (b, a)
        // } else { 
        //     (a, b)
        // };

        // false
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.expr.swap(a, b);
    }

    pub fn number_operators(&self, k: usize) -> usize {
        self.expr[0..k]
            .iter()
            .copied()
            .filter(TreeItem::is_cut)
            .count()
    }

    fn count_operators(&self) -> usize {
        self.expr.iter().copied().filter(TreeItem::is_cut).count()
    }

    pub fn count_operands(&self) -> usize {
        self.expr.iter().copied().filter(TreeItem::is_rect).count()
    }

    // pub fn is_normalized(&self) -> bool {
    //     for i in 0..self.expr.len() - 1 {
    //         match (self.expr[i], self.expr[i + 1]) {
    //             (TreeItem::Cut(a), TreeItem::Cut(b)) => {
    //                 if a.opposite() != b {
    //                     return false;
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }

    //     true
    // }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TreeItem {
    Rect(usize),
    Cut(Cut),
}

impl TreeItem {
    pub fn is_rect(&self) -> bool {
        match self {
            TreeItem::Rect(_) => true,
            _ => false,
        }
    }

    pub fn is_cut(&self) -> bool {
        match self {
            TreeItem::Cut(_) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for TreeItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeItem::Rect(idx) => idx.fmt(f),
            TreeItem::Cut(cut) => cut.fmt(f),
        }
    }
}

impl fmt::Display for NPE {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in 0..self.expr.len() - 1 {
            write!(f, "{:?} ", self.expr[item])?;
        }

        write!(f, "{:?}", self.expr[self.expr.len() - 1])
    }
}

//! What the tests of the graph kernel and of the functions on it share (release 0.10.0, steps
//! 12 and 13): a graph as a second way sees it, with separation decided by walking every path;
//! the same description made into a graph of the kernel; and a small generator of draws that
//! is the same on every machine.
#![allow(dead_code)]

use zeff_core::dag::{Dag, Edge, Link};
use zeff_identify::graph::Graph;

/// A graph as the second way sees it: arrows over numbered nodes, the common cause of each
/// bidirected edge added as a node after the written ones, and the nodes the data are selected
/// on. Built from the same description as the kernel's graph, by its own code.
pub struct Plain {
    pub n: usize,
    pub arrows: Vec<(usize, usize)>,
    pub selected: Vec<usize>,
}

impl Plain {
    pub fn new(
        written: usize,
        directed: &[(usize, usize)],
        bidirected: &[(usize, usize)],
        selected: &[usize],
    ) -> Plain {
        let mut arrows = directed.to_vec();
        for (k, &(a, b)) in bidirected.iter().enumerate() {
            arrows.push((written + k, a));
            arrows.push((written + k, b));
        }
        Plain {
            n: written + bidirected.len(),
            arrows,
            selected: selected.to_vec(),
        }
    }

    pub fn has(&self, from: usize, to: usize) -> bool {
        self.arrows.contains(&(from, to))
    }

    /// `reaches[a][b]`: a directed path of any length, none among them, leads from a to b.
    pub fn reaches(&self) -> Vec<Vec<bool>> {
        let mut r = vec![vec![false; self.n]; self.n];
        for (i, row) in r.iter_mut().enumerate() {
            row[i] = true;
        }
        for &(a, b) in &self.arrows {
            r[a][b] = true;
        }
        for k in 0..self.n {
            for i in 0..self.n {
                for j in 0..self.n {
                    if r[i][k] && r[k][j] {
                        r[i][j] = true;
                    }
                }
            }
        }
        r
    }

    /// Whether some path from `x` to `y` is open given `given` and the selected nodes: every
    /// path that meets no node twice is walked, and a path is open when each node inside it
    /// lets it through: a collider (both neighbours point into it) when it or one of its
    /// descendants is given, any other node when it is not given.
    pub fn connected(&self, x: usize, y: usize, given: &[bool], reaches: &[Vec<bool>]) -> bool {
        fn walk(
            g: &Plain,
            path: &mut Vec<usize>,
            y: usize,
            given: &[bool],
            reaches: &[Vec<bool>],
        ) -> bool {
            let here = path[path.len() - 1];
            if here == y {
                return true;
            }
            for next in 0..g.n {
                if path.contains(&next) || !(g.has(here, next) || g.has(next, here)) {
                    continue;
                }
                // The node `here` lies inside the path when it has a node before it.
                if path.len() >= 2 {
                    let before = path[path.len() - 2];
                    let collider = g.has(before, here) && g.has(next, here);
                    let lets_through = if collider {
                        (0..g.n).any(|d| given[d] && reaches[here][d])
                    } else {
                        !given[here]
                    };
                    if !lets_through {
                        continue;
                    }
                }
                path.push(next);
                if walk(g, path, y, given, reaches) {
                    return true;
                }
                path.pop();
            }
            false
        }
        walk(self, &mut vec![x], y, given, reaches)
    }

    /// Every open path from `x` to `y` given the nodes marked, as its nodes in order: every
    /// path that meets no node twice is walked to its end and then asked, node by node,
    /// whether each inner node lets it through.
    pub fn open_paths(&self, x: usize, y: usize, given: &[bool]) -> Vec<Vec<usize>> {
        fn walk(g: &Plain, path: &mut Vec<usize>, y: usize, all: &mut Vec<Vec<usize>>) {
            let here = path[path.len() - 1];
            if here == y {
                all.push(path.clone());
                return;
            }
            for next in 0..g.n {
                if !path.contains(&next) && (g.has(here, next) || g.has(next, here)) {
                    path.push(next);
                    walk(g, path, y, all);
                    path.pop();
                }
            }
        }
        let reaches = self.reaches();
        let mut all = Vec::new();
        walk(self, &mut vec![x], y, &mut all);
        all.retain(|path| {
            (1..path.len() - 1).all(|i| {
                let (before, here, after) = (path[i - 1], path[i], path[i + 1]);
                if self.has(before, here) && self.has(after, here) {
                    (0..self.n).any(|d| given[d] && reaches[here][d])
                } else {
                    !given[here]
                }
            })
        });
        all
    }

    /// Whether some path from `x` to `y` that is not causal is open given the nodes marked:
    /// every path that meets no node twice is walked; one made of arrows that all point onward
    /// is causal and is left alone; any other is open when each node inside it lets it through,
    /// a collider when it or one of its descendants is given, any other node when it is not.
    pub fn open_path_not_causal(
        &self,
        x: usize,
        y: usize,
        given: &[bool],
        reaches: &[Vec<bool>],
    ) -> bool {
        fn walk(
            g: &Plain,
            path: &mut Vec<usize>,
            onward: bool,
            y: usize,
            given: &[bool],
            reaches: &[Vec<bool>],
        ) -> bool {
            let here = path[path.len() - 1];
            if here == y {
                return !onward;
            }
            for next in 0..g.n {
                if path.contains(&next) || !(g.has(here, next) || g.has(next, here)) {
                    continue;
                }
                if path.len() >= 2 {
                    let before = path[path.len() - 2];
                    let collider = g.has(before, here) && g.has(next, here);
                    let lets_through = if collider {
                        (0..g.n).any(|d| given[d] && reaches[here][d])
                    } else {
                        !given[here]
                    };
                    if !lets_through {
                        continue;
                    }
                }
                path.push(next);
                if walk(g, path, onward && g.has(here, next), y, given, reaches) {
                    return true;
                }
                path.pop();
            }
            false
        }
        walk(self, &mut vec![x], true, y, given, reaches)
    }

    /// Whether `set`, with the nodes the data are selected on, gives the effect of `x` on `y`
    /// by the generalized adjustment criterion as it is defined: none of them lies on a causal
    /// path from `x` to `y` or below a node that does, and every path between the two that is
    /// not causal is blocked.
    pub fn adjusts(&self, x: usize, y: usize, set: &[usize]) -> bool {
        let reaches = self.reaches();
        let causal = self.proper_causal(&[x], &[y]);
        let mut given = vec![false; self.n];
        for &v in set.iter().chain(&self.selected) {
            given[v] = true;
        }
        let forbidden = |v: usize| (0..self.n).any(|c| causal[c] && reaches[c][v]);
        !(0..self.n).any(|v| given[v] && forbidden(v))
            && !self.open_path_not_causal(x, y, &given, &reaches)
    }

    /// Whether the sets are separated: no node of one, not given, has an open path to a node
    /// of the other, not given; two sets that share such a node are not separated.
    pub fn separated(&self, x: &[usize], y: &[usize], given: &[usize]) -> bool {
        let mut mark = vec![false; self.n];
        for &v in given.iter().chain(&self.selected) {
            mark[v] = true;
        }
        let reaches = self.reaches();
        for &a in x.iter().filter(|&&a| !mark[a]) {
            for &b in y.iter().filter(|&&b| !mark[b]) {
                if a == b || self.connected(a, b, &mark, &reaches) {
                    return false;
                }
            }
        }
        true
    }

    /// The nodes on a proper causal path from the exposure to the outcome, the exposure left
    /// out: every directed path from a node of the exposure is walked, stopped where it would
    /// enter the exposure again, and the nodes of each one that ends in the outcome are kept
    /// up to that end.
    pub fn proper_causal(&self, exposure: &[usize], outcome: &[usize]) -> Vec<bool> {
        fn walk(
            g: &Plain,
            path: &mut Vec<usize>,
            exposure: &[usize],
            outcome: &[usize],
            on: &mut [bool],
        ) {
            let here = path[path.len() - 1];
            if path.len() > 1 && outcome.contains(&here) {
                for &node in &path[1..] {
                    on[node] = true;
                }
            }
            for next in 0..g.n {
                if g.has(here, next) && !exposure.contains(&next) && !path.contains(&next) {
                    path.push(next);
                    walk(g, path, exposure, outcome, on);
                    path.pop();
                }
            }
        }
        let mut on = vec![false; self.n];
        for &start in exposure {
            walk(self, &mut vec![start], exposure, outcome, &mut on);
        }
        on
    }
}

/// The same description as a graph of the kernel, through the value a `dag { }` block makes.
pub fn kernel(
    written: usize,
    directed: &[(usize, usize)],
    bidirected: &[(usize, usize)],
    latent: &[usize],
    selected: &[usize],
) -> Graph {
    let nodes: Vec<String> = (0..written).map(|i| format!("v{i}")).collect();
    let mut edges: Vec<Edge> = directed
        .iter()
        .map(|&(from, to)| Edge {
            from,
            to,
            link: Link::Directed,
        })
        .collect();
    // The bidirected edges after the directed ones, in order: the nodes added for them are
    // numbered in the order they are met.
    edges.extend(bidirected.iter().map(|&(from, to)| Edge {
        from,
        to,
        link: Link::Bidirected,
    }));
    Graph::of(&Dag::new(nodes, edges, latent.to_vec(), selected.to_vec()))
}

pub fn marks(set: &[usize], n: usize) -> Vec<bool> {
    (0..n).map(|v| set.contains(&v)).collect()
}

pub fn acyclic(n: usize, arrows: &[(usize, usize)]) -> bool {
    let mut left = vec![true; n];
    loop {
        let free = (0..n).find(|&v| left[v] && !arrows.iter().any(|&(a, b)| left[a] && b == v));
        match free {
            Some(v) => left[v] = false,
            None => return !left.iter().any(|&l| l),
        }
    }
}

/// A small generator of its own, so that the 2,000 graphs are the same on every machine and
/// in every year.
pub struct Draws(pub u64);

impl Draws {
    pub fn next(&mut self) -> u64 {
        // Marsaglia's xorshift, 64 bits.
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    pub fn chance(&mut self, in_a_hundred: u64) -> bool {
        self.next() % 100 < in_a_hundred
    }
}

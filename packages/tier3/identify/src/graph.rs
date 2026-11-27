//! The graph kernel (Language Reference, section 7.3; release 0.10.0, step 12): what the
//! functions of a causal graph ask of it. Ancestors and descendants; the nodes on the proper
//! causal paths from an exposure to an outcome; and d-separation, by which a graph says that
//! two sets of variables are independent given a third.
//!
//! A graph is first prepared ([`Graph::of`]). A bidirected edge `a <-> b` stands for a common
//! cause that is not in the graph: it becomes a node of its own, latent, with an arrow to each
//! of the two. A node declared `selected` is one the data are conditioned on by design: it is
//! added to the conditioning set of every question of separation. An undirected edge is for
//! drawing only and takes no part; a caller that cannot answer in its presence asks
//! [`Graph::has_undirected`] and refuses.
//!
//! d-separation is decided on the moral graph of the ancestors (Lauritzen, Dawid, Larsen and
//! Leimer 1990): take the nodes asked about, those given and all their ancestors; join every
//! two parents of a common child and drop the directions; take away the nodes given; the two
//! sets are separated when no path is left between them. Nothing here depends on the order in
//! which a graph was written but the order of what is listed.

use zeff_core::dag::{Dag, Link};

/// A causal graph prepared for questions of paths and separation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    /// The names: the nodes of the graph as written, then one node for each bidirected edge.
    names: Vec<String>,
    /// How many of them were written; the rest stand for the common causes of bidirected edges.
    written: usize,
    parents: Vec<Vec<usize>>,
    children: Vec<Vec<usize>>,
    latent: Vec<bool>,
    selected: Vec<bool>,
    undirected: bool,
}

impl Graph {
    /// The graph of a `dag { }` block, prepared. The node added for `a <-> b` is called
    /// `a <-> b`, a name no written node can have without backticks around it in every use.
    pub fn of(dag: &Dag) -> Graph {
        let written = dag.nodes().len();
        let mut names: Vec<String> = dag.nodes().to_vec();
        let mut parents: Vec<Vec<usize>> = vec![Vec::new(); written];
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); written];
        let mut latent = vec![false; written];
        let mut selected = vec![false; written];
        for &at in dag.latent() {
            latent[at] = true;
        }
        for &at in dag.selected() {
            selected[at] = true;
        }
        let mut undirected = false;
        for edge in dag.edges() {
            match edge.link {
                Link::Directed => {
                    children[edge.from].push(edge.to);
                    parents[edge.to].push(edge.from);
                }
                Link::Bidirected => {
                    let cause = names.len();
                    names.push(format!("{} <-> {}", names[edge.from], names[edge.to]));
                    parents.push(Vec::new());
                    children.push(vec![edge.from, edge.to]);
                    latent.push(true);
                    selected.push(false);
                    parents[edge.from].push(cause);
                    parents[edge.to].push(cause);
                }
                Link::Undirected => undirected = true,
            }
        }
        Graph {
            names,
            written,
            parents,
            children,
            latent,
            selected,
            undirected,
        }
    }

    /// The number of nodes, those added for bidirected edges among them.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// The number of nodes that were written; they come first.
    pub fn written(&self) -> usize {
        self.written
    }

    pub fn name(&self, node: usize) -> &str {
        &self.names[node]
    }

    /// The place of the written node called `name`.
    pub fn node(&self, name: &str) -> Option<usize> {
        self.names[..self.written].iter().position(|n| n == name)
    }

    /// Whether a node is not observed: declared `latent`, or added for a bidirected edge.
    pub fn is_latent(&self, node: usize) -> bool {
        self.latent[node]
    }

    /// Whether the data are selected on a node.
    pub fn is_selected(&self, node: usize) -> bool {
        self.selected[node]
    }

    /// Whether the graph has an undirected edge, which takes no part in any answer here.
    pub fn has_undirected(&self) -> bool {
        self.undirected
    }

    pub fn parents(&self, node: usize) -> &[usize] {
        &self.parents[node]
    }

    pub fn children(&self, node: usize) -> &[usize] {
        &self.children[node]
    }

    /// The nodes reached from `from` along `step`, `from` among them, as a mark for each node.
    fn reach(&self, from: &[usize], step: &[Vec<usize>], closed: &[bool]) -> Vec<bool> {
        let mut seen = vec![false; self.len()];
        let mut todo: Vec<usize> = Vec::new();
        for &node in from {
            if !seen[node] {
                seen[node] = true;
                todo.push(node);
            }
        }
        while let Some(node) = todo.pop() {
            for &next in &step[node] {
                if !seen[next] && !closed[next] {
                    seen[next] = true;
                    todo.push(next);
                }
            }
        }
        seen
    }

    /// The ancestors of `of`, the nodes themselves among them: a mark for each node.
    pub fn ancestors(&self, of: &[usize]) -> Vec<bool> {
        self.reach(of, &self.parents, &vec![false; self.len()])
    }

    /// The descendants of `of`, the nodes themselves among them: a mark for each node.
    pub fn descendants(&self, of: &[usize]) -> Vec<bool> {
        self.reach(of, &self.children, &vec![false; self.len()])
    }

    /// The nodes that lie on a proper causal path from `exposure` to `outcome`, the exposure
    /// itself left out: a directed path from a node of the exposure to a node of the outcome
    /// that meets the exposure at its start alone. They are the descendants of the exposure
    /// reached without passing through another node of it, that are also ancestors of the
    /// outcome reached without passing through the exposure.
    pub fn proper_causal_nodes(&self, exposure: &[usize], outcome: &[usize]) -> Vec<bool> {
        let mut is_exposure = vec![false; self.len()];
        for &node in exposure {
            is_exposure[node] = true;
        }
        // Forward from the exposure, never into the exposure again.
        let forward = self.reach(exposure, &self.children, &is_exposure);
        // Backward from the outcome, never into the exposure.
        let outcome: Vec<usize> = outcome
            .iter()
            .copied()
            .filter(|&n| !is_exposure[n])
            .collect();
        let backward = self.reach(&outcome, &self.parents, &is_exposure);
        (0..self.len())
            .map(|node| !is_exposure[node] && forward[node] && backward[node])
            .collect()
    }

    /// Whether `x` and `y` are d-separated given `given` and the nodes the data are selected
    /// on. A node of `x` or of `y` that is also given takes no part; two sets that share a node
    /// that is not given are not separated.
    pub fn d_separated(&self, x: &[usize], y: &[usize], given: &[usize]) -> bool {
        let n = self.len();
        let mut conditioned = self.selected.clone();
        for &node in given {
            conditioned[node] = true;
        }
        let x: Vec<usize> = x.iter().copied().filter(|&v| !conditioned[v]).collect();
        let y: Vec<usize> = y.iter().copied().filter(|&v| !conditioned[v]).collect();
        if x.iter().any(|v| y.contains(v)) {
            return false;
        }
        // The ancestral set of everything asked about and everything given.
        let mut about: Vec<usize> = x.clone();
        about.extend(&y);
        about.extend((0..n).filter(|&v| conditioned[v]));
        let within = self.ancestors(&about);
        // Its moral graph: a child and its parents joined, and its parents joined to one another.
        let mut joined: Vec<Vec<usize>> = vec![Vec::new(); n];
        for child in (0..n).filter(|&v| within[v]) {
            let parents = &self.parents[child];
            for (i, &p) in parents.iter().enumerate() {
                joined[child].push(p);
                joined[p].push(child);
                for &q in &parents[i + 1..] {
                    joined[p].push(q);
                    joined[q].push(p);
                }
            }
        }
        // Without the nodes given, is any node of `y` reached from `x`?
        let reached = self.reach(&x, &joined, &conditioned);
        !y.iter().any(|&v| reached[v])
    }

    /// The same graph without the arrows listed, each as its two nodes.
    fn without_arrows(&self, removed: &[(usize, usize)]) -> Graph {
        let mut graph = self.clone();
        for &(from, to) in removed {
            graph.children[from].retain(|&child| child != to);
            graph.parents[to].retain(|&parent| parent != from);
        }
        graph
    }

    /// The same graph with no node selected on: what it would say of data that were not
    /// selected, by which a difficulty is laid to the selection or not.
    pub fn without_selection(&self) -> Graph {
        let mut graph = self.clone();
        graph.selected = vec![false; self.len()];
        graph
    }

    /// The proper back-door graph of an exposure and an outcome (van der Zander, Liśkiewicz and
    /// Textor 2014): the graph without the first arrow of every proper causal path from the
    /// exposure to the outcome. What still joins the two in it is no part of the effect.
    pub fn proper_back_door(&self, exposure: &[usize], outcome: &[usize]) -> Graph {
        let causal = self.proper_causal_nodes(exposure, outcome);
        let first: Vec<(usize, usize)> = exposure
            .iter()
            .flat_map(|&x| {
                self.children[x]
                    .iter()
                    .filter(|&&child| causal[child])
                    .map(move |&child| (x, child))
            })
            .collect();
        self.without_arrows(&first)
    }

    /// The nodes no adjustment set may hold: those on a proper causal path from the exposure
    /// to the outcome, the exposure left out, and their descendants. To hold one fixed takes
    /// away part of the effect, or opens a way between the two that is no effect.
    pub fn forbidden(&self, exposure: &[usize], outcome: &[usize]) -> Vec<bool> {
        let causal = self.proper_causal_nodes(exposure, outcome);
        let on: Vec<usize> = (0..self.len()).filter(|&v| causal[v]).collect();
        self.descendants(&on)
    }

    /// Whether adjusting for `set` gives the total effect of the exposure on the outcome, by
    /// the generalized adjustment criterion (Shpitser, VanderWeele and Robins 2010, in the
    /// form of van der Zander, Liśkiewicz and Textor 2014): the set and the nodes the data are
    /// selected on hold no forbidden node, and they separate the exposure from the outcome in
    /// the proper back-door graph. Whether the nodes of the set are observed is the caller's
    /// business.
    pub fn is_adjustment_set(&self, exposure: &[usize], outcome: &[usize], set: &[usize]) -> bool {
        let forbidden = self.forbidden(exposure, outcome);
        if set.iter().any(|&v| forbidden[v])
            || (0..self.len()).any(|v| self.selected[v] && forbidden[v])
        {
            return false;
        }
        self.proper_back_door(exposure, outcome)
            .d_separated(exposure, outcome, set)
    }
}

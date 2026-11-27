//! A causal graph as a value (Language Reference, sections 3.8 and 7.3; catalogue row 429;
//! release 0.10.0, step 11): what a `dag { }` block states, kept as its author wrote it. The
//! nodes are in the order first written; the edges in the order first written, each once;
//! the nodes declared `latent` and those declared `selected` in the order declared.
//!
//! The checker has already refused what is no graph of causes (a directed cycle, an edge from
//! a node to itself, a node both latent and selected), so a value of this type is always one.
//! Two graphs are the same graph when they have the same nodes, the same edges and the same
//! declarations, whatever order their lines were written in. A graph prints as a block that
//! reads back as the same graph: one edge to a line, then the nodes that have no edge, then
//! the declarations.

use std::fmt;

/// The three links a graph may hold between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Link {
    /// `a -> b`: a direct causal effect of the first on the second.
    Directed,
    /// `a <-> b`: an unobserved common cause of the two.
    Bidirected,
    /// `a -- b`: a link for drawing only.
    Undirected,
}

impl Link {
    /// The sign as a block writes it.
    pub fn sign(self) -> &'static str {
        match self {
            Link::Directed => "->",
            Link::Bidirected => "<->",
            Link::Undirected => "--",
        }
    }
}

/// One edge: the places of its two nodes among the graph's nodes, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub link: Link,
}

/// A causal graph.
#[derive(Debug, Clone)]
pub struct Dag {
    nodes: Vec<String>,
    edges: Vec<Edge>,
    latent: Vec<usize>,
    selected: Vec<usize>,
}

impl Dag {
    /// A graph from its parts, as the checker left them: every place is a node, no edge joins
    /// a node to itself, no edge is written twice, and the directed edges have no cycle.
    pub fn new(
        nodes: Vec<String>,
        edges: Vec<Edge>,
        latent: Vec<usize>,
        selected: Vec<usize>,
    ) -> Dag {
        Dag {
            nodes,
            edges,
            latent,
            selected,
        }
    }

    /// The names of the nodes, in the order first written.
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// The edges, in the order first written.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// The places of the nodes declared `latent`.
    pub fn latent(&self) -> &[usize] {
        &self.latent
    }

    /// The places of the nodes declared `selected`.
    pub fn selected(&self) -> &[usize] {
        &self.selected
    }

    /// The place of the node called `name`.
    pub fn node(&self, name: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n == name)
    }

    /// The graph in a form that does not depend on the order of its lines: the names sorted,
    /// each edge by the names of its ends (the two ends of a link without a direction in
    /// order), the edges sorted, and the two declarations by name.
    #[allow(clippy::type_complexity)]
    fn canonical(&self) -> (Vec<&str>, Vec<(Link, &str, &str)>, Vec<&str>, Vec<&str>) {
        let name = |at: usize| self.nodes[at].as_str();
        let mut nodes: Vec<&str> = self.nodes.iter().map(String::as_str).collect();
        nodes.sort_unstable();
        let mut edges: Vec<(Link, &str, &str)> = self
            .edges
            .iter()
            .map(|edge| {
                let (a, b) = (name(edge.from), name(edge.to));
                if edge.link != Link::Directed && b < a {
                    (edge.link, b, a)
                } else {
                    (edge.link, a, b)
                }
            })
            .collect();
        edges.sort_unstable();
        edges.dedup();
        let named = |places: &[usize]| {
            let mut names: Vec<&str> = places.iter().map(|&at| name(at)).collect();
            names.sort_unstable();
            names.dedup();
            names
        };
        (nodes, edges, named(&self.latent), named(&self.selected))
    }
}

impl PartialEq for Dag {
    fn eq(&self, other: &Dag) -> bool {
        self.canonical() == other.canonical()
    }
}

/// The words a name cannot be written as without backticks: the reserved words of section 3.1
/// of the Language Reference.
const RESERVED: [&str; 26] = [
    "let", "fn", "if", "else", "case", "for", "in", "break", "continue", "return", "use", "model",
    "dag", "as", "true", "false", "NA", "Inf", "type", "while", "match", "mut", "pub", "loop",
    "trait", "impl",
];

/// The reserved words, for the test that holds them to the lexer's own list.
pub fn reserved_words() -> &'static [&'static str] {
    &RESERVED
}

/// A name as a script writes it: bare when it is letters, digits and `_` starting with a letter
/// or `_` and is no reserved word, and in backticks otherwise.
pub fn written_name(name: &str) -> String {
    let mut characters = name.chars();
    let plain = match characters.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {
            characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    };
    if plain && name != "_" && !RESERVED.contains(&name) {
        name.to_string()
    } else {
        format!("`{name}`")
    }
}

impl fmt::Display for Dag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.nodes.is_empty() {
            return write!(f, "dag {{ }}");
        }
        let name = |at: usize| written_name(&self.nodes[at]);
        writeln!(f, "dag {{")?;
        for edge in &self.edges {
            writeln!(
                f,
                "    {} {} {}",
                name(edge.from),
                edge.link.sign(),
                name(edge.to)
            )?;
        }
        // A node with no edge is a line of its own. One that is declared needs no such line:
        // its declaration names it.
        for at in 0..self.nodes.len() {
            let joined = self.edges.iter().any(|e| e.from == at || e.to == at);
            let declared = self.latent.contains(&at) || self.selected.contains(&at);
            if !joined && !declared {
                writeln!(f, "    {}", name(at))?;
            }
        }
        for (word, places) in [("latent", &self.latent), ("selected", &self.selected)] {
            if !places.is_empty() {
                let names: Vec<String> = places.iter().map(|&at| name(at)).collect();
                writeln!(f, "    {word} {}", names.join(", "))?;
            }
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(nodes: &[&str], edges: &[(usize, usize, Link)], latent: &[usize]) -> Dag {
        Dag::new(
            nodes.iter().map(|n| (*n).to_string()).collect(),
            edges
                .iter()
                .map(|&(from, to, link)| Edge { from, to, link })
                .collect(),
            latent.to_vec(),
            Vec::new(),
        )
    }

    #[test]
    fn a_graph_prints_as_a_block_one_edge_to_a_line() {
        let g = graph(
            &["ability", "training", "productivity", "alone"],
            &[
                (0, 1, Link::Directed),
                (1, 2, Link::Directed),
                (0, 2, Link::Bidirected),
                (1, 2, Link::Undirected),
            ],
            &[0],
        );
        assert_eq!(
            g.to_string(),
            "dag {\n    ability -> training\n    training -> productivity\n    ability <-> productivity\n    training -- productivity\n    alone\n    latent ability\n}"
        );
        assert_eq!(graph(&[], &[], &[]).to_string(), "dag { }");
    }

    #[test]
    fn two_graphs_are_equal_whatever_the_order_of_their_lines() {
        let a = graph(
            &["x", "y", "z"],
            &[(0, 1, Link::Directed), (2, 1, Link::Bidirected)],
            &[2],
        );
        // The same graph with its nodes met in another order and its common cause written
        // the other way round.
        let b = graph(
            &["z", "y", "x"],
            &[(1, 0, Link::Bidirected), (2, 1, Link::Directed)],
            &[0],
        );
        assert_eq!(a, b);
        // A directed edge the other way round is another graph; so is one node more, a link
        // of another kind, and a declaration less.
        assert_ne!(
            a,
            graph(
                &["x", "y", "z"],
                &[(1, 0, Link::Directed), (2, 1, Link::Bidirected)],
                &[2]
            )
        );
        assert_ne!(
            a,
            graph(
                &["x", "y", "z", "w"],
                &[(0, 1, Link::Directed), (2, 1, Link::Bidirected)],
                &[2]
            )
        );
        assert_ne!(
            a,
            graph(
                &["x", "y", "z"],
                &[(0, 1, Link::Directed), (2, 1, Link::Undirected)],
                &[2]
            )
        );
        assert_ne!(
            a,
            graph(
                &["x", "y", "z"],
                &[(0, 1, Link::Directed), (2, 1, Link::Bidirected)],
                &[]
            )
        );
    }

    #[test]
    fn a_name_is_written_bare_or_in_backticks() {
        assert_eq!(written_name("educ"), "educ");
        assert_eq!(written_name("_x9"), "_x9");
        assert_eq!(written_name("gross sales"), "`gross sales`");
        assert_eq!(written_name("9lives"), "`9lives`");
        assert_eq!(written_name("for"), "`for`");
        assert_eq!(written_name("NA"), "`NA`");
        assert_eq!(written_name("_"), "`_`");
        assert_eq!(written_name("latent"), "latent");
    }
}

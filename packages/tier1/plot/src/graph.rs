//! The graph form (Language Reference, section 7.6, "Diagrams"; open decision #191; release
//! 0.10.0, step 19): what every diagram is before it is laid out. A causal graph, and from
//! later releases a flowchart, a sample flow and the path diagram of a structural equation
//! model, are each first lowered to this one form: nodes, edges, groups and ports. The layout
//! reads this form and nothing else, and the scene of the figure keeps it beside what was
//! drawn, so that a viewer finds the node or the edge under the pointer, a table view lists
//! the same structure a sighted reader sees, and the text exports are written from it once.
//!
//! Every node and every edge carries a stable element name, the name its primitives carry in
//! a drawing: `node:tenure`, `edge:training->tenure`. A name is made from the names the
//! script gave and from nothing else, so the same graph has the same names in every run.
//!
//! The form says what a thing is, never how it is drawn: a node is latent, not dashed; an
//! edge is a common cause, not an arc. The kinds of this release are those a causal graph
//! needs; a later kind of diagram adds kinds and changes nothing here. The form is a section
//! of the scene's JSON with a version of its own ([`GRAPH_VERSION`]), which rises when a
//! reader of the earlier version would misread the new one. Beside what the diagram is, the
//! form holds how the script asks for it to be placed ([`Placing`]): the way the layers
//! follow, the nodes to keep on one layer, the gaps, or a place for every node.

use zeff_core::dag::{Dag, Edge as DagEdge, Link};

use crate::json::Json;

/// The version of the graph form, the value of `version` in its JSON.
pub const GRAPH_VERSION: u32 = 1;

/// What a node is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A variable that is measured.
    Observed,
    /// A variable that is not measured.
    Latent,
}

impl NodeKind {
    pub const ALL: [NodeKind; 2] = [NodeKind::Observed, NodeKind::Latent];

    pub fn name(self) -> &'static str {
        match self {
            NodeKind::Observed => "observed",
            NodeKind::Latent => "latent",
        }
    }

    pub fn from_name(name: &str) -> Option<NodeKind> {
        NodeKind::ALL.into_iter().find(|kind| kind.name() == name)
    }
}

/// The outline a node is drawn in. The form holds it because the layout must know how much
/// room a node takes before anything is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    Box,
    Ellipse,
}

impl NodeShape {
    pub const ALL: [NodeShape; 2] = [NodeShape::Box, NodeShape::Ellipse];

    pub fn name(self) -> &'static str {
        match self {
            NodeShape::Box => "box",
            NodeShape::Ellipse => "ellipse",
        }
    }

    pub fn from_name(name: &str) -> Option<NodeShape> {
        NodeShape::ALL
            .into_iter()
            .find(|shape| shape.name() == name)
    }
}

/// The part a node plays in the question the figure is drawn for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Exposure,
    Outcome,
}

impl Role {
    pub const ALL: [Role; 2] = [Role::Exposure, Role::Outcome];

    pub fn name(self) -> &'static str {
        match self {
            Role::Exposure => "exposure",
            Role::Outcome => "outcome",
        }
    }

    pub fn from_name(name: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|role| role.name() == name)
    }
}

/// A side of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Top, Side::Right, Side::Bottom, Side::Left];

    pub fn name(self) -> &'static str {
        match self {
            Side::Top => "top",
            Side::Right => "right",
            Side::Bottom => "bottom",
            Side::Left => "left",
        }
    }

    pub fn from_name(name: &str) -> Option<Side> {
        Side::ALL.into_iter().find(|side| side.name() == name)
    }
}

/// A named place on the border of a node where an edge may end. A causal graph has none: its
/// edges end wherever the layout brings them to the border.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    pub id: String,
    pub side: Side,
}

/// A node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The name the script gave it, which is unique among the nodes.
    pub id: String,
    pub kind: NodeKind,
    pub shape: NodeShape,
    /// The text drawn in it, a line to an item.
    pub label: Vec<String>,
    /// Whether the data are selected on it.
    pub selected: bool,
    pub role: Option<Role>,
    pub ports: Vec<Port>,
}

impl Node {
    /// The element name: `node:tenure`.
    pub fn name(&self) -> String {
        format!("node:{}", self.id)
    }
}

/// What an edge says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// The first acts on the second: `a -> b`.
    Path,
    /// A common cause of the two that is not in the graph: `a <-> b`.
    CommonCause,
    /// A link that says nothing of cause, for drawing only: `a -- b`.
    Link,
}

impl EdgeKind {
    pub const ALL: [EdgeKind; 3] = [EdgeKind::Path, EdgeKind::CommonCause, EdgeKind::Link];

    pub fn name(self) -> &'static str {
        match self {
            EdgeKind::Path => "path",
            EdgeKind::CommonCause => "common_cause",
            EdgeKind::Link => "link",
        }
    }

    pub fn from_name(name: &str) -> Option<EdgeKind> {
        EdgeKind::ALL.into_iter().find(|kind| kind.name() == name)
    }

    /// The sign between the two names in an element name and in a `dag { }` block.
    pub fn sign(self) -> &'static str {
        match self {
            EdgeKind::Path => "->",
            EdgeKind::CommonCause => "<->",
            EdgeKind::Link => "--",
        }
    }

    /// Whether an arrow head is drawn at the first end, and at the second.
    pub fn heads(self) -> (bool, bool) {
        match self {
            EdgeKind::Path => (false, true),
            EdgeKind::CommonCause => (true, true),
            EdgeKind::Link => (false, false),
        }
    }
}

/// An end of an edge: a node, by its place among the nodes, and the port of it the edge ends
/// on, where the diagram names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct End {
    pub node: usize,
    pub port: Option<String>,
}

/// An edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: End,
    pub to: End,
    pub kind: EdgeKind,
    /// The text drawn beside it, a line to an item; a causal graph draws none.
    pub label: Vec<String>,
}

/// Nodes drawn together, under a label where there is one. A causal graph has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub id: String,
    pub label: Vec<String>,
    /// The nodes of the group, by their places.
    pub members: Vec<usize>,
}

impl Group {
    /// The element name: `group:measurement`.
    pub fn name(&self) -> String {
        format!("group:{}", self.id)
    }
}

/// Which way the layers of a diagram follow one another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// The first layer at the top.
    #[default]
    Down,
    /// The first layer at the left.
    Right,
}

impl Direction {
    pub const ALL: [Direction; 2] = [Direction::Down, Direction::Right];

    pub fn name(self) -> &'static str {
        match self {
            Direction::Down => "down",
            Direction::Right => "right",
        }
    }

    pub fn from_name(name: &str) -> Option<Direction> {
        Direction::ALL
            .into_iter()
            .find(|direction| direction.name() == name)
    }
}

/// How the script asks for the diagram to be placed. Where it asks for nothing, the layout
/// places it: in layers, down the page.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Placing {
    pub direction: Direction,
    /// Sets of nodes, by their places, that are to stand on one layer.
    pub same_rank: Vec<Vec<usize>>,
    /// The room between two layers and between two neighbours on a layer, in millimetres,
    /// where the script gives them; otherwise they follow from the size of the type.
    pub spacing: Option<(f64, f64)>,
    /// The place of each node on a grid, as its column and its row counted from 0, a part
    /// of a step allowed, where the script places the nodes itself; the layout then places
    /// none and `direction` and `same_rank` are not used.
    pub at: Option<Vec<(f64, f64)>>,
}

/// A diagram as the layout reads it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphForm {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
    pub placing: Placing,
}

/// What is wrong with a graph form that cannot be what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// Two nodes of one name.
    NodeTwice(String),
    /// An edge, or a group, that names a node the form does not have.
    NoSuchNode(usize),
    /// An edge that ends on a port its node does not have.
    NoSuchPort(String, String),
    /// Two groups of one name.
    GroupTwice(String),
    /// A form that is no causal graph: it has a port, a group or an edge with a label.
    NotACausalGraph(&'static str),
    /// A placing that does not fit the form: places for another number of nodes than it has,
    /// a place or a gap that is no number, or a gap that is not above zero.
    BadPlacing(&'static str),
}

impl GraphForm {
    /// The form of a causal graph: a node for each node, in the order of the graph, an
    /// observed one as a box and a latent one as an ellipse, as path diagrams draw them; an
    /// edge for each edge, in the order of the graph.
    pub fn of_dag(dag: &Dag) -> GraphForm {
        let nodes = dag
            .nodes()
            .iter()
            .enumerate()
            .map(|(at, id)| {
                let latent = dag.latent().contains(&at);
                Node {
                    id: id.clone(),
                    kind: if latent {
                        NodeKind::Latent
                    } else {
                        NodeKind::Observed
                    },
                    shape: if latent {
                        NodeShape::Ellipse
                    } else {
                        NodeShape::Box
                    },
                    label: vec![id.clone()],
                    selected: dag.selected().contains(&at),
                    role: None,
                    ports: Vec::new(),
                }
            })
            .collect();
        let edges = dag
            .edges()
            .iter()
            .map(|edge| Edge {
                from: End {
                    node: edge.from,
                    port: None,
                },
                to: End {
                    node: edge.to,
                    port: None,
                },
                kind: match edge.link {
                    Link::Directed => EdgeKind::Path,
                    Link::Bidirected => EdgeKind::CommonCause,
                    Link::Undirected => EdgeKind::Link,
                },
                label: Vec::new(),
            })
            .collect();
        GraphForm {
            nodes,
            edges,
            groups: Vec::new(),
            placing: Placing::default(),
        }
    }

    /// The causal graph this form is the form of: the way back from [`GraphForm::of_dag`],
    /// with nothing lost. The parts a node plays in a figure are no part of the graph and are
    /// left behind; a form with a port, a group or a label on an edge is no causal graph.
    pub fn to_dag(&self) -> Result<Dag, Fault> {
        self.check()?;
        if self.nodes.iter().any(|node| !node.ports.is_empty()) {
            return Err(Fault::NotACausalGraph("a port"));
        }
        if !self.groups.is_empty() {
            return Err(Fault::NotACausalGraph("a group"));
        }
        if self.edges.iter().any(|edge| !edge.label.is_empty()) {
            return Err(Fault::NotACausalGraph("an edge with a label"));
        }
        let places = |keep: &dyn Fn(&Node) -> bool| -> Vec<usize> {
            self.nodes
                .iter()
                .enumerate()
                .filter(|(_, node)| keep(node))
                .map(|(at, _)| at)
                .collect()
        };
        Ok(Dag::new(
            self.nodes.iter().map(|node| node.id.clone()).collect(),
            self.edges
                .iter()
                .map(|edge| DagEdge {
                    from: edge.from.node,
                    to: edge.to.node,
                    link: match edge.kind {
                        EdgeKind::Path => Link::Directed,
                        EdgeKind::CommonCause => Link::Bidirected,
                        EdgeKind::Link => Link::Undirected,
                    },
                })
                .collect(),
            places(&|node| node.kind == NodeKind::Latent),
            places(&|node| node.selected),
        ))
    }

    /// The element name of an edge: the names of its two nodes with the sign of its kind
    /// between them, `edge:training->tenure`, `edge:price<->demand`. A second edge of the
    /// same kind between the same two nodes, which a causal graph cannot have, is told apart
    /// by its number among them: `edge:a->b#2`.
    pub fn edge_name(&self, at: usize) -> String {
        let edge = &self.edges[at];
        let same = |other: &Edge| {
            other.kind == edge.kind
                && other.from.node == edge.from.node
                && other.to.node == edge.to.node
        };
        let before = self.edges[..at].iter().filter(|other| same(other)).count();
        let base = format!(
            "edge:{}{}{}",
            self.nodes[edge.from.node].id,
            edge.kind.sign(),
            self.nodes[edge.to.node].id
        );
        if before == 0 {
            base
        } else {
            format!("{base}#{}", before + 1)
        }
    }

    /// Whether the form is what it says: each node named once, each group named once, every
    /// edge and every group naming nodes the form has, every port named on an edge a port of
    /// its node.
    pub fn check(&self) -> Result<(), Fault> {
        for (at, node) in self.nodes.iter().enumerate() {
            if self.nodes[..at].iter().any(|other| other.id == node.id) {
                return Err(Fault::NodeTwice(node.id.clone()));
            }
        }
        for (at, group) in self.groups.iter().enumerate() {
            if self.groups[..at].iter().any(|other| other.id == group.id) {
                return Err(Fault::GroupTwice(group.id.clone()));
            }
            if let Some(&member) = group.members.iter().find(|&&m| m >= self.nodes.len()) {
                return Err(Fault::NoSuchNode(member));
            }
        }
        for edge in &self.edges {
            for end in [&edge.from, &edge.to] {
                let Some(node) = self.nodes.get(end.node) else {
                    return Err(Fault::NoSuchNode(end.node));
                };
                if let Some(port) = &end.port
                    && !node.ports.iter().any(|p| &p.id == port)
                {
                    return Err(Fault::NoSuchPort(node.id.clone(), port.clone()));
                }
            }
        }
        for set in &self.placing.same_rank {
            if let Some(&member) = set.iter().find(|&&m| m >= self.nodes.len()) {
                return Err(Fault::NoSuchNode(member));
            }
        }
        if let Some((layers, nodes)) = self.placing.spacing
            && !(layers.is_finite() && nodes.is_finite() && layers > 0.0 && nodes > 0.0)
        {
            return Err(Fault::BadPlacing("a gap that is not a length above zero"));
        }
        if let Some(at) = &self.placing.at {
            if at.len() != self.nodes.len() {
                return Err(Fault::BadPlacing("places for another number of nodes"));
            }
            if at.iter().any(|(x, y)| !(x.is_finite() && y.is_finite())) {
                return Err(Fault::BadPlacing("a place that is no number"));
            }
        }
        Ok(())
    }

    /// The form as its JSON value, the keys in a fixed order: the section `graph` of a
    /// scene's JSON.
    pub fn json(&self) -> Json {
        let lines = |text: &[String]| Json::texts(text);
        let end = |end: &End| {
            Json::Object(vec![
                ("node", Json::whole(end.node)),
                ("port", end.port.as_ref().map_or(Json::Null, Json::text)),
            ])
        };
        Json::Object(vec![
            ("version", Json::Number(f64::from(GRAPH_VERSION))),
            (
                "nodes",
                Json::List(
                    self.nodes
                        .iter()
                        .map(|node| {
                            Json::Object(vec![
                                ("name", Json::text(node.name())),
                                ("id", Json::text(node.id.as_str())),
                                ("kind", Json::text(node.kind.name())),
                                ("shape", Json::text(node.shape.name())),
                                ("label", lines(&node.label)),
                                ("selected", Json::Bool(node.selected)),
                                (
                                    "role",
                                    node.role.map_or(Json::Null, |role| Json::text(role.name())),
                                ),
                                (
                                    "ports",
                                    Json::List(
                                        node.ports
                                            .iter()
                                            .map(|port| {
                                                Json::Object(vec![
                                                    ("id", Json::text(port.id.as_str())),
                                                    ("side", Json::text(port.side.name())),
                                                ])
                                            })
                                            .collect(),
                                    ),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "edges",
                Json::List(
                    self.edges
                        .iter()
                        .enumerate()
                        .map(|(at, edge)| {
                            let (first, second) = edge.kind.heads();
                            Json::Object(vec![
                                ("name", Json::text(self.edge_name(at))),
                                ("from", end(&edge.from)),
                                ("to", end(&edge.to)),
                                ("kind", Json::text(edge.kind.name())),
                                (
                                    "heads",
                                    Json::List(vec![Json::Bool(first), Json::Bool(second)]),
                                ),
                                ("label", lines(&edge.label)),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "groups",
                Json::List(
                    self.groups
                        .iter()
                        .map(|group| {
                            Json::Object(vec![
                                ("name", Json::text(group.name())),
                                ("id", Json::text(group.id.as_str())),
                                ("label", lines(&group.label)),
                                (
                                    "members",
                                    Json::List(
                                        group.members.iter().map(|&m| Json::whole(m)).collect(),
                                    ),
                                ),
                            ])
                        })
                        .collect(),
                ),
            ),
            (
                "placing",
                Json::Object(vec![
                    ("direction", Json::text(self.placing.direction.name())),
                    (
                        "same_rank",
                        Json::List(
                            self.placing
                                .same_rank
                                .iter()
                                .map(|set| {
                                    Json::List(set.iter().map(|&m| Json::whole(m)).collect())
                                })
                                .collect(),
                        ),
                    ),
                    (
                        "spacing",
                        self.placing.spacing.map_or(Json::Null, |(layers, nodes)| {
                            Json::numbers(&[layers, nodes])
                        }),
                    ),
                    (
                        "at",
                        self.placing.at.as_ref().map_or(Json::Null, |at| {
                            Json::List(at.iter().map(|&(x, y)| Json::numbers(&[x, y])).collect())
                        }),
                    ),
                ]),
            ),
        ])
    }

    /// The line a margin and a list of figures show for the diagram: `6 nodes, 7 edges`.
    pub fn summary(&self) -> String {
        let counted = |n: usize, one: &str, many: &str| {
            format!(
                "{} {}",
                zeff_diagnostics::count(n as u64),
                if n == 1 { one } else { many }
            )
        };
        format!(
            "{}, {}",
            counted(self.nodes.len(), "node", "nodes"),
            counted(self.edges.len(), "edge", "edges")
        )
    }
}

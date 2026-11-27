//! The graph form (release 0.10.0, step 19; open decision #191; Language Reference, section
//! 7.6, "Diagrams"): a causal graph lowered to the one form every diagram has and brought back
//! with nothing lost; the element names; the section `graph` of a scene's JSON, read back by
//! a reader written here; what a form that is not what it says is refused for; and the
//! version a scene with a diagram is written as.

mod common;

use common::*;
use proptest::prelude::*;
use serde_json::Value as Read;
use zeff_core::dag::{Dag, Edge as DagEdge, Link};
use zeff_core::value::{Aesthetic, Plot};
use zeff_plot::graph::{
    Edge, EdgeKind, End, Fault, GRAPH_VERSION, GraphForm, Group, Node, NodeKind, NodeShape, Port,
    Role, Side,
};
use zeff_plot::scene::{start, with_layer};

/// The graph of section 7.3 of the Language Reference.
fn training() -> Dag {
    let names = [
        "ability",
        "training",
        "productivity",
        "tenure",
        "age",
        "region",
    ];
    let arrows = [(0, 1), (1, 2), (0, 2), (1, 3), (3, 2), (4, 1), (5, 1)];
    Dag::new(
        names.iter().map(|n| (*n).to_string()).collect(),
        arrows
            .iter()
            .map(|&(from, to)| DagEdge {
                from,
                to,
                link: Link::Directed,
            })
            .collect(),
        vec![0],
        Vec::new(),
    )
}

/// A graph with each kind of link, a selected node, a latent one, a node alone and a name
/// that is not a plain one.
fn mixed() -> Dag {
    let names = [
        "price", "demand", "season", "weather", "ad spend", "listed", "taste",
    ];
    let edges = [
        (0, 1, Link::Bidirected),
        (1, 2, Link::Undirected),
        (4, 1, Link::Directed),
        (6, 1, Link::Directed),
    ];
    Dag::new(
        names.iter().map(|n| (*n).to_string()).collect(),
        edges
            .iter()
            .map(|&(from, to, link)| DagEdge { from, to, link })
            .collect(),
        vec![6],
        vec![5],
    )
}

/// The form a JSON text holds, by a reader that shares nothing with the writer: the section
/// as any program outside the engine would read it.
fn read(json: &Read) -> GraphForm {
    let text = |v: &Read| v.as_str().unwrap().to_string();
    let lines = |v: &Read| -> Vec<String> { v.as_array().unwrap().iter().map(text).collect() };
    let end = |v: &Read| End {
        node: v["node"].as_u64().unwrap() as usize,
        port: v["port"].as_str().map(String::from),
    };
    assert_eq!(json["version"].as_u64(), Some(u64::from(GRAPH_VERSION)));
    GraphForm {
        nodes: json["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| Node {
                id: text(&n["id"]),
                kind: NodeKind::from_name(n["kind"].as_str().unwrap()).unwrap(),
                shape: NodeShape::from_name(n["shape"].as_str().unwrap()).unwrap(),
                label: lines(&n["label"]),
                selected: n["selected"].as_bool().unwrap(),
                role: n["role"].as_str().map(|r| Role::from_name(r).unwrap()),
                ports: n["ports"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| Port {
                        id: text(&p["id"]),
                        side: Side::from_name(p["side"].as_str().unwrap()).unwrap(),
                    })
                    .collect(),
            })
            .collect(),
        edges: json["edges"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| Edge {
                from: end(&e["from"]),
                to: end(&e["to"]),
                kind: EdgeKind::from_name(e["kind"].as_str().unwrap()).unwrap(),
                label: lines(&e["label"]),
            })
            .collect(),
        groups: json["groups"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| Group {
                id: text(&g["id"]),
                label: lines(&g["label"]),
                members: g["members"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| m.as_u64().unwrap() as usize)
                    .collect(),
            })
            .collect(),
    }
}

fn parsed(form: &GraphForm) -> Read {
    serde_json::from_str(&form.json().write()).unwrap()
}

#[test]
fn the_graph_of_section_7_3_lowers_to_the_form_and_comes_back() {
    let dag = training();
    let form = GraphForm::of_dag(&dag);
    assert_eq!(form.check(), Ok(()));
    assert_eq!(
        (form.nodes.len(), form.edges.len(), form.groups.len()),
        (6, 7, 0)
    );
    assert_eq!(form.summary(), "6 nodes, 7 edges");
    // The nodes in the order of the graph, each with its name as its label; the latent one
    // an ellipse and the observed ones boxes, as path diagrams draw them.
    let names: Vec<String> = form.nodes.iter().map(Node::name).collect();
    assert_eq!(
        names,
        [
            "node:ability",
            "node:training",
            "node:productivity",
            "node:tenure",
            "node:age",
            "node:region"
        ]
    );
    assert_eq!(
        (form.nodes[0].kind, form.nodes[0].shape),
        (NodeKind::Latent, NodeShape::Ellipse)
    );
    for node in &form.nodes[1..] {
        assert_eq!(
            (node.kind, node.shape),
            (NodeKind::Observed, NodeShape::Box)
        );
        assert_eq!(node.label, vec![node.id.clone()]);
        assert!(!node.selected && node.role.is_none() && node.ports.is_empty());
    }
    // The edges in the order of the graph, each named by its two nodes and its sign.
    let edges: Vec<String> = (0..form.edges.len()).map(|at| form.edge_name(at)).collect();
    assert_eq!(
        edges,
        [
            "edge:ability->training",
            "edge:training->productivity",
            "edge:ability->productivity",
            "edge:training->tenure",
            "edge:tenure->productivity",
            "edge:age->training",
            "edge:region->training"
        ]
    );
    // And back: the same graph, to the print.
    let back = form.to_dag().unwrap();
    assert_eq!(back, dag);
    assert_eq!(back.to_string(), dag.to_string());
    insta::assert_snapshot!("graph_form_training", form.json().write());
}

#[test]
fn every_kind_of_link_a_selected_node_and_a_name_with_a_space() {
    let dag = mixed();
    let form = GraphForm::of_dag(&dag);
    assert_eq!(form.check(), Ok(()));
    let edges: Vec<String> = (0..form.edges.len()).map(|at| form.edge_name(at)).collect();
    assert_eq!(
        edges,
        [
            "edge:price<->demand",
            "edge:demand--season",
            "edge:ad spend->demand",
            "edge:taste->demand"
        ]
    );
    assert_eq!(
        form.edges
            .iter()
            .map(|e| e.kind.heads())
            .collect::<Vec<_>>(),
        [(true, true), (false, false), (false, true), (false, true)]
    );
    let listed = form.nodes.iter().find(|n| n.id == "listed").unwrap();
    assert!(listed.selected && listed.kind == NodeKind::Observed);
    assert_eq!(
        form.nodes
            .iter()
            .filter(|n| n.kind == NodeKind::Latent)
            .count(),
        1
    );
    assert_eq!(form.to_dag().unwrap(), dag);
    assert_eq!(form.to_dag().unwrap().to_string(), dag.to_string());
    // A part played in a figure is no part of the graph: marked, the form is still the form
    // of the same graph.
    let mut marked = form.clone();
    marked.nodes[0].role = Some(Role::Exposure);
    marked.nodes[1].role = Some(Role::Outcome);
    assert_eq!(marked.to_dag().unwrap(), dag);
    assert_ne!(marked, form);
    assert_eq!(read(&parsed(&marked)), marked);
    // The empty graph.
    let empty = Dag::new(Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let form = GraphForm::of_dag(&empty);
    assert_eq!(form, GraphForm::default());
    assert_eq!(form.to_dag().unwrap(), empty);
    assert_eq!(form.summary(), "0 nodes, 0 edges");
    insta::assert_snapshot!(
        "graph_form_mixed",
        GraphForm::of_dag(&mixed()).json().write()
    );
}

#[test]
fn the_section_reads_back_with_ports_groups_and_labels() {
    // What a causal graph has none of, and a later kind of diagram will have: ports, a
    // group, a label on an edge, two edges of one kind between the same nodes.
    let node = |id: &str, ports: Vec<Port>| Node {
        id: id.to_string(),
        kind: NodeKind::Observed,
        shape: NodeShape::Box,
        label: vec![id.to_string(), "second line".to_string()],
        selected: false,
        role: None,
        ports,
    };
    let port = |id: &str, side: Side| Port {
        id: id.to_string(),
        side,
    };
    let form = GraphForm {
        nodes: vec![
            node(
                "engage",
                vec![port("out", Side::Right), port("in", Side::Left)],
            ),
            node("perform", vec![port("in", Side::Top)]),
            node("quote \"this\"\\", Vec::new()),
        ],
        edges: vec![
            Edge {
                from: End {
                    node: 0,
                    port: Some("out".to_string()),
                },
                to: End {
                    node: 1,
                    port: Some("in".to_string()),
                },
                kind: EdgeKind::Path,
                label: vec!["0.42".to_string()],
            },
            Edge {
                from: End {
                    node: 0,
                    port: None,
                },
                to: End {
                    node: 1,
                    port: None,
                },
                kind: EdgeKind::Path,
                label: Vec::new(),
            },
            Edge {
                from: End {
                    node: 2,
                    port: None,
                },
                to: End {
                    node: 0,
                    port: Some("in".to_string()),
                },
                kind: EdgeKind::CommonCause,
                label: Vec::new(),
            },
        ],
        groups: vec![Group {
            id: "measurement".to_string(),
            label: vec!["Measurement".to_string()],
            members: vec![0, 1],
        }],
    };
    assert_eq!(form.check(), Ok(()));
    assert_eq!(form.edge_name(0), "edge:engage->perform");
    assert_eq!(form.edge_name(1), "edge:engage->perform#2");
    assert_eq!(form.groups[0].name(), "group:measurement");
    let json = parsed(&form);
    assert_eq!(read(&json), form);
    // The names are in the section as the form gives them.
    assert_eq!(json["edges"][1]["name"], "edge:engage->perform#2");
    assert_eq!(json["nodes"][0]["name"], "node:engage");
    assert_eq!(json["groups"][0]["name"], "group:measurement");
    assert_eq!(json["edges"][0]["heads"], serde_json::json!([false, true]));
    // It is no causal graph, and says which part is not one's.
    assert_eq!(form.to_dag(), Err(Fault::NotACausalGraph("a port")));
    let mut plain = form.clone();
    for node in &mut plain.nodes {
        node.ports.clear();
    }
    for edge in &mut plain.edges {
        edge.from.port = None;
        edge.to.port = None;
    }
    assert_eq!(plain.to_dag(), Err(Fault::NotACausalGraph("a group")));
    plain.groups.clear();
    assert_eq!(
        plain.to_dag(),
        Err(Fault::NotACausalGraph("an edge with a label"))
    );
}

#[test]
fn a_form_that_is_not_what_it_says_is_refused() {
    let form = GraphForm::of_dag(&training());
    let mut twice = form.clone();
    twice.nodes[3].id = "age".to_string();
    assert_eq!(twice.check(), Err(Fault::NodeTwice("age".to_string())));
    assert_eq!(twice.to_dag(), Err(Fault::NodeTwice("age".to_string())));
    let mut stray = form.clone();
    stray.edges[2].to.node = 9;
    assert_eq!(stray.check(), Err(Fault::NoSuchNode(9)));
    let mut port = form.clone();
    port.edges[0].from.port = Some("left".to_string());
    assert_eq!(
        port.check(),
        Err(Fault::NoSuchPort("ability".to_string(), "left".to_string()))
    );
    let mut groups = form.clone();
    let group = |members: Vec<usize>| Group {
        id: "g".to_string(),
        label: Vec::new(),
        members,
    };
    groups.groups = vec![group(vec![0, 1]), group(vec![2])];
    assert_eq!(groups.check(), Err(Fault::GroupTwice("g".to_string())));
    groups.groups = vec![group(vec![0, 6])];
    assert_eq!(groups.check(), Err(Fault::NoSuchNode(6)));
}

#[test]
fn a_scene_with_a_diagram_is_version_two_and_one_without_is_as_it_was() {
    let plot: Plot = start(
        frame(
            "d",
            vec![
                ("x", numbers(&[1.0, 2.0, 3.0])),
                ("y", numbers(&[2.0, 1.0, 4.0])),
            ],
        ),
        vec![mapped(Aesthetic::X, "x"), mapped(Aesthetic::Y, "y")],
        Vec::new(),
    );
    let mut scene = with_layer(&plot, points(1.0)).1;
    let before = scene.to_json();
    let read_scene = |text: &str| -> Read { serde_json::from_str(text).unwrap() };
    assert_eq!(scene.version(), 1);
    assert_eq!(read_scene(&before)["zeff_scene"], 1);
    assert!(read_scene(&before).get("graph").is_none());
    // With a diagram: version 2, and the section, which reads back to the form.
    let form = GraphForm::of_dag(&training());
    scene.graph = Some(form.clone());
    let after = read_scene(&scene.to_json());
    assert_eq!(scene.version(), 2);
    assert_eq!(after["zeff_scene"], 2);
    assert_eq!(read(&after["graph"]), form);
    // Nothing else of the scene's JSON is touched: without the section and the version it
    // is the text it was.
    scene.graph = None;
    assert_eq!(scene.to_json(), before);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Any graph of causes lowers to the form and comes back the same graph; the section
    /// reads back to the same form; every node and every edge has a name of its own.
    #[test]
    fn a_graph_lowers_to_the_form_and_back_without_loss(
        count in 0usize..9,
        pairs in prop::collection::vec((0usize..9, 0usize..9, 0u8..6), 0..24),
        latent in prop::collection::vec(0usize..9, 0..4),
        selected in prop::collection::vec(0usize..9, 0..4),
    ) {
        const NAMES: [&str; 9] = ["a", "b", "c", "gross sales", "e", "latent", "x->y", "ü", "node:z"];
        let nodes: Vec<String> = NAMES[..count].iter().map(|n| (*n).to_string()).collect();
        let mut edges: Vec<DagEdge> = Vec::new();
        for (i, j, kind) in pairs {
            if i >= count || j >= count || i == j {
                continue;
            }
            // An arrow from the earlier node to the later, so that there is no cycle.
            let (from, to, link) = match kind {
                0..=3 => (i.min(j), i.max(j), Link::Directed),
                4 => (i, j, Link::Bidirected),
                _ => (i, j, Link::Undirected),
            };
            let same = |e: &DagEdge| {
                e.link == link
                    && ((e.from, e.to) == (from, to)
                        || (link != Link::Directed && (e.from, e.to) == (to, from)))
            };
            if !edges.iter().any(same) {
                edges.push(DagEdge { from, to, link });
            }
        }
        let mut latent: Vec<usize> = latent.into_iter().filter(|&v| v < count).collect();
        latent.sort_unstable();
        latent.dedup();
        let mut selected: Vec<usize> = selected
            .into_iter()
            .filter(|&v| v < count && !latent.contains(&v))
            .collect();
        selected.sort_unstable();
        selected.dedup();
        let dag = Dag::new(nodes, edges, latent, selected);
        let form = GraphForm::of_dag(&dag);
        prop_assert_eq!(form.check(), Ok(()));
        let back = form.to_dag().unwrap();
        prop_assert_eq!(&back, &dag);
        prop_assert_eq!(back.to_string(), dag.to_string());
        // Lowered again, the same form.
        prop_assert_eq!(&GraphForm::of_dag(&back), &form);
        // The section reads back to the form.
        prop_assert_eq!(&read(&parsed(&form)), &form);
        // Every element has a name of its own.
        let mut names: Vec<String> = form.nodes.iter().map(Node::name).collect();
        names.extend((0..form.edges.len()).map(|at| form.edge_name(at)));
        let total = names.len();
        names.sort();
        names.dedup();
        prop_assert_eq!(names.len(), total);
    }
}

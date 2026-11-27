//! The layered layout, coordinates, edges and labels (release 0.10.0, step 21; open decision
//! #121, D4). What every placed diagram must be, on the graphs of the handbooks and on
//! generated ones of up to thirty nodes, down the page and to the right, at three spacings:
//! no two outlines touch, neighbours on a layer keep their gap, the layers follow in order
//! with their gap, every edge starts on the border of its first node and ends on the border
//! of its second, an arrow points onward, and everything drawn is on the page. Then what the
//! method promises: a chain is a straight line, a node stands under the middle of its two
//! causes, a long edge runs straight through its bends, a common cause is an arc over its
//! layer; the same diagram written in another order is placed the same to the last bit; an
//! outline is sized from its label; and thirty nodes are placed within the time of a frame.

use proptest::prelude::*;
use zeff_plot::drawing::{Place, Step};
use zeff_plot::graph::{Edge, EdgeKind, End, GraphForm, Node, NodeKind, NodeShape};
use zeff_plot::layered::{Layers, layers};
use zeff_plot::placed::{Direction, Measure, Options, Placed, outline_size, place, text_size};

/// A form of nodes called by `names`, the ones listed in `ellipses` latent, with arrows and
/// ties between them by place.
fn form(
    names: &[&str],
    ellipses: &[usize],
    arrows: &[(usize, usize)],
    ties: &[(usize, usize, EdgeKind)],
) -> GraphForm {
    let end = |node: usize| End { node, port: None };
    GraphForm {
        nodes: names
            .iter()
            .enumerate()
            .map(|(at, name)| Node {
                id: (*name).to_string(),
                kind: if ellipses.contains(&at) {
                    NodeKind::Latent
                } else {
                    NodeKind::Observed
                },
                shape: if ellipses.contains(&at) {
                    NodeShape::Ellipse
                } else {
                    NodeShape::Box
                },
                label: vec![(*name).to_string()],
                selected: false,
                role: None,
                ports: Vec::new(),
            })
            .collect(),
        edges: arrows
            .iter()
            .map(|&(from, to)| Edge {
                from: end(from),
                to: end(to),
                kind: EdgeKind::Path,
                label: Vec::new(),
            })
            .chain(ties.iter().map(|&(from, to, kind)| Edge {
                from: end(from),
                to: end(to),
                kind,
                label: Vec::new(),
            }))
            .collect(),
        groups: Vec::new(),
    }
}

const TINY: f64 = 1e-9;

/// The places a line passes or is drawn towards: its ends and its controls.
fn points(steps: &[Step]) -> Vec<Place> {
    let mut all = Vec::new();
    for step in steps {
        match *step {
            Step::Move(p) | Step::Line(p) => all.push(p),
            Step::Curve(c1, c2, p) => all.extend([c1, c2, p]),
            Step::Arc { to, .. } => all.push(to),
            Step::Close => {}
        }
    }
    all
}

/// What every placed diagram must be.
fn sound(form: &GraphForm, laid: &Layers, placed: &Placed, options: &Options) {
    let down = options.direction == Direction::Down;
    let (node_gap, layer_gap) = (
        options.measure.node_gap * options.spacing,
        options.measure.layer_gap * options.spacing,
    );
    let margin = options.measure.margin;
    assert_eq!(placed.nodes.len(), form.nodes.len());
    assert_eq!(placed.edges.len(), form.edges.len());
    // Along and across, whichever way the layers go.
    let along = |p: Place| if down { p.0 } else { p.1 };
    let across = |p: Place| if down { p.1 } else { p.0 };
    let extent = |v: usize| {
        if down {
            placed.nodes[v].width
        } else {
            placed.nodes[v].height
        }
    };
    let thick = |v: usize| {
        if down {
            placed.nodes[v].height
        } else {
            placed.nodes[v].width
        }
    };
    for (v, node) in placed.nodes.iter().enumerate() {
        assert!(node.centre.0.is_finite() && node.centre.1.is_finite());
        // Sized from its label, and on the page with the margin round it.
        let size = outline_size(form.nodes[v].shape, &form.nodes[v].label, &options.measure);
        assert_eq!((node.width, node.height), size);
        assert!(node.centre.0 - node.width / 2.0 >= margin - TINY);
        assert!(node.centre.1 - node.height / 2.0 >= margin - TINY);
        assert!(node.centre.0 + node.width / 2.0 <= placed.width - margin + TINY);
        assert!(node.centre.1 + node.height / 2.0 <= placed.height - margin + TINY);
    }
    // No two outlines touch: apart sideways or upright by more than their half sizes.
    for a in 0..placed.nodes.len() {
        for b in a + 1..placed.nodes.len() {
            let (p, q) = (&placed.nodes[a], &placed.nodes[b]);
            let sideways = (p.centre.0 - q.centre.0).abs() - (p.width + q.width) / 2.0;
            let upright = (p.centre.1 - q.centre.1).abs() - (p.height + q.height) / 2.0;
            assert!(
                sideways > TINY || upright > TINY,
                "{} and {} touch",
                form.nodes[a].id,
                form.nodes[b].id
            );
        }
    }
    // The nodes of a layer in a row, in their order, each its gap from the next; the
    // layers in order, each its gap from the next.
    let mut before: Option<f64> = None;
    for row in &laid.rows {
        let members: Vec<usize> = row
            .iter()
            .filter_map(|slot| match slot {
                zeff_plot::layered::Slot::Node(v) => Some(*v),
                zeff_plot::layered::Slot::Bend(_) => None,
            })
            .collect();
        if members.is_empty() {
            continue;
        }
        let level = across(placed.nodes[members[0]].centre);
        let thickest = members.iter().map(|&v| thick(v)).fold(0.0, f64::max);
        for pair in members.windows(2) {
            let (p, q) = (pair[0], pair[1]);
            assert!((across(placed.nodes[q].centre) - level).abs() < TINY);
            let room = along(placed.nodes[q].centre)
                - along(placed.nodes[p].centre)
                - (extent(p) + extent(q)) / 2.0;
            assert!(
                room >= node_gap - TINY,
                "{} and {} are {room} apart, less than {node_gap}",
                form.nodes[p].id,
                form.nodes[q].id
            );
        }
        if let Some(end) = before {
            assert!(level - thickest / 2.0 - end >= layer_gap - TINY);
        }
        before = Some(level + thickest / 2.0);
    }
    for (e, edge) in placed.edges.iter().enumerate() {
        let (from, to) = (form.edges[e].from.node, form.edges[e].to.node);
        let (start, end) = edge.ends();
        // On the border of each of its nodes.
        assert!(
            placed.nodes[from].outside(start).abs() < 1e-7,
            "{}: starts {} off the border",
            form.edge_name(e),
            placed.nodes[from].outside(start)
        );
        assert!(
            placed.nodes[to].outside(end).abs() < 1e-7,
            "{}: ends {} off the border",
            form.edge_name(e),
            placed.nodes[to].outside(end)
        );
        assert_eq!(edge.heads, form.edges[e].kind.heads());
        // An arrow points onward, from its layer to a later one.
        if form.edges[e].kind == EdgeKind::Path {
            assert!(across(end) > across(start));
        }
        // Everything of the line is on the page.
        for p in points(&edge.steps) {
            assert!(p.0.is_finite() && p.1.is_finite());
            assert!(
                p.0 >= margin - TINY && p.0 <= placed.width - margin + TINY,
                "{p:?}"
            );
            assert!(
                p.1 >= margin - TINY && p.1 <= placed.height - margin + TINY,
                "{p:?}"
            );
        }
        assert!(matches!(edge.steps.first(), Some(Step::Move(_))));
        assert!(edge.steps.len() >= 2);
    }
}

/// The graphs of the handbooks.
fn examples() -> Vec<(&'static str, GraphForm)> {
    let both = EdgeKind::CommonCause;
    vec![
        (
            "training",
            form(
                &[
                    "ability",
                    "training",
                    "productivity",
                    "tenure",
                    "age",
                    "region",
                ],
                &[0],
                &[(0, 1), (1, 2), (0, 2), (1, 3), (3, 2), (4, 1), (5, 1)],
                &[],
            ),
        ),
        ("chain", form(&["x", "m", "y"], &[], &[(0, 1), (1, 2)], &[])),
        (
            "two causes",
            form(&["a", "b", "c"], &[], &[(0, 2), (1, 2)], &[]),
        ),
        (
            "m",
            form(
                &["x", "y", "a", "b", "m"],
                &[2, 3],
                &[(0, 1), (2, 0), (2, 4), (3, 4), (3, 1)],
                &[],
            ),
        ),
        (
            "butterfly",
            form(
                &["x", "y", "a", "b", "m"],
                &[],
                &[(0, 1), (2, 0), (2, 4), (3, 4), (3, 1), (4, 0), (4, 1)],
                &[],
            ),
        ),
        (
            "front door",
            form(&["x", "m", "y"], &[], &[(0, 1), (1, 2)], &[(0, 2, both)]),
        ),
        (
            "common causes",
            form(
                &["price", "demand", "season", "a much longer name"],
                &[],
                &[(2, 1)],
                &[(0, 1, both), (0, 3, both), (2, 3, EdgeKind::Link)],
            ),
        ),
        (
            "earnings",
            form(
                &[
                    "female", "ability", "educ", "exper", "union", "lwage", "age",
                ],
                &[1],
                &[
                    (0, 2),
                    (1, 2),
                    (0, 5),
                    (2, 5),
                    (3, 5),
                    (4, 5),
                    (1, 5),
                    (0, 4),
                    (6, 3),
                    (2, 3),
                ],
                &[],
            ),
        ),
        (
            "long edges",
            form(
                &["a", "b", "c", "d", "e"],
                &[],
                &[(0, 1), (1, 2), (2, 3), (3, 4), (0, 4), (0, 3), (1, 4)],
                &[],
            ),
        ),
        (
            "two parts",
            form(
                &["a", "b", "c", "p", "q", "alone"],
                &[5],
                &[(0, 1), (1, 2), (3, 4)],
                &[],
            ),
        ),
        ("one", form(&["only"], &[], &[], &[])),
        ("empty", form(&[], &[], &[], &[])),
    ]
}

/// A placed diagram as text: each node's middle and size, each edge's line, to the
/// thousandth of a point.
fn shown(form: &GraphForm, placed: &Placed) -> String {
    let n = |x: f64| format!("{x:.3}");
    let p = |at: Place| format!("{},{}", n(at.0), n(at.1));
    let mut out = format!("page {} x {}\n", n(placed.width), n(placed.height));
    for (v, node) in placed.nodes.iter().enumerate() {
        out.push_str(&format!(
            "  {} {} at {} size {} x {}\n",
            form.nodes[v].shape.name(),
            form.nodes[v].id,
            p(node.centre),
            n(node.width),
            n(node.height)
        ));
    }
    for (e, edge) in placed.edges.iter().enumerate() {
        let line: Vec<String> = edge
            .steps
            .iter()
            .map(|step| match *step {
                Step::Move(at) => format!("M {}", p(at)),
                Step::Line(at) => format!("L {}", p(at)),
                Step::Curve(c1, c2, at) => format!("C {} {} {}", p(c1), p(c2), p(at)),
                _ => "?".to_string(),
            })
            .collect();
        out.push_str(&format!("  {}: {}\n", form.edge_name(e), line.join(" ")));
    }
    out
}

#[test]
fn the_example_graphs_are_sound_every_way() {
    let mut all = String::new();
    for (name, form) in examples() {
        let laid = layers(&form, &[]).unwrap();
        for direction in [Direction::Down, Direction::Right] {
            for spacing in [1.0, 0.6, 2.5] {
                let options = Options {
                    direction,
                    spacing,
                    measure: Measure::at(8.0),
                };
                let placed = place(&form, &laid, &options);
                sound(&form, &laid, &placed, &options);
                if spacing == 1.0 {
                    let way = if direction == Direction::Down {
                        "down"
                    } else {
                        "right"
                    };
                    all.push_str(&format!("{name}, {way}: {}", shown(&form, &placed)));
                }
            }
        }
    }
    insta::assert_snapshot!("placed_examples", all);
}

#[test]
fn what_the_method_promises() {
    let options = Options::at(8.0);
    let placed_of = |form: &GraphForm| {
        let laid = layers(form, &[]).unwrap();
        (place(form, &laid, &options), laid)
    };
    // A chain is a straight line down the page, each arrow straight.
    let chain = form(&["x", "m", "y"], &[], &[(0, 1), (1, 2)], &[]);
    let (placed, _) = placed_of(&chain);
    let x = placed.nodes[0].centre.0;
    assert!(
        placed
            .nodes
            .iter()
            .all(|node| (node.centre.0 - x).abs() < TINY)
    );
    for edge in &placed.edges {
        let (start, end) = edge.ends();
        assert_eq!(edge.steps.len(), 2);
        assert!((start.0 - x).abs() < TINY && (end.0 - x).abs() < TINY);
    }
    // A node stands under the middle of its two causes, and over the middle of its two
    // effects.
    let causes = form(&["a", "b", "c"], &[], &[(0, 2), (1, 2)], &[]);
    let (placed, _) = placed_of(&causes);
    let middle = (placed.nodes[0].centre.0 + placed.nodes[1].centre.0) / 2.0;
    assert!((placed.nodes[2].centre.0 - middle).abs() < TINY);
    let effects = form(&["a", "b", "c"], &[], &[(2, 0), (2, 1)], &[]);
    let (placed, _) = placed_of(&effects);
    let middle = (placed.nodes[0].centre.0 + placed.nodes[1].centre.0) / 2.0;
    assert!((placed.nodes[2].centre.0 - middle).abs() < TINY);
    // A long edge runs straight through its bends: the line through them does not move
    // sideways between its first bend and its last.
    let long = form(
        &["a", "b", "c", "d"],
        &[],
        &[(0, 1), (1, 2), (2, 3), (0, 3)],
        &[],
    );
    let (placed, _) = placed_of(&long);
    let Some(edge) = placed.edges.last() else {
        panic!()
    };
    let through: Vec<Place> = edge
        .steps
        .iter()
        .filter_map(|step| match *step {
            Step::Curve(_, _, at) => Some(at),
            _ => None,
        })
        .collect();
    assert_eq!(through.len(), 3, "two bends and the end");
    assert!((through[0].0 - through[1].0).abs() < TINY);
    // A common cause of two nodes of one layer is an arc over the layer: one curve, its
    // controls above both nodes, its ends on their borders.
    let arc = form(
        &["price", "demand"],
        &[],
        &[],
        &[(0, 1, EdgeKind::CommonCause)],
    );
    let (placed, _) = placed_of(&arc);
    let [Step::Move(start), Step::Curve(c1, c2, end)] = placed.edges[0].steps[..] else {
        panic!("{:?}", placed.edges[0].steps);
    };
    let top = placed.nodes[0].centre.1 - placed.nodes[0].height / 2.0;
    assert!(c1.1 < top && c2.1 < top);
    assert!(start.1 <= placed.nodes[0].centre.1 && end.1 <= placed.nodes[1].centre.1);
    assert_eq!(placed.edges[0].heads, (true, true));
    // Between layers, a common cause bows to one side of the straight line.
    let bowed = form(
        &["x", "m", "y"],
        &[],
        &[(0, 1), (1, 2)],
        &[(0, 1, EdgeKind::CommonCause)],
    );
    let (placed, _) = placed_of(&bowed);
    let [Step::Move(start), Step::Curve(c1, _, end)] = placed.edges[2].steps[..] else {
        panic!("{:?}", placed.edges[2].steps);
    };
    assert!((c1.0 - start.0).abs() > 1.0 && (start.0 - end.0).abs() < 8.0);
    // To the right, the layers follow from left to right and a chain is a level line.
    let right = Options {
        direction: Direction::Right,
        ..options
    };
    let laid = layers(&chain, &[]).unwrap();
    let placed = place(&chain, &laid, &right);
    let y = placed.nodes[0].centre.1;
    assert!(
        placed
            .nodes
            .iter()
            .all(|node| (node.centre.1 - y).abs() < TINY)
    );
    assert!(placed.nodes[0].centre.0 < placed.nodes[1].centre.0);
    assert!(placed.nodes[1].centre.0 < placed.nodes[2].centre.0);
    // Wider spacing moves the layers apart by as much, and changes no outline.
    let wide = Options {
        spacing: 2.0,
        ..options
    };
    let (narrow, _) = placed_of(&chain);
    let spread = place(&chain, &laid, &wide);
    let step = |p: &Placed| p.nodes[1].centre.1 - p.nodes[0].centre.1;
    assert!((step(&spread) - step(&narrow) - options.measure.layer_gap).abs() < TINY);
    assert_eq!(spread.nodes[0].width, narrow.nodes[0].width);
}

#[test]
fn an_outline_is_sized_from_its_label() {
    let measure = Measure::at(8.0);
    let lines =
        |texts: &[&str]| -> Vec<String> { texts.iter().map(|t| (*t).to_string()).collect() };
    // A longer label is a wider box of the same height; a second line is a taller one.
    let short = outline_size(NodeShape::Box, &lines(&["age"]), &measure);
    let long = outline_size(NodeShape::Box, &lines(&["productivity"]), &measure);
    let two = outline_size(
        NodeShape::Box,
        &lines(&["productivity", "per hour"]),
        &measure,
    );
    assert!(long.0 > short.0 && long.1 == short.1);
    assert!(two.1 > long.1 && two.0 == long.0);
    // The box is the text with the margin round it.
    let (w, h) = text_size(&lines(&["productivity"]), &measure);
    assert_eq!(long, (w + 2.0 * measure.pad.0, h + 2.0 * measure.pad.1));
    assert_eq!(h, 8.0 * 1.25);
    // The ellipse holds the same box: its corner lies on the ellipse.
    let around = outline_size(NodeShape::Ellipse, &lines(&["productivity"]), &measure);
    let corner = (long.0 / around.0).powi(2) + (long.1 / around.1).powi(2);
    assert!((corner - 1.0).abs() < 1e-12);
    // Twice the type, twice the outline; a letter with a mark and a Greek letter have a
    // width, taken from the face that has them.
    let twice = outline_size(
        NodeShape::Box,
        &lines(&["productivity"]),
        &Measure::at(16.0),
    );
    assert!((twice.0 - 2.0 * long.0).abs() < 1e-9 && (twice.1 - 2.0 * long.1).abs() < 1e-9);
    let marked = outline_size(NodeShape::Box, &lines(&["x̄ and β̂"]), &measure);
    assert!(marked.0 > short.0);
    // A label of an edge has its box at the middle of the line.
    let mut labelled = form(&["x", "y"], &[], &[(0, 1)], &[]);
    labelled.edges[0].label = lines(&["0.42"]);
    let laid = layers(&labelled, &[]).unwrap();
    let placed = place(&labelled, &laid, &Options::at(8.0));
    let label = placed.edges[0].label.as_ref().unwrap();
    let (start, end) = placed.edges[0].ends();
    assert!((label.centre.1 - (start.1 + end.1) / 2.0).abs() < TINY);
    assert!(label.width > 0.0 && label.height > 8.0);
}

/// A small generator of its own, the same on every machine.
struct Draws(u64);

impl Draws {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
    fn chance(&mut self, in_a_thousand: u64) -> bool {
        self.next() % 1000 < in_a_thousand
    }
}

/// A graph of `n` nodes with names of several lengths, about `per_node` arrows to a node.
fn generated(draws: &mut Draws, n: usize, per_node: f64) -> GraphForm {
    const WORDS: [&str; 6] = [
        "x",
        "age",
        "tenure",
        "productivity",
        "years of schooling",
        "u",
    ];
    let names: Vec<String> = (0..n)
        .map(|i| format!("{}{i}", WORDS[draws.below(WORDS.len())]))
        .collect();
    let names: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        order.swap(i, draws.below(i + 1));
    }
    let share = (per_node * 2000.0 / (n.max(2) - 1) as f64) as u64;
    let (mut arrows, mut ties) = (Vec::new(), Vec::new());
    for i in 0..n {
        for j in i + 1..n {
            if draws.chance(share) {
                arrows.push((order[i], order[j]));
            } else if draws.chance(share / 8) {
                let kind = if draws.chance(500) {
                    EdgeKind::CommonCause
                } else {
                    EdgeKind::Link
                };
                ties.push((order[i], order[j], kind));
            }
        }
    }
    let ellipses: Vec<usize> = (0..n).filter(|_| draws.chance(200)).collect();
    form(&names, &ellipses, &arrows, &ties)
}

#[test]
fn generated_graphs_of_up_to_thirty_nodes_are_sound() {
    let mut draws = Draws(0x2026_1128_0021_0001);
    let (mut graphs, mut curved, mut arcs) = (0, 0, 0);
    for round in 0..240 {
        let n = 1 + draws.below(30);
        let form = generated(&mut draws, n, 1.4);
        let laid = layers(&form, &[]).unwrap();
        let options = Options {
            direction: if round % 2 == 0 {
                Direction::Down
            } else {
                Direction::Right
            },
            spacing: [1.0, 0.5, 1.7][round % 3],
            measure: Measure::at(8.0),
        };
        let placed = place(&form, &laid, &options);
        sound(&form, &laid, &placed, &options);
        graphs += 1;
        curved += placed
            .edges
            .iter()
            .filter(|edge| edge.steps.len() > 2)
            .count();
        arcs += form
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::CommonCause)
            .count();
    }
    assert!(
        graphs == 240 && curved > 200 && arcs > 100,
        "{curved} {arcs}"
    );
}

#[test]
fn thirty_nodes_are_placed_within_the_time_of_a_frame() {
    let mut draws = Draws(0x2026_1128_0021_0002);
    let form = generated(&mut draws, 30, 1.5);
    assert_eq!(form.nodes.len(), 30);
    assert!(form.edges.len() >= 30, "{}", form.edges.len());
    let options = Options::at(8.0);
    // The best of several runs, so that another program on the machine does not decide.
    let mut best = f64::INFINITY;
    let mut halves = (0.0, 0.0);
    for _ in 0..7 {
        let start = std::time::Instant::now();
        let laid = layers(&form, &[]).unwrap();
        let first = start.elapsed().as_secs_f64() * 1000.0;
        let placed = place(&form, &laid, &options);
        let both = start.elapsed().as_secs_f64() * 1000.0;
        if both < best {
            best = both;
            halves = (first, both - first);
        }
        sound(&form, &laid, &placed, &options);
    }
    println!(
        "thirty nodes: {best:.2} ms, {:.2} for the layers and their order, {:.2} for the places",
        halves.0, halves.1
    );
    assert!(
        best < 16.0,
        "thirty nodes took {best:.2} ms: {:.2} for the layers and their order, {:.2} for the places",
        halves.0,
        halves.1
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    /// The same diagram with its nodes and its edges written in another order is placed the
    /// same, to the last bit: each node at the same place, each edge on the same line.
    #[test]
    fn the_order_a_diagram_is_written_in_changes_nothing(
        pairs in prop::collection::vec((0usize..8, 0usize..8, 0u8..8), 0..20),
        turn in 0usize..8,
        flip in any::<bool>(),
        right in any::<bool>(),
    ) {
        const NAMES: [&str; 8] = ["age", "b", "cost", "d", "educ", "f g", "h", "zeal"];
        let mut arrows: Vec<(usize, usize)> = Vec::new();
        let mut ties: Vec<(usize, usize, EdgeKind)> = Vec::new();
        for (i, j, kind) in pairs {
            if i == j {
                continue;
            }
            let (low, high) = (i.min(j), i.max(j));
            match kind {
                0..=5 if !arrows.contains(&(low, high)) => arrows.push((low, high)),
                6 if !ties.iter().any(|t| (t.0, t.1) == (low, high)) => {
                    ties.push((low, high, EdgeKind::Link));
                }
                _ => {}
            }
        }
        let options = Options {
            direction: if right { Direction::Right } else { Direction::Down },
            ..Options::at(8.0)
        };
        let first = form(&NAMES, &[2], &arrows, &ties);
        let laid = layers(&first, &[]).unwrap();
        let placed = place(&first, &laid, &options);
        sound(&first, &laid, &placed, &options);
        let mut order: Vec<usize> = (0..8).collect();
        order.rotate_left(turn);
        if flip {
            order.reverse();
        }
        let at = |old: usize| order.iter().position(|&o| o == old).unwrap();
        let names: Vec<&str> = order.iter().map(|&o| NAMES[o]).collect();
        let mut arrows2: Vec<(usize, usize)> = arrows.iter().map(|&(a, b)| (at(a), at(b))).collect();
        arrows2.reverse();
        let ties2: Vec<(usize, usize, EdgeKind)> =
            ties.iter().rev().map(|&(a, b, k)| (at(a), at(b), k)).collect();
        let second = form(&names, &[at(2)], &arrows2, &ties2);
        let relaid = layers(&second, &[]).unwrap();
        let replaced = place(&second, &relaid, &options);
        prop_assert_eq!(placed.width.to_bits(), replaced.width.to_bits());
        prop_assert_eq!(placed.height.to_bits(), replaced.height.to_bits());
        for (old, name) in NAMES.iter().enumerate() {
            prop_assert_eq!(&placed.nodes[old], &replaced.nodes[at(old)], "{}", name);
        }
        for (e, edge) in first.edges.iter().enumerate() {
            let twin = second
                .edges
                .iter()
                .position(|other| {
                    other.kind == edge.kind
                        && other.from.node == at(edge.from.node)
                        && other.to.node == at(edge.to.node)
                })
                .unwrap();
            prop_assert_eq!(&placed.edges[e], &replaced.edges[twin], "{}", first.edge_name(e));
        }
    }
}

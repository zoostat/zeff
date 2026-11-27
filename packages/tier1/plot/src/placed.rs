//! The layered layout of a diagram, second half (Language Reference, section 7.3, `g.plot()`;
//! open decision #121, D4; release 0.10.0, step 21): where on the page each node stands, how
//! large its outline is, and the line each edge follows. The first half ([`crate::layered`])
//! gave the layers and the order within them.
//!
//! **Sizes.** A node's outline is sized from its label, measured in the figure's own face: a
//! box is the text with a margin round it, an ellipse the smallest that holds the same box.
//!
//! **Along the layers.** The places are Brandes and Köpf's (2001). The members of the layers
//! are joined into upright blocks, each member to its median neighbour on the layer before,
//! an inner piece of a long edge before any other, so that long edges run straight; the
//! blocks are then set as near one another as the sizes of their members and the gaps allow.
//! This is done four times, joining upwards and downwards and packing to the left and to
//! the right, and each member is put midway between the two middle places of the four, which
//! sets a node under the middle of its causes and over the middle of its effects.
//!
//! **Across the layers.** A layer is as thick as its thickest node, and the layers follow one
//! another with a gap between. `direction:` says which way the layers follow: down the page,
//! or to the right, where the two axes change places.
//!
//! **Edges.** An edge runs from the border of one node to the border of the other: straight
//! where it joins neighbouring layers, and through its bends as a smooth curve where it
//! crosses several. A common cause is an arc: over the layer where its two nodes share one,
//! and bowed a little where they do not.
//!
//! Every length is in points and follows from the size of the type; `spacing:` multiplies
//! the gaps and nothing else. The arithmetic is that of IEEE 754 in a fixed order, so the
//! same diagram has the same places to the last bit on every machine.

use crate::drawing::{Place, Step};
use crate::font::{FaceName, face, set};
use crate::graph::{EdgeKind, GraphForm, NodeShape};
use crate::layered::{Layers, Slot};

/// Which way the layers follow one another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// The first layer at the top.
    #[default]
    Down,
    /// The first layer at the left.
    Right,
}

/// The lengths of a diagram, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    /// The size of the type of the labels.
    pub size: f64,
    /// From one baseline to the next, as a multiple of the size.
    pub leading: f64,
    /// The room between a label and its outline, sideways and upright.
    pub pad: (f64, f64),
    /// The least room between two neighbours on a layer.
    pub node_gap: f64,
    /// The room between two layers.
    pub layer_gap: f64,
    /// The room round the whole diagram.
    pub margin: f64,
}

impl Measure {
    /// The lengths that go with a size of type.
    pub fn at(size: f64) -> Measure {
        Measure {
            size,
            leading: 1.25,
            pad: (0.75 * size, 0.5 * size),
            node_gap: 2.0 * size,
            layer_gap: 3.5 * size,
            margin: 0.5 * size,
        }
    }
}

/// How a diagram is placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    pub direction: Direction,
    /// What the gaps between nodes and between layers are multiplied by; 1 as it stands.
    pub spacing: f64,
    pub measure: Measure,
}

impl Options {
    pub fn at(size: f64) -> Options {
        Options {
            direction: Direction::Down,
            spacing: 1.0,
            measure: Measure::at(size),
        }
    }
}

/// A node on the page: the middle of its outline and the outline's size.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedNode {
    pub centre: Place,
    pub width: f64,
    pub height: f64,
    pub shape: NodeShape,
}

impl PlacedNode {
    /// Where the ray from the centre towards `to` leaves the outline.
    pub fn border(&self, towards: Place) -> Place {
        let (dx, dy) = (towards.0 - self.centre.0, towards.1 - self.centre.1);
        if dx == 0.0 && dy == 0.0 {
            return self.centre;
        }
        let (a, b) = (self.width / 2.0, self.height / 2.0);
        let t = match self.shape {
            NodeShape::Box => {
                let tx = if dx == 0.0 {
                    f64::INFINITY
                } else {
                    a / dx.abs()
                };
                let ty = if dy == 0.0 {
                    f64::INFINITY
                } else {
                    b / dy.abs()
                };
                tx.min(ty)
            }
            NodeShape::Ellipse => 1.0 / ((dx / a) * (dx / a) + (dy / b) * (dy / b)).sqrt(),
        };
        (self.centre.0 + t * dx, self.centre.1 + t * dy)
    }

    /// How far outside the outline a place is, as a share of the outline: 0 on the border,
    /// below 0 inside. For a box the larger of the two shares, for an ellipse its equation.
    pub fn outside(&self, at: Place) -> f64 {
        let (dx, dy) = (
            (at.0 - self.centre.0) / (self.width / 2.0),
            (at.1 - self.centre.1) / (self.height / 2.0),
        );
        match self.shape {
            NodeShape::Box => dx.abs().max(dy.abs()) - 1.0,
            NodeShape::Ellipse => (dx * dx + dy * dy).sqrt() - 1.0,
        }
    }
}

/// The label of an edge on the page: the middle of its box and the box's size.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedLabel {
    pub centre: Place,
    pub width: f64,
    pub height: f64,
}

/// An edge on the page: its line, from the border of its first node to the border of its
/// second, and whether an arrow head is drawn at each end.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacedEdge {
    pub steps: Vec<Step>,
    pub heads: (bool, bool),
    /// Where its label stands, for an edge that has one. The layout of this release makes
    /// no room for a label: it is set on the middle of the line.
    pub label: Option<PlacedLabel>,
}

impl PlacedEdge {
    /// The place the line starts at and the place it ends at.
    pub fn ends(&self) -> (Place, Place) {
        let at = |step: &Step| match *step {
            Step::Move(p) | Step::Line(p) | Step::Curve(_, _, p) => Some(p),
            Step::Arc { to, .. } => Some(to),
            Step::Close => None,
        };
        let first = self.steps.first().and_then(at).unwrap_or((0.0, 0.0));
        let last = self.steps.iter().rev().find_map(at).unwrap_or(first);
        (first, last)
    }
}

/// A diagram on the page.
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub width: f64,
    pub height: f64,
    /// The nodes of the form, in its order.
    pub nodes: Vec<PlacedNode>,
    /// The edges of the form, in its order.
    pub edges: Vec<PlacedEdge>,
}

/// The size of a label of `lines`: the widest line, and the lines one under another.
pub fn text_size(lines: &[String], measure: &Measure) -> (f64, f64) {
    let units = f64::from(face(FaceName::Sans).units_per_em());
    let widest = lines
        .iter()
        .map(|line| f64::from(set(FaceName::Sans, line).width) * measure.size / units)
        .fold(0.0, f64::max);
    let count = lines.len().max(1) as f64;
    (widest, count * measure.size * measure.leading)
}

/// The size of a node's outline for its label.
pub fn outline_size(shape: NodeShape, lines: &[String], measure: &Measure) -> (f64, f64) {
    let (w, h) = text_size(lines, measure);
    let (w, h) = (w + 2.0 * measure.pad.0, h + 2.0 * measure.pad.1);
    match shape {
        NodeShape::Box => (w, h),
        // The smallest ellipse of the same proportions that holds the box.
        NodeShape::Ellipse => (w * std::f64::consts::SQRT_2, h * std::f64::consts::SQRT_2),
    }
}

/// The members of the layers as the placing sees them.
struct Members {
    /// Each layer's members, by number, in order.
    rows: Vec<Vec<usize>>,
    /// The layer and the place in it of each member.
    layer: Vec<usize>,
    /// How much room each takes along its layer.
    extent: Vec<f64>,
    /// Whether it is a bend of an edge.
    bend: Vec<bool>,
    /// The members joined to it on the layer before, and on the layer after.
    upper: Vec<Vec<usize>>,
    lower: Vec<Vec<usize>>,
}

/// The places along the layers by one of the four ways: joined to the layer before in
/// `rows`' own order, packed towards the start of each row. `towards` gives the neighbours
/// on the layer before; `marked` the pieces that may not be joined.
fn one_way(
    rows: &[Vec<usize>],
    towards: &[Vec<usize>],
    extent: &[f64],
    gap: &dyn Fn(usize, usize) -> f64,
    marked: &dyn Fn(usize, usize) -> bool,
) -> Vec<f64> {
    let n = extent.len();
    let mut place = vec![0usize; n];
    for row in rows {
        for (at, &v) in row.iter().enumerate() {
            place[v] = at;
        }
    }
    // Upright blocks: each member joined to a median neighbour on the layer before.
    let mut root: Vec<usize> = (0..n).collect();
    let mut align: Vec<usize> = (0..n).collect();
    for row in rows.iter().skip(1) {
        let mut reached: Option<usize> = None;
        for &v in row {
            let mut near: Vec<usize> = towards[v].clone();
            near.sort_by_key(|&u| place[u]);
            near.dedup();
            if near.is_empty() {
                continue;
            }
            let middle = [(near.len() - 1) / 2, near.len() / 2];
            for &m in &middle {
                if align[v] != v {
                    break;
                }
                let u = near[m];
                // A neighbour further along than the last one joined is free: no member
                // before `v` on this layer can have been joined to it.
                if !marked(u, v) && reached.is_none_or(|r| r < place[u]) {
                    align[u] = v;
                    root[v] = root[u];
                    align[v] = root[v];
                    reached = Some(place[u]);
                }
            }
        }
    }
    // The blocks as near one another as their members allow: block `a` before block `b` by
    // at least the most any two of their members that are neighbours on a layer need.
    let mut apart: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    let mut waits = vec![0usize; n];
    for row in rows {
        for pair in row.windows(2) {
            let (p, w) = (pair[0], pair[1]);
            let (a, b) = (root[p], root[w]);
            let need = (extent[p] + extent[w]) / 2.0 + gap(p, w);
            match apart[a].iter_mut().find(|(to, _)| *to == b) {
                Some(found) => found.1 = found.1.max(need),
                None => {
                    apart[a].push((b, need));
                    waits[b] += 1;
                }
            }
        }
    }
    // The blocks in an order in which each comes after those before it on any layer.
    let mut order: Vec<usize> = Vec::new();
    let mut ready: Vec<usize> = (0..n).filter(|&v| root[v] == v && waits[v] == 0).collect();
    ready.reverse();
    while let Some(a) = ready.pop() {
        order.push(a);
        let mut freed: Vec<usize> = Vec::new();
        for &(b, _) in &apart[a] {
            waits[b] -= 1;
            if waits[b] == 0 {
                freed.push(b);
            }
        }
        freed.sort_unstable_by(|x, y| y.cmp(x));
        ready.extend(freed);
    }
    let mut x = vec![0.0f64; n];
    // As far towards the start as the blocks before allow.
    for &a in &order {
        for &(b, need) in &apart[a] {
            x[b] = x[b].max(x[a] + need);
        }
    }
    // Then each block as far the other way as the blocks after it allow, so that a block is
    // not left behind where there is room.
    for &a in order.iter().rev() {
        let room = apart[a]
            .iter()
            .map(|&(b, need)| x[b] - need)
            .fold(f64::INFINITY, f64::min);
        if room.is_finite() {
            x[a] = x[a].max(room);
        }
    }
    (0..n).map(|v| x[root[v]]).collect()
}

/// The places along the layers: the middle of the four ways.
fn along(members: &Members, node_gap: f64) -> Vec<f64> {
    let n = members.extent.len();
    if n == 0 {
        return Vec::new();
    }
    let gap = |a: usize, b: usize| -> f64 {
        // Two bends may run closer together than two nodes.
        if members.bend[a] && members.bend[b] {
            node_gap * 0.5
        } else {
            node_gap
        }
    };
    // A piece that crosses an inner piece of a long edge is not joined, so that the long
    // edge stays straight.
    let mut place = vec![0usize; n];
    for row in &members.rows {
        for (at, &v) in row.iter().enumerate() {
            place[v] = at;
        }
    }
    let mut crossed: Vec<(usize, usize)> = Vec::new();
    for row in members.rows.iter().skip(1) {
        let pieces: Vec<(usize, usize)> = row
            .iter()
            .flat_map(|&v| members.upper[v].iter().map(move |&u| (u, v)))
            .collect();
        for &(u, v) in &pieces {
            if members.bend[u] && members.bend[v] {
                continue;
            }
            let crosses_inner = pieces.iter().any(|&(a, b)| {
                members.bend[a]
                    && members.bend[b]
                    && ((place[a] < place[u] && place[b] > place[v])
                        || (place[a] > place[u] && place[b] < place[v]))
            });
            if crosses_inner {
                crossed.push((u, v));
            }
        }
    }
    let marked =
        |a: usize, b: usize| -> bool { crossed.contains(&(a, b)) || crossed.contains(&(b, a)) };
    let turned: Vec<Vec<usize>> = members
        .rows
        .iter()
        .map(|row| row.iter().rev().copied().collect())
        .collect();
    let upside: Vec<Vec<usize>> = members.rows.iter().rev().cloned().collect();
    let upside_turned: Vec<Vec<usize>> = turned.iter().rev().cloned().collect();
    // Joined to the layer before or after; packed to the start or, the rows turned round
    // and the places taken from the other end, to the end.
    let ways: [Vec<f64>; 4] = [
        one_way(
            &members.rows,
            &members.upper,
            &members.extent,
            &gap,
            &marked,
        ),
        one_way(&turned, &members.upper, &members.extent, &gap, &marked)
            .iter()
            .map(|x| -x)
            .collect(),
        one_way(&upside, &members.lower, &members.extent, &gap, &marked),
        one_way(
            &upside_turned,
            &members.lower,
            &members.extent,
            &gap,
            &marked,
        )
        .iter()
        .map(|x| -x)
        .collect(),
    ];
    // The four brought together on the narrowest: those packed to the start by their
    // starts, those packed to the end by their ends.
    let span = |way: &[f64]| -> (f64, f64) {
        (0..n).fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), v| {
            (
                low.min(way[v] - members.extent[v] / 2.0),
                high.max(way[v] + members.extent[v] / 2.0),
            )
        })
    };
    let spans: Vec<(f64, f64)> = ways.iter().map(|way| span(way)).collect();
    let mut narrowest = 0;
    for (k, s) in spans.iter().enumerate() {
        if s.1 - s.0 < spans[narrowest].1 - spans[narrowest].0 {
            narrowest = k;
        }
    }
    let moved: Vec<Vec<f64>> = ways
        .iter()
        .enumerate()
        .map(|(k, way)| {
            let by = if k % 2 == 0 {
                spans[narrowest].0 - spans[k].0
            } else {
                spans[narrowest].1 - spans[k].1
            };
            way.iter().map(|x| x + by).collect()
        })
        .collect();
    (0..n)
        .map(|v| {
            let mut four = [moved[0][v], moved[1][v], moved[2][v], moved[3][v]];
            four.sort_by(f64::total_cmp);
            (four[1] + four[2]) / 2.0
        })
        .collect()
}

/// A smooth curve through `points`, as steps after a move to the first: straight between
/// two, and through more as the cubic curves of a Catmull–Rom spline.
fn through(points: &[Place]) -> Vec<Step> {
    let mut steps = vec![Step::Move(points[0])];
    if points.len() == 2 {
        steps.push(Step::Line(points[1]));
        return steps;
    }
    for i in 0..points.len() - 1 {
        let before = points[i.saturating_sub(1)];
        let (from, to) = (points[i], points[i + 1]);
        let after = points[(i + 2).min(points.len() - 1)];
        steps.push(Step::Curve(
            (
                from.0 + (to.0 - before.0) / 6.0,
                from.1 + (to.1 - before.1) / 6.0,
            ),
            (
                to.0 - (after.0 - from.0) / 6.0,
                to.1 - (after.1 - from.1) / 6.0,
            ),
            to,
        ));
    }
    steps
}

/// The diagram on the page: the nodes of `form` on the layers `laid` gives them.
pub fn place(form: &GraphForm, laid: &Layers, options: &Options) -> Placed {
    let measure = &options.measure;
    let down = options.direction == Direction::Down;
    let node_gap = measure.node_gap * options.spacing;
    let layer_gap = measure.layer_gap * options.spacing;
    let sizes: Vec<(f64, f64)> = form
        .nodes
        .iter()
        .map(|node| outline_size(node.shape, &node.label, measure))
        .collect();
    // Along a layer a node takes its width when the layers go down, its height when they go
    // to the right; across, the other.
    let along_of = |v: usize| if down { sizes[v].0 } else { sizes[v].1 };
    let across_of = |v: usize| if down { sizes[v].1 } else { sizes[v].0 };

    // The members: the nodes, and the bends of each edge layer by layer.
    let mut members = Members {
        rows: Vec::new(),
        layer: Vec::new(),
        extent: Vec::new(),
        bend: Vec::new(),
        upper: Vec::new(),
        lower: Vec::new(),
    };
    let mut number_of_node = vec![usize::MAX; form.nodes.len()];
    let mut bends_of: Vec<Vec<(usize, usize)>> = vec![Vec::new(); form.edges.len()];
    for (layer, row) in laid.rows.iter().enumerate() {
        let mut numbers = Vec::with_capacity(row.len());
        for slot in row {
            let number = members.extent.len();
            match *slot {
                Slot::Node(v) => {
                    number_of_node[v] = number;
                    members.extent.push(along_of(v));
                    members.bend.push(false);
                }
                Slot::Bend(e) => {
                    bends_of[e].push((layer, number));
                    members.extent.push(0.0);
                    members.bend.push(true);
                }
            }
            members.layer.push(layer);
            members.upper.push(Vec::new());
            members.lower.push(Vec::new());
            numbers.push(number);
        }
        members.rows.push(numbers);
    }
    // Each edge as its chain of members from its upper end to its lower.
    let mut chains: Vec<Vec<usize>> = Vec::with_capacity(form.edges.len());
    for (e, edge) in form.edges.iter().enumerate() {
        let (a, b) = (edge.from.node, edge.to.node);
        let from_first = laid.rank[a] <= laid.rank[b];
        let (top, bottom) = if from_first { (a, b) } else { (b, a) };
        let mut chain = vec![number_of_node[top]];
        bends_of[e].sort_unstable();
        chain.extend(bends_of[e].iter().map(|&(_, number)| number));
        chain.push(number_of_node[bottom]);
        if laid.rank[top] != laid.rank[bottom] {
            for pair in chain.windows(2) {
                members.lower[pair[0]].push(pair[1]);
                members.upper[pair[1]].push(pair[0]);
            }
        }
        if !from_first {
            chain.reverse();
        }
        chains.push(chain);
    }
    let places = along(&members, node_gap);
    // Across: each layer as thick as its thickest node.
    let mut middle_of_layer = Vec::with_capacity(laid.rows.len());
    let mut thickness_of_layer = Vec::with_capacity(laid.rows.len());
    let mut at = 0.0;
    for row in &laid.rows {
        let thick = row
            .iter()
            .map(|slot| match *slot {
                Slot::Node(v) => across_of(v),
                Slot::Bend(_) => 0.0,
            })
            .fold(0.0, f64::max);
        middle_of_layer.push(at + thick / 2.0);
        thickness_of_layer.push(thick);
        at += thick + layer_gap;
    }
    let point = |number: usize| -> Place {
        let (a, c) = (places[number], middle_of_layer[members.layer[number]]);
        if down { (a, c) } else { (c, a) }
    };
    let mut nodes: Vec<PlacedNode> = form
        .nodes
        .iter()
        .enumerate()
        .map(|(v, node)| PlacedNode {
            centre: point(number_of_node[v]),
            width: sizes[v].0,
            height: sizes[v].1,
            shape: node.shape,
        })
        .collect();
    // The edges.
    let mut edges: Vec<PlacedEdge> = Vec::with_capacity(form.edges.len());
    for (e, edge) in form.edges.iter().enumerate() {
        let (a, b) = (edge.from.node, edge.to.node);
        let chain = &chains[e];
        let middle: Vec<Place> = chain[1..chain.len() - 1]
            .iter()
            .map(|&m| point(m))
            .collect();
        let (from, to) = (&nodes[a], &nodes[b]);
        let steps = if a == b {
            vec![Step::Move(from.centre)]
        } else if middle.is_empty() && edge.kind == EdgeKind::CommonCause {
            // An arc: over the layer where the two share one, bowed a little where not.
            let (d0, d1) = (to.centre.0 - from.centre.0, to.centre.1 - from.centre.1);
            let length = (d0 * d0 + d1 * d1).sqrt();
            let same_layer = laid.rank[a] == laid.rank[b];
            // The way the arc bows: towards the layer before for two nodes of one layer,
            // to the left of the line otherwise.
            let bow: Place = if same_layer {
                let thick = thickness_of_layer[laid.rank[a]];
                let by = thick / 2.0 + 0.35 * layer_gap + 0.08 * length;
                if down { (0.0, -by) } else { (-by, 0.0) }
            } else {
                let by = 0.14 * length;
                (d1 / length * by, -d0 / length * by)
            };
            let c1 = (
                from.centre.0 + d0 / 4.0 + bow.0,
                from.centre.1 + d1 / 4.0 + bow.1,
            );
            let c2 = (
                to.centre.0 - d0 / 4.0 + bow.0,
                to.centre.1 - d1 / 4.0 + bow.1,
            );
            vec![
                Step::Move(from.border(c1)),
                Step::Curve(c1, c2, to.border(c2)),
            ]
        } else {
            let after = middle.first().copied().unwrap_or(to.centre);
            let before = middle.last().copied().unwrap_or(from.centre);
            let mut points = vec![from.border(after)];
            points.extend(middle.iter().copied());
            points.push(to.border(before));
            through(&points)
        };
        let label = (!edge.label.is_empty()).then(|| {
            let (w, h) = text_size(&edge.label, measure);
            let placed = PlacedEdge {
                steps: steps.clone(),
                heads: (false, false),
                label: None,
            };
            let (first, last) = placed.ends();
            PlacedLabel {
                centre: ((first.0 + last.0) / 2.0, (first.1 + last.1) / 2.0),
                width: w + measure.size,
                height: h + 0.5 * measure.size,
            }
        });
        edges.push(PlacedEdge {
            steps,
            heads: edge.kind.heads(),
            label,
        });
    }
    // The whole brought to the page: a margin round everything drawn, the curves by their
    // controls, which hold them.
    let mut low = (f64::INFINITY, f64::INFINITY);
    let mut high = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut seen = |p: Place| {
        low = (low.0.min(p.0), low.1.min(p.1));
        high = (high.0.max(p.0), high.1.max(p.1));
    };
    for node in &nodes {
        seen((
            node.centre.0 - node.width / 2.0,
            node.centre.1 - node.height / 2.0,
        ));
        seen((
            node.centre.0 + node.width / 2.0,
            node.centre.1 + node.height / 2.0,
        ));
    }
    for edge in &edges {
        for step in &edge.steps {
            match *step {
                Step::Move(p) | Step::Line(p) => seen(p),
                Step::Curve(c1, c2, p) => {
                    seen(c1);
                    seen(c2);
                    seen(p);
                }
                Step::Arc { to, .. } => seen(to),
                Step::Close => {}
            }
        }
        if let Some(label) = &edge.label {
            seen((
                label.centre.0 - label.width / 2.0,
                label.centre.1 - label.height / 2.0,
            ));
            seen((
                label.centre.0 + label.width / 2.0,
                label.centre.1 + label.height / 2.0,
            ));
        }
    }
    if nodes.is_empty() {
        return Placed {
            width: 2.0 * measure.margin,
            height: 2.0 * measure.margin,
            nodes,
            edges,
        };
    }
    let by = (measure.margin - low.0, measure.margin - low.1);
    let shift = |p: Place| -> Place { (p.0 + by.0, p.1 + by.1) };
    for node in &mut nodes {
        node.centre = shift(node.centre);
    }
    for edge in &mut edges {
        for step in &mut edge.steps {
            *step = match *step {
                Step::Move(p) => Step::Move(shift(p)),
                Step::Line(p) => Step::Line(shift(p)),
                Step::Curve(c1, c2, p) => Step::Curve(shift(c1), shift(c2), shift(p)),
                Step::Arc {
                    radii,
                    turn,
                    larger,
                    clockwise,
                    to,
                } => Step::Arc {
                    radii,
                    turn,
                    larger,
                    clockwise,
                    to: shift(to),
                },
                Step::Close => Step::Close,
            };
        }
        if let Some(label) = &mut edge.label {
            label.centre = shift(label.centre);
        }
    }
    Placed {
        width: high.0 - low.0 + 2.0 * measure.margin,
        height: high.1 - low.1 + 2.0 * measure.margin,
        nodes,
        edges,
    }
}

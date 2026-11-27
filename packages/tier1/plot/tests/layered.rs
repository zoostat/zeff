//! The layered layout, layers and order (release 0.10.0, step 20; open decision #121, D4).
//! The layers against the least there is, found here by trying every layering: on every
//! graph of causes of up to four nodes and on generated ones of five and six with weights.
//! The order against the fewest crossings there are, found here by trying every order of
//! every layer: on the graphs of the handbooks, where the layout must reach it, and on
//! generated graphs, where it must come close. The properties: no arrow points upwards; a
//! bend on every layer an edge passes and nowhere else; the same diagram with its nodes and
//! edges in another order has the same layout. And `same_rank:` with its refusals.

use proptest::prelude::*;
use zeff_plot::graph::{Edge, EdgeKind, End, GraphForm, Node, NodeKind, NodeShape};
use zeff_plot::layered::{Layers, Slot, Unlayered, layers, shortest};

/// A form of nodes called by `names`, with arrows and ties between them by place.
fn form(names: &[&str], arrows: &[(usize, usize)], ties: &[(usize, usize, EdgeKind)]) -> GraphForm {
    let end = |node: usize| End { node, port: None };
    GraphForm {
        nodes: names
            .iter()
            .map(|name| Node {
                id: (*name).to_string(),
                kind: NodeKind::Observed,
                shape: NodeShape::Box,
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

fn acyclic(n: usize, arrows: &[(usize, usize)]) -> bool {
    let mut left = vec![true; n];
    loop {
        let free = (0..n).find(|&v| left[v] && !arrows.iter().any(|&(a, b)| left[a] && b == v));
        match free {
            Some(v) => left[v] = false,
            None => return !left.iter().any(|&l| l),
        }
    }
}

/// The sum over the arrows of weight times the layers crossed.
fn length(rank: &[usize], arrows: &[(usize, usize, u32)]) -> u64 {
    arrows
        .iter()
        .map(|&(from, to, weight)| u64::from(weight) * (rank[to] - rank[from]) as u64)
        .sum()
}

/// The least length there is, by trying every layering of the nodes on layers 0 to n - 1.
fn least_length(n: usize, arrows: &[(usize, usize, u32)]) -> u64 {
    let mut rank = vec![0usize; n];
    let mut least = u64::MAX;
    loop {
        if arrows.iter().all(|&(from, to, _)| rank[to] > rank[from]) {
            least = least.min(length(&rank, arrows));
        }
        let mut at = 0;
        loop {
            if at == n {
                return least;
            }
            rank[at] += 1;
            if rank[at] < n {
                break;
            }
            rank[at] = 0;
            at += 1;
        }
    }
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
    fn chance(&mut self, in_a_hundred: u64) -> bool {
        self.next() % 100 < in_a_hundred
    }
}

#[test]
fn the_layers_are_the_shortest_there_are() {
    let mut graphs = 0;
    // Every graph of causes of up to four nodes.
    for n in 1..=4usize {
        let pairs: Vec<(usize, usize)> = (0..n)
            .flat_map(|a| (0..n).map(move |b| (a, b)))
            .filter(|(a, b)| a != b)
            .collect();
        for chosen in 0u32..(1 << pairs.len()) {
            let plain: Vec<(usize, usize)> = pairs
                .iter()
                .enumerate()
                .filter(|(i, _)| chosen >> i & 1 == 1)
                .map(|(_, &p)| p)
                .collect();
            if !acyclic(n, &plain) {
                continue;
            }
            let arrows: Vec<(usize, usize, u32)> = plain.iter().map(|&(a, b)| (a, b, 1)).collect();
            let rank = shortest(n, &arrows);
            assert!(
                arrows.iter().all(|&(a, b, _)| rank[b] > rank[a]),
                "{arrows:?}: {rank:?}"
            );
            assert_eq!(
                length(&rank, &arrows),
                least_length(n, &arrows),
                "{arrows:?}: {rank:?}"
            );
            graphs += 1;
        }
    }
    assert_eq!(graphs, 1 + 3 + 25 + 543);
    // Generated graphs of five and six nodes, the arrows weighted from 1 to 3.
    let mut draws = Draws(0x2026_1128_0020_0001);
    let mut improved = 0;
    for _ in 0..400 {
        let n = 5 + draws.below(2);
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            order.swap(i, draws.below(i + 1));
        }
        let mut arrows = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                if draws.chance(35) {
                    arrows.push((order[i], order[j], 1 + draws.below(3) as u32));
                }
            }
        }
        let rank = shortest(n, &arrows);
        assert!(
            arrows.iter().all(|&(a, b, _)| rank[b] > rank[a]),
            "{arrows:?}: {rank:?}"
        );
        let least = least_length(n, &arrows);
        assert_eq!(length(&rank, &arrows), least, "{arrows:?}: {rank:?}");
        // The layering a longest path gives is often longer: the simplex has work to do.
        let mut longest = vec![0usize; n];
        for _ in 0..n {
            for &(a, b, _) in &arrows {
                longest[b] = longest[b].max(longest[a] + 1);
            }
        }
        improved += usize::from(length(&longest, &arrows) > least);
    }
    assert!(improved > 40, "{improved}");
}

/// The pieces of the edges between neighbouring layers, read from a layout by a way of its
/// own: for each edge, the place of its upper end and of its lower end on each pair of
/// layers it joins.
fn pieces(form: &GraphForm, laid: &Layers) -> Vec<Vec<(Slot, Slot)>> {
    let mut all = vec![Vec::new(); laid.rows.len().saturating_sub(1)];
    for (e, edge) in form.edges.iter().enumerate() {
        let (a, b) = (edge.from.node, edge.to.node);
        let (top, bottom) = if laid.rank[a] <= laid.rank[b] {
            (a, b)
        } else {
            (b, a)
        };
        let (first, last) = (laid.rank[top], laid.rank[bottom]);
        for (layer, list) in all.iter_mut().enumerate().take(last).skip(first) {
            let upper = if layer == first {
                Slot::Node(top)
            } else {
                Slot::Bend(e)
            };
            let lower = if layer + 1 == last {
                Slot::Node(bottom)
            } else {
                Slot::Bend(e)
            };
            list.push((upper, lower));
        }
    }
    all
}

fn crossings(rows: &[Vec<Slot>], between: &[Vec<(Slot, Slot)>]) -> usize {
    let at = |row: &[Slot], slot: Slot| row.iter().position(|s| *s == slot).unwrap();
    let mut count = 0;
    for (layer, list) in between.iter().enumerate() {
        let placed: Vec<(usize, usize)> = list
            .iter()
            .map(|&(u, l)| (at(&rows[layer], u), at(&rows[layer + 1], l)))
            .collect();
        for (i, &(a, b)) in placed.iter().enumerate() {
            for &(c, d) in &placed[i + 1..] {
                if (a < c && b > d) || (a > c && b < d) {
                    count += 1;
                }
            }
        }
    }
    count
}

/// The fewest crossings any order of the layers gives: every order of every layer is tried.
fn fewest_crossings(rows: &[Vec<Slot>], between: &[Vec<(Slot, Slot)>]) -> usize {
    fn orders(row: &[Slot]) -> Vec<Vec<Slot>> {
        if row.len() <= 1 {
            return vec![row.to_vec()];
        }
        let mut all = Vec::new();
        for i in 0..row.len() {
            let mut rest = row.to_vec();
            let first = rest.remove(i);
            for mut tail in orders(&rest) {
                tail.insert(0, first);
                all.push(tail);
            }
        }
        all
    }
    fn go(
        layer: usize,
        chosen: &mut Vec<Vec<Slot>>,
        every: &[Vec<Vec<Slot>>],
        between: &[Vec<(Slot, Slot)>],
        best: &mut usize,
    ) {
        if layer == every.len() {
            *best = (*best).min(crossings(chosen, between));
            return;
        }
        for order in &every[layer] {
            chosen.push(order.clone());
            // What is chosen so far already crosses no less than the best: go no further.
            if crossings(chosen, &between[..chosen.len().saturating_sub(1)]) < *best {
                go(layer + 1, chosen, every, between, best);
            }
            chosen.pop();
        }
    }
    let every: Vec<Vec<Vec<Slot>>> = rows.iter().map(|row| orders(row)).collect();
    let mut best = usize::MAX;
    go(0, &mut Vec::new(), &every, between, &mut best);
    best
}

/// What every layout must be, whatever the diagram.
fn sound(form: &GraphForm, laid: &Layers) {
    let n = form.nodes.len();
    assert_eq!(laid.rank.len(), n);
    // No arrow points upwards or along a layer.
    for edge in &form.edges {
        if edge.kind == EdgeKind::Path {
            assert!(laid.rank[edge.to.node] > laid.rank[edge.from.node]);
        }
    }
    // Each node once, on its layer; a bend of an edge on every layer strictly between its
    // ends and nowhere else; no layer empty.
    for v in 0..n {
        for (layer, row) in laid.rows.iter().enumerate() {
            let times = row.iter().filter(|s| **s == Slot::Node(v)).count();
            assert_eq!(times, usize::from(layer == laid.rank[v]));
        }
    }
    for (e, edge) in form.edges.iter().enumerate() {
        let (a, b) = (laid.rank[edge.from.node], laid.rank[edge.to.node]);
        let (low, high) = (a.min(b), a.max(b));
        for (layer, row) in laid.rows.iter().enumerate() {
            let times = row.iter().filter(|s| **s == Slot::Bend(e)).count();
            assert_eq!(
                times,
                usize::from(layer > low && layer < high),
                "edge {e} on layer {layer}"
            );
        }
    }
    assert!(laid.rows.iter().all(|row| !row.is_empty()));
    assert_eq!(
        laid.rows.len(),
        laid.rank.iter().copied().max().map_or(0, |r| r + 1)
    );
    // The crossings the layout says it has are those its rows have.
    assert_eq!(laid.crossings, crossings(&laid.rows, &pieces(form, laid)));
}

/// A layout as text, by the names: each layer's members in order, a bend by its edge.
fn shown(form: &GraphForm, laid: &Layers) -> String {
    laid.rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|slot| match slot {
                    Slot::Node(v) => form.nodes[*v].id.clone(),
                    Slot::Bend(e) => {
                        // An arrow by its cause and its effect; a tie, which has no first
                        // and second end, by the earlier name first.
                        let edge = &form.edges[*e];
                        let (a, b) = (&form.nodes[edge.from.node].id, &form.nodes[edge.to.node].id);
                        if edge.kind == EdgeKind::Path || a <= b {
                            format!("({a}~{b})")
                        } else {
                            format!("({b}~{a})")
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The graphs of the handbooks, each with the fewest crossings it can have.
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
                &[(0, 1), (1, 2), (0, 2), (1, 3), (3, 2), (4, 1), (5, 1)],
                &[],
            ),
        ),
        ("chain", form(&["x", "m", "y"], &[(0, 1), (1, 2)], &[])),
        (
            "fork",
            form(&["z", "x", "y"], &[(0, 1), (0, 2), (1, 2)], &[]),
        ),
        (
            "collider",
            form(&["x", "y", "c"], &[(0, 2), (1, 2), (0, 1)], &[]),
        ),
        (
            "m",
            form(
                &["x", "y", "a", "b", "m"],
                &[(0, 1), (2, 0), (2, 4), (3, 4), (3, 1)],
                &[],
            ),
        ),
        (
            "butterfly",
            form(
                &["x", "y", "a", "b", "m"],
                &[(0, 1), (2, 0), (2, 4), (3, 4), (3, 1), (4, 0), (4, 1)],
                &[],
            ),
        ),
        (
            "instrument",
            form(
                &["z", "x", "y", "u"],
                &[(0, 1), (1, 2), (3, 1), (3, 2)],
                &[],
            ),
        ),
        (
            "front door",
            form(&["x", "m", "y"], &[(0, 1), (1, 2)], &[(0, 2, both)]),
        ),
        (
            "earnings",
            form(
                &[
                    "female", "ability", "educ", "exper", "union", "lwage", "age",
                ],
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
                &[(0, 1), (1, 2), (2, 3), (3, 4), (0, 4), (0, 3), (1, 4)],
                &[],
            ),
        ),
        (
            "two parts",
            form(
                &["a", "b", "c", "p", "q", "alone"],
                &[(0, 1), (1, 2), (3, 4)],
                &[],
            ),
        ),
        (
            "needs a crossing",
            form(
                &["a", "b", "c", "x", "y", "z"],
                &[
                    (0, 3),
                    (0, 4),
                    (0, 5),
                    (1, 3),
                    (1, 4),
                    (1, 5),
                    (2, 3),
                    (2, 4),
                    (2, 5),
                ],
                &[],
            ),
        ),
        ("empty", form(&[], &[], &[])),
        ("one", form(&["only"], &[], &[])),
    ]
}

#[test]
fn the_example_graphs_have_no_crossing_where_none_is_needed() {
    let mut all = String::new();
    for (name, form) in examples() {
        let laid = layers(&form, &[]).unwrap();
        sound(&form, &laid);
        let least = fewest_crossings(&laid.rows, &pieces(&form, &laid));
        assert_eq!(laid.crossings, least, "{name}: {}", shown(&form, &laid));
        all.push_str(&format!(
            "{name}: {} crossings\n  {}\n",
            laid.crossings,
            shown(&form, &laid)
        ));
    }
    insta::assert_snapshot!("layers_examples", all);
}

#[test]
fn generated_graphs_are_sound_and_close_to_the_fewest_crossings() {
    let mut draws = Draws(0x2026_1128_0020_0002);
    let (mut graphs, mut at_the_least, mut over) = (0usize, 0usize, 0usize);
    for _ in 0..300 {
        let n = 3 + draws.below(5);
        let names: Vec<String> = (0..n).map(|i| format!("v{i}")).collect();
        let names: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            order.swap(i, draws.below(i + 1));
        }
        let (mut arrows, mut ties) = (Vec::new(), Vec::new());
        for i in 0..n {
            for j in i + 1..n {
                if draws.chance(30) {
                    arrows.push((order[i], order[j]));
                } else if draws.chance(8) {
                    let kind = if draws.chance(50) {
                        EdgeKind::CommonCause
                    } else {
                        EdgeKind::Link
                    };
                    ties.push((order[j], order[i], kind));
                }
            }
        }
        let form = form(&names, &arrows, &ties);
        let laid = layers(&form, &[]).unwrap();
        sound(&form, &laid);
        // Where the layers are small enough to try every order, the layout is held to the
        // fewest crossings there are.
        let orders: f64 = laid
            .rows
            .iter()
            .map(|row| (1..=row.len()).map(|k| k as f64).product::<f64>())
            .product();
        if orders <= 50_000.0 {
            let least = fewest_crossings(&laid.rows, &pieces(&form, &laid));
            assert!(laid.crossings >= least);
            graphs += 1;
            at_the_least += usize::from(laid.crossings == least);
            over += laid.crossings - least;
        }
    }
    assert!(
        graphs > 250 && at_the_least * 100 >= graphs * 97 && over <= 12,
        "{graphs} {at_the_least} {over}"
    );
}

#[test]
fn nodes_asked_to_share_a_layer_do_and_what_cannot_be_is_refused() {
    // A fork whose two branches differ in length: the ends are brought to one layer.
    let f = form(
        &["z", "a", "b", "x", "y"],
        &[(0, 1), (1, 2), (2, 3), (0, 4)],
        &[],
    );
    let free = layers(&f, &[]).unwrap();
    assert_ne!(free.rank[3], free.rank[4]);
    let tied = layers(&f, &[vec![3, 4]]).unwrap();
    sound(&f, &tied);
    assert_eq!(tied.rank[3], tied.rank[4]);
    assert_eq!(tied.rank, [0, 1, 2, 3, 3]);
    // Two sets; a set of one, and a node named twice in its set, change nothing.
    let two = layers(&f, &[vec![3, 4], vec![1, 1], vec![2]]).unwrap();
    assert_eq!(two, tied);
    // Nodes that an arrow, or a chain of arrows, sets apart.
    assert_eq!(layers(&f, &[vec![0, 1]]), Err(Unlayered::Apart(0, 1)));
    assert!(matches!(
        layers(&f, &[vec![0, 3]]),
        Err(Unlayered::Apart(_, _))
    ));
    // A node in two sets; a place that is no node.
    assert_eq!(
        layers(&f, &[vec![3, 4], vec![4, 1]]),
        Err(Unlayered::InTwoSets(4))
    );
    assert_eq!(layers(&f, &[vec![3, 9]]), Err(Unlayered::NoSuchNode(9)));
    // Arrows in a ring, which no graph of causes has, are named.
    let ringed = form(
        &["a", "b", "c", "d"],
        &[(0, 1), (1, 2), (2, 0), (2, 3)],
        &[],
    );
    let Err(Unlayered::Ring(nodes)) = layers(&ringed, &[]) else {
        panic!("a ring was laid out");
    };
    let mut nodes = nodes;
    nodes.sort_unstable();
    assert_eq!(nodes, [0, 1, 2]);
    assert_eq!(
        layers(&form(&["a"], &[(0, 0)], &[]), &[]),
        Err(Unlayered::Ring(vec![0]))
    );
    // A tie does not rank its ends: joined by a common cause alone, two nodes share the
    // first layer; and a tie between layers is given its bends like an arrow.
    let tie = form(&["a", "b"], &[], &[(0, 1, EdgeKind::CommonCause)]);
    assert_eq!(layers(&tie, &[]).unwrap().rank, [0, 0]);
    let long = form(
        &["a", "b", "c", "d"],
        &[(0, 1), (1, 2), (2, 3)],
        &[(3, 0, EdgeKind::Link)],
    );
    let laid = layers(&long, &[]).unwrap();
    sound(&long, &laid);
    assert_eq!(shown(&long, &laid), "a | b (a~d) | c (a~d) | d");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// The same diagram with its nodes and its edges written in another order has the same
    /// layout: the same layer for each name, the same order within each layer.
    #[test]
    fn the_order_a_diagram_is_written_in_changes_nothing(
        pairs in prop::collection::vec((0usize..8, 0usize..8, 0u8..8), 0..20),
        turn in 0usize..8,
        flip in any::<bool>(),
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
                    ties.push((low, high, EdgeKind::CommonCause));
                }
                _ => {}
            }
        }
        let first = form(&NAMES, &arrows, &ties);
        let laid = layers(&first, &[]).unwrap();
        sound(&first, &laid);
        // The nodes turned round and, sometimes, reversed; the edges reversed.
        let mut order: Vec<usize> = (0..8).collect();
        order.rotate_left(turn);
        if flip {
            order.reverse();
        }
        let place = |old: usize| order.iter().position(|&o| o == old).unwrap();
        let names: Vec<&str> = order.iter().map(|&o| NAMES[o]).collect();
        let mut arrows2: Vec<(usize, usize)> = arrows.iter().map(|&(a, b)| (place(a), place(b))).collect();
        arrows2.reverse();
        // A tie written the other way round is the same tie.
        let ties2: Vec<(usize, usize, EdgeKind)> =
            ties.iter().rev().map(|&(a, b, k)| (place(b), place(a), k)).collect();
        let second = form(&names, &arrows2, &ties2);
        let relaid = layers(&second, &[]).unwrap();
        sound(&second, &relaid);
        for (old, name) in NAMES.iter().enumerate() {
            prop_assert_eq!(laid.rank[old], relaid.rank[place(old)], "{}", name);
        }
        prop_assert_eq!(shown(&first, &laid), shown(&second, &relaid));
        prop_assert_eq!(laid.crossings, relaid.crossings);
    }
}

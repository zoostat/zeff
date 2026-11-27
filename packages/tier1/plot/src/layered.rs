//! The layered layout of a diagram, first half (Language Reference, section 7.3, `g.plot()`;
//! open decision #121, D4; release 0.10.0, step 20): which layer each node stands on, and in
//! what order the nodes of a layer stand. The coordinates, the edges and the labels are the
//! second half, at step 21.
//!
//! The method is Sugiyama's, in the form of Gansner, Koutsofios, North and Vo (1993). The
//! layers are those that make the arrows as short as they can be while every arrow points
//! from a layer to a later one: the sum of the layers each arrow crosses is the least there
//! is, found by the network simplex. An arrow that crosses more than one layer is given a
//! bend on each layer between, so that every piece joins neighbouring layers. The order
//! within the layers is then improved in a fixed number of sweeps, down and up by turns, each
//! placing a layer's members by the median of their neighbours on the layer before and then
//! exchanging neighbours where that leaves fewer crossings; the order with the fewest
//! crossings met is kept. The sweeps start from four first orders in turn, and a diagram
//! small enough has every order of its layers tried where the sweeps leave a crossing, so
//! that a small diagram, which most causal graphs are, has no crossing it does not need.
//!
//! Only an arrow, a path from a cause to its effect, says which of two nodes stands above.
//! A common cause and a link for drawing join two nodes without ranking them: they take
//! part in the order, where their ends stand on different layers, and not in the layers.
//!
//! Nothing here depends on the order in which the diagram was written. The nodes are taken
//! in the order of their names and the edges in the order of the names of their ends, every
//! tie is broken by that order, and no step draws a number at random: the same diagram has
//! the same layout in every run and on every machine, and so has the same diagram with its
//! lines in another order.

use crate::graph::{EdgeKind, GraphForm};

/// The sweeps of the ordering from each first order: down and up by turns.
pub const SWEEPS: usize = 24;

/// A diagram with no more orders of its layers than this, every order of every layer
/// counted together, has every one of them tried when the sweeps leave a crossing.
pub const EVERY_ORDER: f64 = 50_000.0;

/// A place on a layer: a node of the diagram, or a bend of an edge that passes the layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// The node, by its place in the form.
    Node(usize),
    /// The edge, by its place in the form.
    Bend(usize),
}

/// The layers of a diagram and the order within them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layers {
    /// The layer of each node of the form, the first layer being 0.
    pub rank: Vec<usize>,
    /// Each layer's members, in their order.
    pub rows: Vec<Vec<Slot>>,
    /// How many pairs of edge pieces cross between neighbouring layers.
    pub crossings: usize,
}

/// Why a diagram has no layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unlayered {
    /// The arrows go round in a ring through the nodes named.
    Ring(Vec<usize>),
    /// Two nodes asked to stand on one layer have an arrow between them, or a chain of
    /// arrows; the two are named.
    Apart(usize, usize),
    /// A node named in two sets of `same_rank:`.
    InTwoSets(usize),
    /// A place that is no node of the form.
    NoSuchNode(usize),
}

/// The canonical order of the nodes and edges of a form, and the arrows among the edges.
struct Canon {
    /// The nodes of the form in the order of their names; `at[node]` is its place in it.
    order: Vec<usize>,
    at: Vec<usize>,
    /// The edges in canonical order: the edge's place in the form, and its ends in
    /// canonical places, a tie's ends with the earlier first.
    edges: Vec<(usize, usize, usize, bool)>,
}

impl Canon {
    fn of(form: &GraphForm) -> Canon {
        let mut order: Vec<usize> = (0..form.nodes.len()).collect();
        order.sort_by(|&a, &b| form.nodes[a].id.cmp(&form.nodes[b].id));
        let mut at = vec![0; order.len()];
        for (place, &node) in order.iter().enumerate() {
            at[node] = place;
        }
        let mut edges: Vec<(usize, usize, usize, bool)> = form
            .edges
            .iter()
            .enumerate()
            .map(|(e, edge)| {
                let (a, b) = (at[edge.from.node], at[edge.to.node]);
                let arrow = edge.kind == EdgeKind::Path;
                if arrow {
                    (e, a, b, true)
                } else {
                    (e, a.min(b), a.max(b), false)
                }
            })
            .collect();
        edges.sort_by_key(|&(e, a, b, arrow)| (a, b, !arrow, form.edges[e].kind.name(), e));
        Canon { order, at, edges }
    }
}

/// The layers that make the arrows shortest: for nodes 0..n and arrows `(from, to, weight)`
/// with no ring among them, a layer for each node such that every arrow points to a later
/// layer and the sum over the arrows of weight times the layers crossed is the least there
/// is. Each connected part starts at layer 0.
pub fn shortest(n: usize, arrows: &[(usize, usize, u32)]) -> Vec<usize> {
    let mut rank: Vec<i64> = vec![0; n];
    // A first feasible layering: each node one below the lowest of its causes.
    let mut left: Vec<usize> = vec![0; n];
    for &(_, to, _) in arrows {
        left[to] += 1;
    }
    let mut ready: Vec<usize> = (0..n).filter(|&v| left[v] == 0).collect();
    let mut seen = 0;
    while let Some(v) = ready.pop() {
        seen += 1;
        for &(from, to, _) in arrows {
            if from == v {
                rank[to] = rank[to].max(rank[v] + 1);
                left[to] -= 1;
                if left[to] == 0 {
                    ready.push(to);
                }
            }
        }
    }
    debug_assert_eq!(seen, n, "the arrows hold a ring");
    // The connected parts, each improved by itself.
    let mut part = vec![usize::MAX; n];
    let mut parts = 0;
    for start in 0..n {
        if part[start] != usize::MAX {
            continue;
        }
        let mut todo = vec![start];
        part[start] = parts;
        while let Some(v) = todo.pop() {
            for &(from, to, _) in arrows {
                let other = if from == v {
                    to
                } else if to == v {
                    from
                } else {
                    continue;
                };
                if part[other] == usize::MAX {
                    part[other] = parts;
                    todo.push(other);
                }
            }
        }
        parts += 1;
    }
    for p in 0..parts {
        let nodes: Vec<usize> = (0..n).filter(|&v| part[v] == p).collect();
        let inside: Vec<usize> = (0..arrows.len())
            .filter(|&e| part[arrows[e].0] == p)
            .collect();
        if nodes.len() > 1 {
            simplex(&nodes, &inside, arrows, &mut rank);
        }
        let least = nodes.iter().map(|&v| rank[v]).min().unwrap_or(0);
        for &v in &nodes {
            rank[v] -= least;
        }
    }
    rank.iter().map(|&r| r as usize).collect()
}

/// The network simplex on one connected part: `nodes` and the arrows `inside` it, `rank` a
/// feasible layering on entry and a shortest one on return.
fn simplex(nodes: &[usize], inside: &[usize], arrows: &[(usize, usize, u32)], rank: &mut [i64]) {
    let slack = |e: usize, rank: &[i64]| rank[arrows[e].1] - rank[arrows[e].0] - 1;
    // A spanning tree of tight arrows: grown from the first node, the layers of the tree
    // moved as one to make the nearest arrow tight whenever the tree can grow no further.
    let mut in_tree = vec![false; rank.len()];
    let mut tree: Vec<usize> = Vec::new();
    in_tree[nodes[0]] = true;
    let mut size = 1;
    while size < nodes.len() {
        let grown = inside.iter().copied().find(|&e| {
            let (from, to, _) = arrows[e];
            in_tree[from] != in_tree[to] && slack(e, rank) == 0
        });
        if let Some(e) = grown {
            let (from, to, _) = arrows[e];
            in_tree[from] = true;
            in_tree[to] = true;
            tree.push(e);
            size += 1;
            continue;
        }
        // The arrow with the least slack that leaves the tree; the first of them.
        let nearest = inside
            .iter()
            .copied()
            .filter(|&e| in_tree[arrows[e].0] != in_tree[arrows[e].1])
            .min_by_key(|&e| slack(e, rank))
            .expect("a connected part has an arrow out of every smaller tree");
        let by = slack(nearest, rank);
        // The tree moves towards the arrow's other end.
        let delta = if in_tree[arrows[nearest].0] { by } else { -by };
        for &v in nodes {
            if in_tree[v] {
                rank[v] += delta;
            }
        }
    }
    // Exchange a tree arrow whose cut value is negative for the arrow that may take its
    // place, until there is none.
    let mut turns = 0;
    let most = 8 * (inside.len() + 1) * (nodes.len() + 1);
    loop {
        turns += 1;
        if turns > most {
            return;
        }
        let mut leaving = None;
        for (slot, &e) in tree.iter().enumerate() {
            // The side of the tree the arrow's tail is on, without the arrow.
            let tail = side_of(arrows[e].0, e, &tree, arrows, rank.len());
            let mut cut: i64 = 0;
            for &f in inside {
                let (from, to, weight) = arrows[f];
                if tail[from] && !tail[to] {
                    cut += i64::from(weight);
                } else if !tail[from] && tail[to] {
                    cut -= i64::from(weight);
                }
            }
            if cut < 0 {
                leaving = Some((slot, tail));
                break;
            }
        }
        let Some((slot, tail)) = leaving else {
            return;
        };
        // The arrow from the head's side to the tail's side with the least slack.
        let entering = inside
            .iter()
            .copied()
            .filter(|&f| !tail[arrows[f].0] && tail[arrows[f].1])
            .min_by_key(|&f| slack(f, rank))
            .expect("a negative cut value has an arrow the other way");
        let by = slack(entering, rank);
        // The entering arrow points from the head's side into the tail's side: the tail's
        // side moves up by its slack, which makes it tight and leaves every arrow that
        // points the other way with more room, and every other arrow that points the same
        // way with no less than none.
        for &v in nodes {
            if tail[v] {
                rank[v] -= by;
            }
        }
        tree[slot] = entering;
    }
}

/// The nodes joined to `start` by the arrows of `tree` other than `without`.
fn side_of(
    start: usize,
    without: usize,
    tree: &[usize],
    arrows: &[(usize, usize, u32)],
    n: usize,
) -> Vec<bool> {
    let mut side = vec![false; n];
    side[start] = true;
    let mut todo = vec![start];
    while let Some(v) = todo.pop() {
        for &e in tree {
            if e == without {
                continue;
            }
            let (from, to, _) = arrows[e];
            let other = if from == v {
                to
            } else if to == v {
                from
            } else {
                continue;
            };
            if !side[other] {
                side[other] = true;
                todo.push(other);
            }
        }
    }
    side
}

/// A ring among the arrows, as its nodes in order, if there is one.
fn ring(n: usize, arrows: &[(usize, usize, u32)]) -> Option<Vec<usize>> {
    // 0 not met, 1 on the way, 2 done.
    fn visit(
        v: usize,
        arrows: &[(usize, usize, u32)],
        state: &mut [u8],
        way: &mut Vec<usize>,
    ) -> Option<Vec<usize>> {
        state[v] = 1;
        way.push(v);
        for &(from, to, _) in arrows {
            if from != v {
                continue;
            }
            if state[to] == 1 {
                let start = way.iter().position(|&w| w == to).unwrap_or(0);
                return Some(way[start..].to_vec());
            }
            if state[to] == 0
                && let Some(found) = visit(to, arrows, state, way)
            {
                return Some(found);
            }
        }
        way.pop();
        state[v] = 2;
        None
    }
    let mut state = vec![0u8; n];
    for v in 0..n {
        if state[v] == 0
            && let Some(found) = visit(v, arrows, &mut state, &mut Vec::new())
        {
            return Some(found);
        }
    }
    None
}

/// One piece of an edge between neighbouring layers: the places of its two ends among the
/// members of its layers.
type Piece = (usize, usize);

/// The crossings between two neighbouring layers: the pairs of pieces whose ends are in
/// opposite orders on the two.
fn crossings_between(pieces: &[Piece], upper: &[usize], lower: &[usize]) -> usize {
    let mut count = 0;
    for (i, &(a, b)) in pieces.iter().enumerate() {
        for &(c, d) in &pieces[i + 1..] {
            let (ua, uc, lb, ld) = (upper[a], upper[c], lower[b], lower[d]);
            if (ua < uc && lb > ld) || (ua > uc && lb < ld) {
                count += 1;
            }
        }
    }
    count
}

/// The layers of a diagram and the order within them. `same_rank` lists sets of nodes, by
/// their places in the form, that are to stand on one layer.
pub fn layers(form: &GraphForm, same_rank: &[Vec<usize>]) -> Result<Layers, Unlayered> {
    let n = form.nodes.len();
    let canon = Canon::of(form);
    // The sets of `same_rank:`, each as one node of the layering.
    let mut set_of: Vec<usize> = (0..n).collect();
    let mut taken = vec![false; n];
    for set in same_rank {
        let mut members: Vec<usize> = Vec::new();
        for &node in set {
            if node >= n {
                return Err(Unlayered::NoSuchNode(node));
            }
            if members.contains(&canon.at[node]) {
                continue;
            }
            if taken[node] {
                return Err(Unlayered::InTwoSets(node));
            }
            members.push(canon.at[node]);
        }
        for &node in set {
            taken[node] = true;
        }
        if let Some(&first) = members.iter().min() {
            for &member in &members {
                set_of[member] = first;
            }
        }
    }
    // The arrows between the sets, those of one pair as one arrow of their number.
    let mut arrows: Vec<(usize, usize, u32)> = Vec::new();
    for &(_, a, b, arrow) in &canon.edges {
        if !arrow {
            continue;
        }
        let (from, to) = (set_of[a], set_of[b]);
        if from == to {
            if a == b {
                // An arrow from a node to itself is no arrow of a diagram that can be laid
                // out in layers.
                return Err(Unlayered::Ring(vec![canon.order[a]]));
            }
            return Err(Unlayered::Apart(canon.order[a], canon.order[b]));
        }
        match arrows.iter_mut().find(|(f, t, _)| (*f, *t) == (from, to)) {
            Some(found) => found.2 += 1,
            None => arrows.push((from, to, 1)),
        }
    }
    if let Some(found) = ring(n, &arrows) {
        // A ring through a set of `same_rank:` is the set's doing when the arrows alone
        // have none.
        let plain: Vec<(usize, usize, u32)> = canon
            .edges
            .iter()
            .filter(|edge| edge.3)
            .map(|&(_, a, b, _)| (a, b, 1))
            .collect();
        return Err(match ring(n, &plain) {
            Some(real) => Unlayered::Ring(real.iter().map(|&v| canon.order[v]).collect()),
            None => {
                let within =
                    |set: usize| -> Vec<usize> { (0..n).filter(|&v| set_of[v] == set).collect() };
                let pair = found
                    .iter()
                    .map(|&set| within(set))
                    .find(|members| members.len() > 1)
                    .unwrap_or_default();
                Unlayered::Apart(
                    canon.order[pair.first().copied().unwrap_or(0)],
                    canon.order[pair.get(1).copied().unwrap_or(0)],
                )
            }
        });
    }
    let by_set = shortest(n, &arrows);
    // Each connected part of the sets started at 0; a set's members share its layer. A node
    // joined to nothing by an arrow stands on the first layer.
    let rank_canon: Vec<usize> = (0..n).map(|v| by_set[set_of[v]]).collect();
    let depth = rank_canon.iter().copied().max().map_or(0, |r| r + 1);

    // The members of the layers: the nodes, and a bend for each layer an edge passes.
    // A member is numbered: the nodes in canonical order first, then the bends edge by edge.
    #[derive(Clone, Copy)]
    struct Member {
        slot: Slot,
        layer: usize,
    }
    let mut members: Vec<Member> = (0..n)
        .map(|v| Member {
            slot: Slot::Node(canon.order[v]),
            layer: rank_canon[v],
        })
        .collect();
    // The pieces between layer r and layer r + 1, as (member above, member below).
    let mut pieces: Vec<Vec<(usize, usize)>> = vec![Vec::new(); depth.saturating_sub(1)];
    for &(e, a, b, _) in &canon.edges {
        let (top, bottom) = if rank_canon[a] <= rank_canon[b] {
            (a, b)
        } else {
            (b, a)
        };
        let (from, to) = (rank_canon[top], rank_canon[bottom]);
        if from == to {
            continue;
        }
        let mut above = top;
        for layer in from + 1..to {
            members.push(Member {
                slot: Slot::Bend(e),
                layer,
            });
            let bend = members.len() - 1;
            pieces[layer - 1].push((above, bend));
            above = bend;
        }
        pieces[to - 1].push((above, bottom));
    }
    // A first order: each layer's members as they are met, walking the pieces down from
    // each member in turn.
    let mut below: Vec<Vec<usize>> = vec![Vec::new(); members.len()];
    let mut above: Vec<Vec<usize>> = vec![Vec::new(); members.len()];
    for layer in &pieces {
        for &(u, l) in layer {
            below[u].push(l);
            above[l].push(u);
        }
    }
    let mut rows: Vec<Vec<usize>> = vec![Vec::new(); depth];
    let mut met = vec![false; members.len()];
    for start in 0..members.len() {
        if met[start] {
            continue;
        }
        let mut todo = vec![start];
        met[start] = true;
        while let Some(m) = todo.pop() {
            rows[members[m].layer].push(m);
            // The neighbours in reverse, so that the first is walked first.
            for &next in below[m].iter().rev().chain(above[m].iter().rev()) {
                if !met[next] {
                    met[next] = true;
                    todo.push(next);
                }
            }
        }
    }
    let place_of = |rows: &[Vec<usize>]| -> Vec<usize> {
        let mut place = vec![0; members.len()];
        for row in rows {
            for (at, &m) in row.iter().enumerate() {
                place[m] = at;
            }
        }
        place
    };
    let count = |rows: &[Vec<usize>]| -> usize {
        let place = place_of(rows);
        pieces
            .iter()
            .map(|layer| crossings_between(layer, &place, &place))
            .sum()
    };
    // The order is improved from several first orders, each made from the canonical order
    // and from nothing else: the members as they are met walking the edges, the same
    // reversed, the members in the order of their names, and that reversed. The best order
    // met is kept; of equals, the first.
    let walked = rows.clone();
    let mut by_name: Vec<Vec<usize>> = vec![Vec::new(); depth];
    for (m, member) in members.iter().enumerate() {
        by_name[member.layer].push(m);
    }
    let turned = |rows: &[Vec<usize>]| -> Vec<Vec<usize>> {
        rows.iter()
            .map(|row| row.iter().rev().copied().collect())
            .collect()
    };
    let starts = [
        walked.clone(),
        turned(&walked),
        by_name.clone(),
        turned(&by_name),
    ];
    let mut best = walked.clone();
    let mut fewest = count(&walked);
    for start in starts {
        if fewest == 0 {
            break;
        }
        let mut rows = start;
        let first = count(&rows);
        if first < fewest {
            fewest = first;
            best = rows.clone();
        }
        for sweep in 0..SWEEPS {
            if fewest == 0 {
                break;
            }
            let down = sweep % 2 == 0;
            let turns: Vec<usize> = if down {
                (1..depth).collect()
            } else {
                (0..depth.saturating_sub(1)).rev().collect()
            };
            for r in turns {
                let place = place_of(&rows);
                let towards = if down { &above } else { &below };
                // The median place of each member's neighbours on the layer before; a member
                // with none keeps its place.
                // Twice the median, so that the middle of two places is a whole number.
                let mut keyed: Vec<(usize, Option<usize>, usize)> = rows[r]
                    .iter()
                    .enumerate()
                    .map(|(at, &m)| {
                        let mut seen: Vec<usize> = towards[m].iter().map(|&o| place[o]).collect();
                        seen.sort_unstable();
                        let median = match seen.len() {
                            0 => None,
                            len if len % 2 == 1 => Some(2 * seen[len / 2]),
                            2 => Some(seen[0] + seen[1]),
                            len => {
                                // Between the two middle places, nearer the side where the
                                // neighbours stand closer together.
                                let (l, h) = (seen[len / 2 - 1], seen[len / 2]);
                                let (left, right) = (l - seen[0], seen[len - 1] - h);
                                Some(
                                    (2 * (l * right + h * left))
                                        .checked_div(left + right)
                                        .unwrap_or(l + h),
                                )
                            }
                        };
                        (m, median, at)
                    })
                    .collect();
                // Those with a median are put in the order of their medians among the places
                // those with a median had; the others stay where they were.
                let mut movable: Vec<(usize, usize, usize)> = keyed
                    .iter()
                    .filter_map(|&(m, median, at)| median.map(|key| (m, key, at)))
                    .collect();
                movable.sort_by_key(|&(_, key, at)| (key, at));
                let mut next = movable.into_iter();
                for entry in keyed.iter_mut() {
                    if entry.1.is_some()
                        && let Some((m, _, _)) = next.next()
                    {
                        entry.0 = m;
                    }
                }
                rows[r] = keyed.iter().map(|&(m, _, _)| m).collect();
            }
            // Neighbours exchanged where that leaves fewer crossings, until none does.
            let mut improved = true;
            let mut rounds = 0;
            while improved && rounds < 2 * members.len() + 2 {
                improved = false;
                rounds += 1;
                for r in 0..depth {
                    for at in 0..rows[r].len().saturating_sub(1) {
                        let before = local(&rows, &pieces, r, &place_of(&rows));
                        rows[r].swap(at, at + 1);
                        let after = local(&rows, &pieces, r, &place_of(&rows));
                        if after < before {
                            improved = true;
                        } else {
                            rows[r].swap(at, at + 1);
                        }
                    }
                }
            }
            let now = count(&rows);
            if now < fewest {
                fewest = now;
                best = rows.clone();
            }
        }
    }
    // A diagram small enough to try every order of every layer is given the fewest
    // crossings there are.
    if fewest > 0 {
        let mut orders: f64 = 1.0;
        for row in &best {
            for k in 1..=row.len() {
                orders *= k as f64;
            }
        }
        if orders <= EVERY_ORDER {
            let mut chosen: Vec<Vec<usize>> = Vec::new();
            let mut found = best.clone();
            every_order(
                &by_name,
                &pieces,
                members.len(),
                &mut chosen,
                &mut fewest,
                &mut found,
            );
            best = found;
        }
    }
    let mut rank = vec![0; n];
    for v in 0..n {
        rank[canon.order[v]] = rank_canon[v];
    }
    Ok(Layers {
        rank,
        rows: best
            .iter()
            .map(|row| row.iter().map(|&m| members[m].slot).collect())
            .collect(),
        crossings: fewest,
    })
}

/// Every order of every layer, layer by layer, a branch left as soon as the layers chosen
/// already cross no less than the best found: `fewest` and `found` are bettered where a
/// whole order crosses less. The orders of a layer are tried in the order of the names, so
/// the first best is the same in every run.
fn every_order(
    by_name: &[Vec<usize>],
    pieces: &[Vec<(usize, usize)>],
    members: usize,
    chosen: &mut Vec<Vec<usize>>,
    fewest: &mut usize,
    found: &mut Vec<Vec<usize>>,
) {
    let crossing = |chosen: &[Vec<usize>]| -> usize {
        let mut place = vec![0; members];
        for row in chosen {
            for (at, &m) in row.iter().enumerate() {
                place[m] = at;
            }
        }
        pieces
            .iter()
            .take(chosen.len().saturating_sub(1))
            .map(|layer| crossings_between(layer, &place, &place))
            .sum()
    };
    if chosen.len() == by_name.len() {
        let now = crossing(chosen);
        if now < *fewest {
            *fewest = now;
            *found = chosen.clone();
        }
        return;
    }
    let row = &by_name[chosen.len()];
    // The orders of the row, in the order of the names: Heap's method would be shorter and
    // would give them in another order.
    let mut order: Vec<usize> = Vec::with_capacity(row.len());
    let mut used = vec![false; row.len()];
    fn each(
        row: &[usize],
        order: &mut Vec<usize>,
        used: &mut [bool],
        visit: &mut dyn FnMut(&[usize]),
    ) {
        if order.len() == row.len() {
            visit(order);
            return;
        }
        for i in 0..row.len() {
            if !used[i] {
                used[i] = true;
                order.push(row[i]);
                each(row, order, used, visit);
                order.pop();
                used[i] = false;
            }
        }
    }
    let mut orders: Vec<Vec<usize>> = Vec::new();
    each(row, &mut order, &mut used, &mut |o| orders.push(o.to_vec()));
    for o in orders {
        if *fewest == 0 {
            return;
        }
        chosen.push(o);
        if crossing(chosen) < *fewest {
            every_order(by_name, pieces, members, chosen, fewest, found);
        }
        chosen.pop();
    }
}

/// The crossings that touch layer `r`: those with the layer above and with the layer below.
fn local(rows: &[Vec<usize>], pieces: &[Vec<(usize, usize)>], r: usize, place: &[usize]) -> usize {
    let mut total = 0;
    if r > 0 {
        total += crossings_between(&pieces[r - 1], place, place);
    }
    if r + 1 < rows.len() {
        total += crossings_between(&pieces[r], place, place);
    }
    total
}

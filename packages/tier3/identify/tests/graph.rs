//! The graph kernel (release 0.10.0, step 12; Language Reference, section 7.3): ancestors,
//! descendants, the nodes on proper causal paths and d-separation, each against a second,
//! independent way written here. Separation is decided by the kernel on the moral graph of the
//! ancestors, and here by walking every path between the two nodes and asking of each whether
//! it is blocked; the two share no code and no idea. They are held to each other on every graph
//! of causes over up to five nodes (30,000 graphs and some), on 2,000 generated graphs with
//! common causes, latent and selected nodes and sets of several nodes, and on the graphs of
//! the textbooks, whose answers are written out.

mod common;

use common::{Draws, Plain, acyclic, kernel, marks};
use zeff_core::dag::{Dag, Edge, Link};
use zeff_identify::graph::Graph;

#[test]
fn every_graph_of_causes_on_up_to_five_nodes() {
    // The numbers of graphs of causes on 0 to 5 named nodes (Robinson 1973).
    let known = [1usize, 1, 3, 25, 543, 29281];
    let (mut graphs, mut questions, mut separated) = (0usize, 0usize, 0usize);
    for (n, &wanted) in known.iter().enumerate() {
        let pairs: Vec<(usize, usize)> = (0..n)
            .flat_map(|a| (0..n).map(move |b| (a, b)))
            .filter(|(a, b)| a != b)
            .collect();
        let mut count = 0;
        for chosen in 0u32..(1 << pairs.len()) {
            let arrows: Vec<(usize, usize)> = pairs
                .iter()
                .enumerate()
                .filter(|(i, _)| chosen >> i & 1 == 1)
                .map(|(_, &p)| p)
                .collect();
            if !acyclic(n, &arrows) {
                continue;
            }
            count += 1;
            let plain = Plain::new(n, &arrows, &[], &[]);
            let graph = kernel(n, &arrows, &[], &[], &[]);
            let reaches = plain.reaches();
            // Ancestors and descendants of each node.
            for (v, row) in reaches.iter().enumerate() {
                let below: Vec<bool> = row.clone();
                let above: Vec<bool> = reaches.iter().map(|other| other[v]).collect();
                assert_eq!(graph.descendants(&[v]), below, "{arrows:?} below {v}");
                assert_eq!(graph.ancestors(&[v]), above, "{arrows:?} above {v}");
            }
            for x in 0..n {
                for y in 0..n {
                    if x == y {
                        continue;
                    }
                    // The nodes on the causal paths from x to y.
                    assert_eq!(
                        graph.proper_causal_nodes(&[x], &[y]),
                        plain.proper_causal(&[x], &[y]),
                        "{arrows:?} from {x} to {y}"
                    );
                    if x > y {
                        continue;
                    }
                    // Separation given every set of the other nodes.
                    let others: Vec<usize> = (0..n).filter(|&v| v != x && v != y).collect();
                    for set in 0u32..(1 << others.len()) {
                        let given: Vec<usize> = others
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| set >> i & 1 == 1)
                            .map(|(_, &v)| v)
                            .collect();
                        let answer = graph.d_separated(&[x], &[y], &given);
                        assert_eq!(
                            answer,
                            plain.separated(&[x], &[y], &given),
                            "{arrows:?}: {x} and {y} given {given:?}"
                        );
                        // Separation has no first and second side.
                        assert_eq!(answer, graph.d_separated(&[y], &[x], &given));
                        questions += 1;
                        separated += usize::from(answer);
                    }
                }
            }
        }
        assert_eq!(count, wanted, "graphs of causes on {n} nodes");
        graphs += count;
    }
    assert_eq!(graphs, 29854);
    // Ten pairs and eight sets for each graph on five nodes, and so on down.
    assert_eq!(questions, 29281 * 80 + 543 * 24 + 25 * 6 + 3);
    // Both answers are common: the test would not pass on a kernel that always said one.
    assert!(
        separated > questions / 10 && separated < questions * 9 / 10,
        "{separated} of {questions}"
    );
}

#[test]
fn two_thousand_generated_graphs_with_common_causes_and_selected_nodes() {
    let mut draws = Draws(0x2026_1128_0010_0012);
    let (mut questions, mut separated, mut with_selected, mut with_common) = (0, 0, 0, 0);
    for _ in 0..2000 {
        let n = 2 + draws.below(7);
        // A hidden order of the nodes, so that arrows go forward in it and never in a ring.
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            order.swap(i, draws.below(i + 1));
        }
        let (mut directed, mut bidirected) = (Vec::new(), Vec::new());
        for i in 0..n {
            for j in i + 1..n {
                if draws.chance(30) {
                    directed.push((order[i], order[j]));
                }
                if draws.chance(8) {
                    bidirected.push((order[i], order[j]));
                }
            }
        }
        let selected: Vec<usize> = (0..n).filter(|_| draws.chance(8)).collect();
        let latent: Vec<usize> = (0..n)
            .filter(|v| !selected.contains(v) && draws.chance(10))
            .collect();
        let plain = Plain::new(n, &directed, &bidirected, &selected);
        let graph = kernel(n, &directed, &bidirected, &latent, &selected);
        assert_eq!(graph.written(), n);
        assert_eq!(graph.len(), n + bidirected.len());
        with_selected += usize::from(!selected.is_empty());
        with_common += usize::from(!bidirected.is_empty());
        // The marks of the graph.
        for v in 0..graph.len() {
            assert_eq!(graph.is_latent(v), v >= n || latent.contains(&v));
            assert_eq!(graph.is_selected(v), selected.contains(&v));
        }
        let reaches = plain.reaches();
        for _ in 0..12 {
            // Two sets of one or two written nodes and a set given, drawn freely: they may
            // overlap, which the kernel has a rule for and the second way the same rule.
            let set = |draws: &mut Draws, most: usize| -> Vec<usize> {
                let size = 1 + draws.below(most);
                let mut s: Vec<usize> = (0..size).map(|_| draws.below(n)).collect();
                s.sort_unstable();
                s.dedup();
                s
            };
            let (x, y) = (set(&mut draws, 2), set(&mut draws, 2));
            let given: Vec<usize> = (0..n).filter(|_| draws.chance(25)).collect();
            let answer = graph.d_separated(&x, &y, &given);
            assert_eq!(
                answer,
                plain.separated(&x, &y, &given),
                "{directed:?} {bidirected:?} selected {selected:?}: {x:?} and {y:?} given {given:?}"
            );
            assert_eq!(answer, graph.d_separated(&y, &x, &given));
            questions += 1;
            separated += usize::from(answer);
            // Ancestors and descendants of a set, and the nodes on its causal paths.
            let below: Vec<bool> = (0..plain.n)
                .map(|w| x.iter().any(|&v| reaches[v][w]))
                .collect();
            let above: Vec<bool> = (0..plain.n)
                .map(|w| x.iter().any(|&v| reaches[w][v]))
                .collect();
            assert_eq!(graph.descendants(&x), below);
            assert_eq!(graph.ancestors(&x), above);
            assert_eq!(
                graph.proper_causal_nodes(&x, &y),
                plain.proper_causal(&x, &y),
                "{directed:?} {bidirected:?}: from {x:?} to {y:?}"
            );
        }
    }
    assert_eq!(questions, 24000);
    assert!(separated > 2000 && separated < 22000, "{separated}");
    assert!(
        with_selected > 400 && with_common > 800,
        "{with_selected} {with_common}"
    );
}

#[test]
fn the_graphs_of_the_textbooks() {
    let none: [usize; 0] = [];
    // A chain x -> m -> y: the ends hang together, and the middle parts them.
    let chain = kernel(3, &[(0, 1), (1, 2)], &[], &[], &[]);
    assert!(!chain.d_separated(&[0], &[2], &none));
    assert!(chain.d_separated(&[0], &[2], &[1]));
    // A fork x <- z -> y: the same.
    let fork = kernel(3, &[(1, 0), (1, 2)], &[], &[], &[]);
    assert!(!fork.d_separated(&[0], &[2], &none));
    assert!(fork.d_separated(&[0], &[2], &[1]));
    // A collider x -> c <- y: the ends are apart, and the middle joins them.
    let collider = kernel(3, &[(0, 1), (2, 1)], &[], &[], &[]);
    assert!(collider.d_separated(&[0], &[2], &none));
    assert!(!collider.d_separated(&[0], &[2], &[1]));
    // A descendant of the collider joins them too.
    let with_child = kernel(4, &[(0, 1), (2, 1), (1, 3)], &[], &[], &[]);
    assert!(with_child.d_separated(&[0], &[2], &none));
    assert!(!with_child.d_separated(&[0], &[2], &[3]));
    // The same collider when the data are selected on it, or on its descendant.
    for chosen in [1usize, 3] {
        let selected = kernel(4, &[(0, 1), (2, 1), (1, 3)], &[], &[], &[chosen]);
        assert!(
            !selected.d_separated(&[0], &[2], &none),
            "selected on {chosen}"
        );
        assert!(selected.is_selected(chosen));
    }
    // The M graph, u1 -> x, u1 -> m, u2 -> m, u2 -> y: x and y are apart until m is given.
    let m = kernel(5, &[(0, 2), (0, 4), (1, 4), (1, 3)], &[], &[0, 1], &[]);
    assert!(m.d_separated(&[2], &[3], &none));
    assert!(!m.d_separated(&[2], &[3], &[4]));
    assert!(m.d_separated(&[2], &[3], &[4, 0]));
    assert!(m.is_latent(0) && m.is_latent(1) && !m.is_latent(2));
    // The same M with its two hidden causes written as bidirected edges: x <-> m <-> y.
    let arcs = kernel(3, &[], &[(0, 2), (2, 1)], &[], &[]);
    assert_eq!((arcs.written(), arcs.len()), (3, 5));
    assert_eq!(arcs.name(3), "v0 <-> v2");
    assert!(arcs.is_latent(3) && arcs.is_latent(4));
    assert_eq!(arcs.children(3), [0, 2]);
    assert_eq!(arcs.parents(2), [3, 4]);
    assert!(arcs.d_separated(&[0], &[1], &none));
    assert!(!arcs.d_separated(&[0], &[1], &[2]));
    // A confounder that is not observed, as a bidirected edge: nothing written parts the two.
    let confounded = kernel(2, &[(0, 1)], &[(0, 1)], &[], &[]);
    assert!(!confounded.d_separated(&[0], &[1], &none));
    // The graph of section 7.3 of the Language Reference: ability 0, training 1,
    // productivity 2, tenure 3, age 4, region 5.
    let study = kernel(
        6,
        &[(0, 1), (1, 2), (0, 2), (1, 3), (3, 2), (4, 1), (5, 1)],
        &[],
        &[0],
        &[],
    );
    // Age and region are apart, and training joins them; age and productivity hang together
    // through training, and training with ability parts them.
    assert!(study.d_separated(&[4], &[5], &none));
    assert!(!study.d_separated(&[4], &[5], &[1]));
    assert!(!study.d_separated(&[4], &[2], &none));
    assert!(!study.d_separated(&[4], &[2], &[1]));
    assert!(study.d_separated(&[4], &[2], &[1, 0]));
    assert!(study.d_separated(&[4], &[3], &[1]));
    // Tenure lies on a causal path from training to productivity, and productivity ends one;
    // ability, age and region lie on none.
    assert_eq!(study.proper_causal_nodes(&[1], &[2]), marks(&[2, 3], 6));
    assert_eq!(study.ancestors(&[1]), marks(&[0, 1, 4, 5], 6));
    assert_eq!(study.descendants(&[1]), marks(&[1, 2, 3], 6));
    assert_eq!(study.node("v3"), Some(3));
    assert_eq!(study.node("v9"), None);
    // Sets: a path that leaves the exposure and comes back into it is no proper causal path.
    let two = kernel(4, &[(0, 1), (1, 2), (2, 3)], &[], &[], &[]);
    assert_eq!(two.proper_causal_nodes(&[0, 2], &[3]), marks(&[3], 4));
    assert_eq!(two.proper_causal_nodes(&[0], &[3]), marks(&[1, 2, 3], 4));
    // Two sets that share a node are not separated, unless that node is given.
    assert!(!two.d_separated(&[0, 1], &[1, 3], &[2]));
    assert!(two.d_separated(&[0, 1], &[1, 3], &[1, 2]));
    // An undirected edge takes no part, and the graph says that it has one.
    let drawn = Graph::of(&Dag::new(
        vec!["a".to_string(), "b".to_string()],
        vec![Edge {
            from: 0,
            to: 1,
            link: Link::Undirected,
        }],
        Vec::new(),
        Vec::new(),
    ));
    assert!(drawn.has_undirected());
    assert!(drawn.d_separated(&[0], &[1], &none));
    assert!(!chain.has_undirected());
    // A graph of nothing.
    let empty = Graph::of(&Dag::new(Vec::new(), Vec::new(), Vec::new(), Vec::new()));
    assert!(empty.is_empty());
    assert_eq!(empty.written(), 0);
}

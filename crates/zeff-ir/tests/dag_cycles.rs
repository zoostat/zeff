//! The search for a directed cycle in a `dag { }` block (release 0.10.0, step 11; Language
//! Reference, section 7.3): against a second method, the removal of nodes that nothing points
//! to, on every graph of up to four nodes and on generated graphs of up to twelve; and that
//! the cycle named is one, in the order the edges were written.

use proptest::prelude::*;
use zeff_ir::check::directed_cycle;

/// Whether the edges have a cycle, by another way: take away, as long as there is one, a node
/// that no remaining edge points to; a cycle is what is left.
fn has_cycle(count: usize, edges: &[(usize, usize, usize)]) -> bool {
    let mut left: Vec<bool> = vec![true; count];
    loop {
        let free = (0..count).find(|&node| {
            left[node] && !edges.iter().any(|&(from, to, _)| left[from] && to == node)
        });
        match free {
            Some(node) => left[node] = false,
            None => return left.iter().any(|&l| l),
        }
    }
}

/// The checks of one graph: the two methods agree, and a cycle that is named is a cycle.
fn check(count: usize, edges: &[(usize, usize, usize)]) {
    let found = directed_cycle(count, edges);
    assert_eq!(found.is_some(), has_cycle(count, edges), "{edges:?}");
    if let Some((cycle, closing)) = found {
        assert!(!cycle.is_empty(), "{edges:?}");
        // Each node of the cycle has an edge to the next, and the edge that closes it leads
        // from the last to the first.
        for pair in cycle.windows(2) {
            assert!(
                edges
                    .iter()
                    .any(|&(from, to, _)| from == pair[0] && to == pair[1]),
                "{cycle:?} in {edges:?}"
            );
        }
        let (last, first) = (cycle[cycle.len() - 1], cycle[0]);
        let closes = edges.iter().find(|e| e.2 == closing).unwrap();
        assert_eq!(
            (closes.0, closes.1),
            (last, first),
            "{cycle:?} in {edges:?}"
        );
        // No node twice.
        let mut seen = cycle.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), cycle.len(), "{cycle:?}");
    }
}

#[test]
fn every_graph_of_up_to_four_nodes() {
    let mut graphs = 0;
    for count in 0..=4usize {
        let pairs: Vec<(usize, usize)> = (0..count)
            .flat_map(|a| (0..count).map(move |b| (a, b)))
            .filter(|(a, b)| a != b)
            .collect();
        for chosen in 0u32..(1 << pairs.len()) {
            let edges: Vec<(usize, usize, usize)> = pairs
                .iter()
                .enumerate()
                .filter(|(i, _)| chosen >> i & 1 == 1)
                .map(|(i, &(from, to))| (from, to, i))
                .collect();
            check(count, &edges);
            graphs += 1;
        }
    }
    // 1 + 1 + 4 + 64 + 4096 sets of directed edges.
    assert_eq!(graphs, 4166);
}

#[test]
fn the_cycle_named_is_the_first_met_in_the_order_written() {
    // A chain with its end joined to its start: the whole chain, from the first node.
    let edges = [(0, 1, 0), (1, 2, 1), (2, 0, 2)];
    assert_eq!(directed_cycle(3, &edges), Some((vec![0, 1, 2], 2)));
    // Two cycles: the one reached first from the first node, by its edges in order.
    let edges = [(0, 1, 0), (1, 0, 1), (1, 2, 2), (2, 1, 3)];
    assert_eq!(directed_cycle(3, &edges), Some((vec![0, 1], 1)));
    // A cycle away from the first node is named from the node the walk enters it at.
    let edges = [(0, 1, 0), (1, 2, 1), (2, 3, 2), (3, 1, 3)];
    assert_eq!(directed_cycle(4, &edges), Some((vec![1, 2, 3], 3)));
    // A diamond is no cycle, and neither is nothing.
    assert_eq!(
        directed_cycle(4, &[(0, 1, 0), (0, 2, 1), (1, 3, 2), (2, 3, 3)]),
        None
    );
    assert_eq!(directed_cycle(0, &[]), None);
    // A chain of a hundred thousand nodes is walked without a deep stack of calls.
    let long: Vec<(usize, usize, usize)> = (0..99_999).map(|i| (i, i + 1, i)).collect();
    assert_eq!(directed_cycle(100_000, &long), None);
    let mut closed = long.clone();
    closed.push((99_999, 0, 99_999));
    let (cycle, closing) = directed_cycle(100_000, &closed).unwrap();
    assert_eq!((cycle.len(), closing), (100_000, 99_999));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn generated_graphs_of_up_to_twelve_nodes(
        count in 1usize..=12,
        raw in prop::collection::vec((0usize..12, 0usize..12), 0..40),
    ) {
        let edges: Vec<(usize, usize, usize)> = raw
            .iter()
            .map(|&(a, b)| (a % count, b % count))
            .filter(|(a, b)| a != b)
            .enumerate()
            .map(|(i, (a, b))| (a, b, i))
            .collect();
        check(count, &edges);
    }
}

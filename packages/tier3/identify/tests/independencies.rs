//! `g.implied_independencies()` (release 0.10.0, step 13; catalogue row 431; open decision
//! #246, option A): the list against a second way written here, which tries every set of
//! observed variables for every pair, smallest first and in the order of the names, and
//! decides each question by walking every path. It uses neither the ancestors by which the
//! function knows at once whether a pair can be separated, nor its search. The two are held to
//! each other on every graph of causes over up to four nodes with each choice of one latent
//! node or none, on every graph over five, and on 2,000 generated graphs with common causes,
//! latent and selected nodes; then the function through scripts: the graph of section 7.3,
//! the one example of dagitty's manual page, every form and every message.

mod common;

use common::{Draws, Plain, acyclic, kernel};
use proptest::prelude::*;
use zeff_identify::independencies::{Independence, implied, implied_within};
use zeff_testkit::run;

/// The second way. The nodes are called `v0`, `v1`, … and there are at most ten, so the order
/// of the names is the order of the numbers.
fn by_trying_every_set(
    written: usize,
    directed: &[(usize, usize)],
    bidirected: &[(usize, usize)],
    latent: &[usize],
    selected: &[usize],
) -> Vec<Independence> {
    let plain = Plain::new(written, directed, bidirected, selected);
    let observed: Vec<usize> = (0..written)
        .filter(|v| !latent.contains(v) && !selected.contains(v))
        .collect();
    let joined = |a: usize, b: usize| {
        directed.contains(&(a, b))
            || directed.contains(&(b, a))
            || bidirected.contains(&(a, b))
            || bidirected.contains(&(b, a))
    };
    let mut list = Vec::new();
    for (i, &x) in observed.iter().enumerate() {
        for &y in &observed[i + 1..] {
            if joined(x, y) {
                continue;
            }
            let others: Vec<usize> = observed
                .iter()
                .copied()
                .filter(|&v| v != x && v != y)
                .collect();
            // Every set of the others, the smaller first and, of one size, the earlier by
            // name first: with at most eight others, the sets are the numbers below 256.
            let mut sets: Vec<Vec<usize>> = (0u32..(1 << others.len()))
                .map(|bits| {
                    others
                        .iter()
                        .enumerate()
                        .filter(|(k, _)| bits >> k & 1 == 1)
                        .map(|(_, &v)| v)
                        .collect()
                })
                .collect();
            sets.sort_by(|a: &Vec<usize>, b: &Vec<usize>| (a.len(), a).cmp(&(b.len(), b)));
            if let Some(given) = sets
                .into_iter()
                .find(|set| plain.separated(&[x], &[y], set))
            {
                list.push(Independence { x, y, given });
            }
        }
    }
    list.sort_by(|a, b| {
        (a.given.len(), a.x, a.y, &a.given).cmp(&(b.given.len(), b.x, b.y, &b.given))
    });
    list
}

#[test]
fn every_graph_of_up_to_five_nodes_with_a_latent_node_or_none() {
    let (mut graphs, mut statements, mut with_given) = (0usize, 0usize, 0usize);
    for n in 0..=5usize {
        let pairs: Vec<(usize, usize)> = (0..n)
            .flat_map(|a| (0..n).map(move |b| (a, b)))
            .filter(|(a, b)| a != b)
            .collect();
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
            // No latent node, and, up to four nodes, each node latent in turn.
            let mut hidden: Vec<Vec<usize>> = vec![Vec::new()];
            if n <= 4 {
                hidden.extend((0..n).map(|v| vec![v]));
            }
            for latent in hidden {
                let got = implied(&kernel(n, &arrows, &[], &latent, &[]));
                assert_eq!(got.unproved, 0);
                let wanted = by_trying_every_set(n, &arrows, &[], &latent, &[]);
                assert_eq!(got.list, wanted, "{arrows:?} with {latent:?} latent");
                graphs += 1;
                statements += wanted.len();
                with_given += wanted.iter().filter(|s| !s.given.is_empty()).count();
            }
        }
    }
    // 29,854 graphs, and the 573 of up to four nodes again for each of their nodes.
    assert_eq!(graphs, 29854 + 3 * 2 + 25 * 3 + 543 * 4 + 1);
    assert!(
        statements > 50_000 && with_given > 20_000,
        "{statements} {with_given}"
    );
}

#[test]
fn two_thousand_generated_graphs_with_common_causes_latent_and_selected_nodes() {
    let mut draws = Draws(0x2026_1128_0013_0001);
    let (mut statements, mut none, mut with_selected) = (0usize, 0usize, 0usize);
    for _ in 0..2000 {
        let n = 2 + draws.below(7);
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            order.swap(i, draws.below(i + 1));
        }
        let (mut directed, mut bidirected) = (Vec::new(), Vec::new());
        for i in 0..n {
            for j in i + 1..n {
                if draws.chance(28) {
                    directed.push((order[i], order[j]));
                }
                if draws.chance(7) {
                    bidirected.push((order[i], order[j]));
                }
            }
        }
        let selected: Vec<usize> = (0..n).filter(|_| draws.chance(8)).collect();
        let latent: Vec<usize> = (0..n)
            .filter(|v| !selected.contains(v) && draws.chance(12))
            .collect();
        let graph = kernel(n, &directed, &bidirected, &latent, &selected);
        let got = implied(&graph);
        assert_eq!(got.unproved, 0);
        let wanted = by_trying_every_set(n, &directed, &bidirected, &latent, &selected);
        assert_eq!(
            got.list, wanted,
            "{directed:?} {bidirected:?} latent {latent:?} selected {selected:?}"
        );
        statements += wanted.len();
        none += usize::from(wanted.is_empty());
        with_selected += usize::from(!selected.is_empty());
        // No latent and no selected node in any statement; each pair once, the earlier name
        // first.
        for s in &got.list {
            for v in [s.x, s.y].iter().chain(&s.given) {
                assert!(!latent.contains(v) && !selected.contains(v));
            }
            assert!(s.x < s.y);
        }
        // With the search cut off at once, every pair is still there and its set still
        // separates and cannot lose a variable; it may only be larger than the smallest.
        let rough = implied_within(&graph, 0);
        assert_eq!(rough.list.len(), got.list.len());
        for s in &rough.list {
            assert!(graph.d_separated(&[s.x], &[s.y], &s.given));
            for &v in &s.given {
                let without: Vec<usize> = s.given.iter().copied().filter(|&k| k != v).collect();
                assert!(!graph.d_separated(&[s.x], &[s.y], &without));
            }
            let exact = got.list.iter().find(|e| (e.x, e.y) == (s.x, s.y)).unwrap();
            assert!(exact.given.len() <= s.given.len());
        }
        assert_eq!(rough.unproved, rough.list.len());
    }
    assert!(
        statements > 4000 && none > 100 && with_selected > 400,
        "{statements} {none} {with_selected}"
    );
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

fn printed(script: &str) -> String {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
}

const TRAINING: &str = "dag {\n    ability -> training -> productivity\n    ability -> productivity\n    training -> tenure -> productivity\n    [age, region] -> training\n    latent ability\n}";

#[test]
fn the_graph_of_section_7_3_and_the_example_of_dagittys_manual() {
    // Worked by hand: age and region have no common cause and meet only at training; tenure
    // hangs on training alone; nothing observed parts age or region from productivity, since
    // holding training fixed opens the way through ability, which is latent.
    assert_eq!(
        printed(&format!("{TRAINING}.implied_independencies()\n")),
        "implied_independencies · 3 among 5 observed variables\n\n  variable   independent of   given\n  age        region\n  age        tenure           training\n  region     tenure           training\n\nlatent, in no statement: ability\n"
    );
    // The example of the manual page of dagitty's `impliedConditionalIndependencies`
    // (`x -> m -> y`: its one statement is "x _||_ y | m", and none once `m` is latent).
    assert_eq!(
        printed("dag { x -> m -> y }.implied_independencies()\n"),
        "implied_independencies · 1 among 3 observed variables\n\n  variable   independent of   given\n  x          y                m\n"
    );
    let hidden = printed("dag {\n    x -> m -> y\n    latent m\n}.implied_independencies()\n");
    assert!(
        hidden.starts_with("implied_independencies · 0 among 2 observed variables\n"),
        "{hidden}"
    );
    // The number of statements as a value.
    assert_eq!(
        printed(&format!(
            "let r = {TRAINING}.implied_independencies()\nprint(r.statistic(), r.n_obs())\n"
        )),
        "3 3\n"
    );
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "let g = {TRAINING}\ng.implied_independencies()\ndag {{ x -> m -> y }}.implied_independencies()\ndag {{\n    x -> m -> y\n    latent m\n}}.implied_independencies()\ndag {{\n    educ -> skill -> wage\n    skill -> employed\n    union -> wage\n    selected employed\n}}.implied_independencies()\ndag {{\n    a -> c\n    b -> c\n    c -> d\n    selected d\n}}.implied_independencies()\ndag {{\n    x <-> y\n    y -> z\n    w\n}}.implied_independencies()\ndag {{\n    u -> [x, y]\n    v -> [x, y]\n    z -> x\n}}.implied_independencies()\ndag {{ a -> b }}.implied_independencies()\ndag {{ a }}.implied_independencies()\ndag {{ }}.implied_independencies()\ndag {{\n    hidden -> [a, b]\n    latent hidden, other\n    selected chosen, kept\n}}.implied_independencies()\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("independencies_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "dag {\n    price -- demand\n    season -> demand\n}.implied_independencies()\n",
        "let g = dag { a -> b }\ng.implied_independencies(a)\n",
        "let g = dag { a -> b }\ng.implied_independencies(type: basis_set)\n",
        "dataset(wages).implied_independencies()\n",
        "implied_independencies()\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("independencies_messages", all);
}

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        * 48
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// The list does not depend on how the graph was written: its lines in another order, an
    /// arrow written twice, give the same list to the character.
    #[test]
    fn the_order_of_the_lines_changes_nothing(
        pairs in prop::collection::vec((0usize..6, 0usize..6, 0u8..8), 1..12),
        hidden in prop::collection::vec(0usize..6, 0..2),
        turn in 0usize..12,
    ) {
        const NAMES: [&str; 6] = ["age", "b", "cost", "d", "educ", "zeal"];
        let mut lines: Vec<String> = pairs
            .iter()
            .filter(|(i, j, _)| i != j)
            .map(|&(i, j, kind)| {
                let (low, high) = (i.min(j), i.max(j));
                if kind == 0 {
                    format!("{} <-> {}", NAMES[low], NAMES[high])
                } else {
                    format!("{} -> {}", NAMES[low], NAMES[high])
                }
            })
            .collect();
        for &at in &hidden {
            lines.push(format!("latent {}", NAMES[at]));
        }
        prop_assume!(!lines.is_empty());
        let block = |lines: &[String]| format!("dag {{\n    {}\n}}", lines.join("\n    "));
        let first = printed(&format!("{}.implied_independencies()\n", block(&lines)));
        let mut turned = lines.clone();
        turned.rotate_left(turn % lines.len());
        turned.reverse();
        turned.push(lines[0].clone());
        let second = printed(&format!("{}.implied_independencies()\n", block(&turned)));
        prop_assert_eq!(first, second);
    }
}
